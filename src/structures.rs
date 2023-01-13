// #![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// #![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Index;

use rand::prelude::*;
use rand_distr::{Distribution, Pert};

use enterpolation::bezier::{Bezier, BezierBuilder, BezierError};
use enterpolation::linear::{Linear, LinearBuilder, LinearError};
use enterpolation::{DiscreteGenerator, Generator};

#[derive(Debug, Clone, PartialEq)]
pub enum Dir {
    H,
    V,
    N,
    E,
    S,
    W,
    X
}

impl Dir {
    fn flip(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
            Dir::H => Dir::H,
            Dir::V => Dir::V,
            Dir::X => Dir::X,
        }
    }
}

#[derive(Debug)]
pub enum IterPos {
    Start,
    End,
    Idx(usize),
}

#[derive(Debug)]
pub enum IterSpec {
    Edges,
    AllInner,
    From(usize, usize, Dir),
    Range((usize, usize), (usize, usize)),
}

pub trait Flat2d {
    type DataPoint;
    // Required methods ===============================================
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn data(&self) -> Vec<Self::DataPoint>;

    // Default implementations ========================================
    fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.rows() + col
    }
    fn get(&self, row: usize, col: usize) -> Self::DataPoint {
        let idx = self.get_index(row, col);
        self.data()[idx]
    }
    fn set(&mut self, row: usize, col: usize, value: Self::DataPoint) {
        let idx = self.get_index(row, col);
        self.data()[idx] = value;
    }
}

#[derive(Debug, Clone)]
pub struct IndexedGrid<T> {
    row_idxs: Vec<usize>,
    col_idxs: Vec<usize>,
    iter_q: VecDeque<(usize, usize)>,
    default: T,
    data: Vec<T>,
}

impl<T> Flat2d for IndexedGrid<T> {
    type DataPoint = T;
    fn rows(&self) -> usize {
        self.row_idxs.len()
    }
    fn cols(&self) -> usize {
        self.col_idxs.len()
    }
    fn get_index(&self, row: usize, col: usize) -> usize {
        let real_row = match self.row_idxs.binary_search(&row) {
            Ok(rr) => rr,
            Err(e) => panic!("Invalid row index: {} <<{:?}>>", row, e),
        };
        let real_col = match self.col_idxs.binary_search(&col) {
            Ok(rr) => rr,
            Err(e) => panic!("Invalid column index: {} <<{:?}>>", col, e),
        };
        real_row * self.rows() + real_col
    }
    fn data(&self) -> Vec<T> {
        self.data
    }
}

impl<T> Iterator for IndexedGrid<T> {
    type Item = usize;
    // Iterates over the row or column indexes, depending on self.iter_dir
    // See also IndexedGrid::setup_iteration.
    fn next(&mut self) -> Option<Self::Item> {
        let data = match self.iter_dir {
            Dir::H | Dir::E => self.col_idxs,
            Dir::V | Dir::S => self.row_idxs,
            Dir::W => {
                let data_ = self.col_idxs.clone();
                data_.reverse();
                data_
            },
            Dir::N => {
                let data_ = self.row_idxs.clone();
                data_.reverse();
                data_
            },
            Dir::X => panic!("iter_dir is set to Dir::X!"),
        };
        if self.iter_pos >= data.len() {
            None
        } else {
            let result = Some(data[self.iter_pos]);
            self.iter_pos += 1;
            result
        }
    }
}

// This is used only to verify that the layout grids match
impl<T> PartialEq for IndexedGrid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.row_idxs == other.row_idxs
            && self.col_idxs == other.col_idxs
            && self.data.len() == other.data.len()
    }
}

impl<T: Clone> IndexedGrid<T> {
    pub fn from(mut points: Vec<(usize, usize, T)>, default: T) -> Self {
        let mut row_idxs = Vec::new();
        let mut col_idxs = Vec::new();
        let seen = Vec::new();
        for (r, c, _) in points.iter() {
            assert!(!seen.contains(&(r, c)));
            row_idxs.push(*r);
            col_idxs.push(*c);
            seen.push((r, c));
        }
        row_idxs.sort_unstable();
        row_idxs.dedup();
        col_idxs.sort_unstable();
        col_idxs.dedup();
        let cap = row_idxs.len() * col_idxs.len();
        let mut data = vec![default; cap];
        for (r, c, h) in points.iter() {
            let real_row = row_idxs
                .binary_search(&r)
                .expect("Invalid row index in IndexedGrid::from().");
            let real_col = col_idxs
                .binary_search(&r)
                .expect("Invalid col index in IndexedGrid::from().");
            let idx = real_row * row_idxs.len() + real_col;
            data[idx] = *h;
        }
        IndexedGrid {
            row_idxs,
            col_idxs,
            iter_pos: 0,
            iter_dir: Dir::X,
            default,
            data,
        }
    }
    pub fn clone_blank(&self) -> Self {
        let row_idxs = self.row_idxs.clone();
        let col_idxs = self.col_idxs.clone();
        let cap = row_idxs.len() * col_idxs.len();
        let default = self.default,
        let data = vec![default; cap];
        IndexedGrid {
            row_idxs,
            col_idxs,
            default,
            data,
        }
    }
    pub fn setup_iteration(&mut self, spec: IterSpec) {
        let q = VecDeque::new();
        match spec {
            IterSpec::Edges => {
                for col in self.col_idxs {
                    q.push_back((0, col));
                }
                let ri_len = self.row_idxs.len();
                let last_col = self.col_idxs.last().unwrap();
                for row in &self.row_idxs[1..(ri_len - 1)] {
                    q.push_back((*row, 0));
                    q.push_back((*row, last_col));
                }
                let last_row = self.row_idxs.last().unwrap();
                for col in self.col_idxs {
                    q.push_back((last_row, col));
                }
            },
            IterSpec::AllInner => {},
            IterSpec::From(row, col, dir) => {},
            IterSpec::Range((r1, c1), (r2, c2)) => {},
        }
        self.iter_q = q;
    }
}

#[cfg(test)]
mod test {
    use crate::{Flat2d, IndexedGrid};
    #[test]
    fn create_igrid() {
        let igrid: IndexedGrid<f32> =
            IndexedGrid::from(vec![(0, 4, 27.5), (3, 4, 12.1), (10, 7, 44.6)], 0.);
        assert_eq!(igrid.get(0, 4), 27.5);
        assert_eq!(igrid.get(3, 4), 12.1);
        assert_eq!(igrid.get(10, 7), 44.6);
    }
}

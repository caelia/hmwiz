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

/*
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
pub enum SeqSpec {
    Edges,
    AllInner,
    Row(usize),
    Col(usize),
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
    sequence: Vec<(usize, usize)>,
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
        let sequence = Vec::new();
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
            sequence,
            default,
            data,
        }
    }

    pub fn clone_blank(&self) -> Self {
        let row_idxs = self.row_idxs.clone();
        let col_idxs = self.col_idxs.clone();
        let sequence = Vec::new();
        let cap = row_idxs.len() * col_idxs.len();
        let default = self.default,
        let data = vec![default; cap];
        IndexedGrid {
            row_idxs,
            col_idxs,
            sequence,
            default,
            data,
        }
    }

    pub fn set_sequence(&mut self, spec: SeqSpec) {
        self.sequence.clear();
        match spec {
            SeqSpec::Edges => {
                for col in self.col_idxs {
                    self.sequence.push((0, col));
                }
                let ri_limit= self.row_idxs.len() - 1;
                let last_col = self.col_idxs.last().unwrap();
                for row in &self.row_idxs[1..ri_limit] {
                    self.sequence.push((*row, 0));
                    self.sequence.push((*row, last_col));
                }
                let last_row = self.row_idxs.last().unwrap();
                for col in self.col_idxs {
                    self.sequence.push((last_row, col));
                }
            },
            SeqSpec::AllInner => {
                let ri_limit = self.row_idxs.len() - 1;
                let ci_limit = self.col_idxs.len() - 1;
                for row in &self.row_idxs[1..ri_limit] {
                    for col in &self.col_idxs[1..ci_limit] {
                        self.sequence.push((*row, *col));
                    }
                }
            },
            SeqSpec::Row(row) => {
                for col in self.col_idxs {
                    self.sequence.push((row, col));
                }
            },
            SeqSpec::Col(col) => {
                for row in self.row_idxs {
                    self.sequence.push((row, col));
                }
            },
            SeqSpec::From(row, col, dir) => {
                let ri_start = self.row_idxs.binary_search(&row).unwrap();
                let ci_start = self.col_idxs.binary_search(&col).unwrap();
                match dir {
                    Dir::N => {
                        for meta_idx in (0..ri_start).rev() {
                            self.sequence.push((self.row_idxs[meta_idx], ci_start));
                        }
                    },
                    Dir::S => {
                        for meta_idx in ri_start..self.row_idxs.len() {
                            self.sequence.push((self.row_idxs[meta_idx], ci_start));
                        }
                    },
                    Dir::E => {
                        for meta_idx in ci_start..self.col_idxs.len() {
                            self.sequence.push((ri_start, self.col_idxs[meta_idx]));
                        }
                    },
                    Dir::W => {
                        for meta_idx in (0..ci_start).rev() {
                            self.sequence.push((ri_start, self.col_idxs[meta_idx]));
                        }
                    },
                    _ => panic!("Invalid iteration direction: {:?}", dir),
                }
            },
            SeqSpec::Range((r1, c1), (r2, c2)) => {
                if r1 == r2 {
                    if c1 > c2 {
                        for meta_idx in (c2..c1).rev() {
                            self.sequence.push((r1, self.col_idxs[meta_idx]));
                        }
                    } else {
                        for col in &self.col_idxs[c1..c2] {
                            self.sequence.push((r1, *col));
                        }
                    }
                } else if c1 == c2 {
                    if r1 > r2 {
                        for meta_idx in (r2..r1).rev() {
                            self.sequence.push((self.row_idxs[meta_idx], c1));
                        }
                    } else {
                        for row in &self.row_idxs[r1..r2] {
                            self.sequence.push((*row, c1));
                        }
                    }
                } else {
                    panic!("Can't iterate over range ({}, {}) - ({}, {})", r1, c1, r2, c2 );
                }
            },
        }
    }

    pub fn indices(&self) -> (Vec<usize>, Vec<usize>) {
        (self.row_idxs, self.col_idxs)
    }

    pub fn seq_find_nearest<F>(&self, f: F) -> Option<(usize, usize, f32)>
            where F: Fn(usize, usize) -> bool {
        for (row, col) in self.sequence {
            if f(row, col) {
                return Some((row, col));
            }
        }
        None
    }

    pub fn seq_do<F>(&mut self, f: F) where F: FnMut(usize, usize) {
         for (row, col) in self.sequence {
            f(row, col);
        }
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
*/

#[derive(Debug)]
pub enum GridOrientation {
    RowMajor,
    ColumnMajor,
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}


impl<T> Grid<T> {
    pub fn new(rows: usize, cols: usize, orientation: GridOrientation, default: T) -> Self {
        Grid { rows, cols, orientation, data: vec![default; rows * cols]}
    }
    fn get_index(&self, row: usize, col: usize) -> usize {
        match self.orientation {
            GridOrientation::RowMajor => row * self.cols + col,
            GridOrientation::ColumnMajor => col * self.rows + row,
        }
    }
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[self.get_index(row, col)]
    }
    pub fn get_rank(&self, index: usize) -> Vec<T> {
        let (nrows, ncols) = (self.rows, self.cols);
        let (start, end) = match self.orientation {
            GridOrientation::RowMajor => (index * ncols, (index + 1) *  ncols),
            GridOrientation::ColumnMajor => (index * nrows, (index + 1) * nrows),
        };
        vec![self.data[start..end]]
    }
    pub fn get_range(&self, index: usize, start: usize, end: usize) -> Vec<T> {
        self.get_rank(index)[start..end]
    }
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[self.get_index(row, col)] = value;
    }
}
// #![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// #![allow(dead_code)]

use std::ops::Index;
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;

use rand::prelude::*;
use rand_distr::{Pert, Distribution};

use enterpolation::linear::{Linear, LinearBuilder, LinearError};
use enterpolation::bezier::{Bezier, BezierBuilder, BezierError};
use enterpolation::{Generator, DiscreteGenerator};

#[derive(Debug)]
pub enum Dir {
    H,
    V
}

pub trait Flat2d {
    type DataPoint;
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.rows() + col
    }
    fn data(&self) -> Vec<Self::DataPoint>;
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
    fn data(&self) -> Vec<T> { self.data }
}

impl<T: Clone> IndexedGrid<T> {
    pub fn from(mut points: Vec<(usize, usize, T)>, default: T) -> Self {
        let mut row_idxs = Vec::new();
        let mut col_idxs = Vec::new();
        let seen = Vec::new();
        for (r, c, _) in points.iter() {
            row_idxs.push(*r);
            col_idxs.push(*c);
            assert!(!seen.contains(&(r, c)));
            seen.push((r, c));
        }
        row_idxs.sort_unstable();
        row_idxs.dedup();
        col_idxs.sort_unstable();
        col_idxs.dedup();
        let cap = row_idxs.len() * col_idxs.len();
        let mut data = vec![default; cap];
        for (r, c, h) in points.iter() {
            let real_row = row_idxs.binary_search(&r).expect("Invalid row index in IndexedGrid::from().");
            let real_col = col_idxs.binary_search(&r).expect("Invalid col index in IndexedGrid::from().");
            let idx = real_row * row_idxs.len() + real_col;
            data[idx] = *h;
        }
        IndexedGrid { row_idxs, col_idxs, data }
    }
}

#[cfg(test)]
mod test {
    use crate::{Flat2d, IndexedGrid};
    #[test]      
    fn create_igrid() {
        let igrid: IndexedGrid<f32> = IndexedGrid::from(
            vec![(0, 4, 27.5), (3, 4, 12.1), (10, 7, 44.6)], 0.
        );
        assert_eq!(igrid.get(0, 4), 27.5);
        assert_eq!(igrid.get(3, 4), 12.1);
        assert_eq!(igrid.get(10, 7), 44.6);
    }
}

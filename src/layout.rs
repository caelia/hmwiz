// #![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// #![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Index;

use rand::prelude::*;
use rand_distr::{Distribution, Pert};

use enterpolation::bezier::{Bezier, BezierBuilder, BezierError};
use enterpolation::linear::{Linear, LinearBuilder, LinearError};
use enterpolation::{DiscreteGenerator, Generator};

use crate::config::Config;
use crate::structures::{GridOrientation, Grid};

// to prevent endless loops when populating layout grid
const LAYOUT_MAX_TRIES: u8 = 10;



#[derive(Debug, Clone, PartialEq)]
pub enum Dir {
    H(bool),
    V(bool),
}

impl Dir {
    fn flip(&self) -> Self {
        match self {
            Dir::H(fwd) => Dir::H(!fwd),
            Dir::V(fwd) => Dir::V(!fwd),
        }
    }
}

#[derive(Debug)]
struct Layout {
    config: Config,
    active_rows: Vec<usize>,
    active_cols: Vec<usize>,
    hgrid: Grid<Option<f32>>,
    vgrid: Grid<Option<f32>>,
    merged: Grid<Option<f32>>,
}

impl Layout {
    pub fn new(config: Config) -> Self {
        let (rows, cols) = config.layout_dimensions();
        let (ridx0, ridxn, cidx0, cidxn) = config.active_limits();
        let hi_points = HashSet::new();
        let lo_points = HashSet::new();
        let trng = thread_rng();

        let active_rows = Vec::new();
        let active_cols= Vec::new();
        let excess_tries_msg = "\
            Exceeded maximum number of tries to set unique layout points.\n
            This is probably just a fluke, but if it happens repeatedly,\n
            please file a bug report.
            ";
        for _ in 0..config.n_hi {
            let row = trng.gen_range(ridx0..ridxn);
            let col = trng.gen_range(cidx0..cidxn);
            let tries = 1;
            loop {
                if hi_points.insert((row, col)) {
                    active_rows.push(row);
                    active_cols.push(col);
                    break;
                }
                tries += 1;
                if tries > LAYOUT_MAX_TRIES {
                    panic!("{}", excess_tries_msg);
                }
            }
        }
        for _ in 0..config.n_lo {
            let row = trng.gen_range(ridx0..ridxn);
            let col = trng.gen_range(cidx0..cidxn);
            let tries = 1;
            loop {
                if lo_points.insert((row, col)) {
                    active_rows.push(row);
                    active_cols.push(col);
                    break;
                }
                tries += 1;
                if tries > LAYOUT_MAX_TRIES {
                    panic!("{}", excess_tries_msg);
                }
            }
        }
        if !(hi_points.is_disjoint(&lo_points)) {
            panic!(
                "hi_points & lo_points set contain points in common.\n
                 This is probably just a fluke, but if it happens\n
                 repeatedly, please file a bug report."
            );
        }

        active_rows.sort();
        active_rows.dedup();
        active_cols.sort();
        active_cols.dedup();

        let all_points = Vec::new();
        for (r, c) in hi_points.iter() {
            let height = trng.gen_range(config.hi_min..=config.hi_max);
            all_points.push((*r, *c, height));
        }
        for (r, c) in lo_points.iter() {
            let height = trng.gen_range(config.lo_min..=config.lo_max);
            all_points.push((*r, *c, height));
        }
        // Add corners
        for (r, c) in [(0, 0), (0, cols - 1), (rows - 1, 0), (rows - 1, cols - 1)] {
            all_points.push((r, c, config.margin_height));
        }

        // Add edge points where needed
        for row in [0, rows - 1] {
            for col in active_cols {
                all_points.push((row, col, config.margin_height));
            }
        }
        for col in [0, cols - 1] {
            for row in active_rows {
                all_points.push((row, col, config.margin_height));
            }
        }

        let hgrid  = Grid::new(config.rows, config.cols, GridOrientation::RowMajor, None);
        let vgrid  = Grid::new(config.rows, config.cols, GridOrientation::ColumnMajor, None);
        let merged = Grid::new(config.rows, config.cols, GridOrientation::RowMajor, None);

        for (row, col, height) in all_points {
            hgrid.set(row, col, Some(height));
            vgrid.set(row, col, Some(height));
        }

        Layout {
            config,
            hgrid,
            vgrid,
            active_rows,
            active_cols,
            merged,
        }
    }

    fn get_hsegment(&self, row: usize, col: usize, dir: Dir) -> Vec<(usize, usize)> {
        let result = Vec::new();
        match dir {
            Dir::H(true) => {
                let start_idx = self.active_cols.binary_search(&col).unwrap();
                let (_, cols) = self.active_cols.split_at[start_idx];
                for c in cols {
                    result.push((row, c));
                }
                result
            },
            Dir::H(false) => {
                let start_idx = self.active_cols.binary_search(&col).unwrap();
                let (cols, _) = self.active_cols.split_at[start_idx];
                for c in cols.rev() {}
                    result.push(row, c);
                }
                result
            },
            _ => panic!("Invalid argument for Layout::get_hsegment: {?}", dir),
        }
    }
    fn get_vsegment(&self, row: usize, col: usize, dir: Dir) -> Vec<(usize, usize)> {
        match dir {
            Dir::V(true) => {},
            Dir::V(false) => {},
            _ => panic!("Invalid argument for Layout::get_hsegment: {?}", dir),
        }
    }
    fn get_next_defined_point(&self, row: usize, col: usize, dir: Dir) -> Option<(usize, usize, f32)> {
        let segment = 
    }

    pub fn set_crossings(&mut self) {
        let (row_idxs, col_idxs) = self.hgrid.indices();
        let rlimit = row_idxs.len() - 1;
        let climit = col_idxs.len() - 1;
        // Horizontal
        for row in &row_idxs[1..rlimit] {
            
        }
        // Vertical
        for col in &col_idxs[1..climit] {
            
        }
    }
}
// }

#[derive(Debug)]
struct SlicePoints {
    data: Vec<f32>,
}

impl Generator<usize> for SlicePoints {
    type Output = f32;
    fn gen(&self, idx: usize) -> f32 {
        self.data[idx]
    }
}

impl DiscreteGenerator for SlicePoints {
    fn len(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug)]
struct RefGrid {
    dir: Dir,
    slices: HashMap<usize, Vec<f32>>,
}

impl RefGrid {
    fn new(dir: Dir) -> Self {
        RefGrid {
            dir,
            slices: HashMap::new(),
        }
    }
    fn keys(&self) -> Vec<usize> {
        let kk: Vec<usize> = self.slices.keys().map(|k| *k).collect();
        kk.sort();
        kk
    }
    fn get(&self, row: usize, col: usize) -> f32 {
        match self.slices.get(&row) {
            Some(vec) => vec[col],
            None => panic!("Attempted to retrieve with nonexistent key."),
        }
    }
    fn add_row(&mut self, row: usize, values: Vec<f32>) {
        assert_eq!(self.slices.insert(row, values), None);
    }
}

#[derive(Debug)]
struct Map {
    rows: usize,
    cols: usize,
    data: Vec<f32>,
}

impl Flat2d for Map {
    type DataPoint = f32;
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn data(&self) -> Vec<f32> {
        self.data
    }
}

impl Map {
    fn new(rows: usize, cols: usize) -> Self {
        let data = vec![0.; rows * cols];
        Map { rows, cols, data }
    }
}

#[derive(Debug)]
struct MapSystem {
    config: Config,
    layout: Layout,
    ref_h: RefGrid,
    ref_v: RefGrid,
    hslices: Map,
    vslices: Map,
}

impl MapSystem {
    fn new(config: Config) -> Self {
        let layout = Layout::new();
        let ref_h = RefGrid::new(Dir::H);
        let ref_v = RefGrid::new(Dir::V);
        let (h, w) = config.size;
        let hslices = Map::new(h, w);
        let vslices = Map::new(w, h);
        MapSystem {
            config,
            layout,
            ref_h,
            ref_v,
            hslices,
            vslices,
        }
    }
}

fn main() {
    println!("This is just a gotdan placeholder.");
}

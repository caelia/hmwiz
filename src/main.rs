// #![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// #![allow(dead_code)]

mod layout;
mod system;
mod structures;

use std::ops::Index;
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;

use rand::prelude::*;
use rand_distr::{Pert, Distribution};

use enterpolation::linear::{Linear, LinearBuilder, LinearError};
use enterpolation::bezier::{Bezier, BezierBuilder, BezierError};
use enterpolation::{Generator, DiscreteGenerator};

use image::{GrayImage, GenericImage, ImageBuffer, Luma};

#[derive(Debug, Clone, Copy)]
struct Config {
    pub max_height: f32,
    pub min_height: f32,
    pub max_slope: f32,
    pub rows: usize,
    pub cols: usize,
    pub margin_width: usize,
    pub margin_height: f32,
    pub pad_width: usize,
    pub n_hi: usize,
    pub hi_min: f32,
    pub hi_max: f32,
    pub n_lo: usize,
    pub lo_min: f32,
    pub lo_max: f32,
}

impl Config {
    pub fn default(
        rows: usize,
        cols: usize,
        n_hi: usize,
        hi_min: f32,
        hi_max: Option<f32>,
        n_lo: usize,
        lo_min: Option<f32>,
        lo_max: f32,
    ) -> Self {
        Config {
            max_height: 255.,
            min_height: 0.,
            max_slope: 1.,
            rows,
            cols,
            margin_width: 16,
            margin_height: 0.,
            pad_width: 4,
            n_hi,
            hi_min,
            hi_max: hi_max.unwrap_or(255.),
            n_lo,
            lo_min: lo_min.unwrap_or(0.),
            lo_max,
        }
    }

    // Gives layout size, which is total dimensions - margins
    // We add 2 because the edges need to overlap the inner edge
    // of the margin.
    pub fn layout_dimensions(&self) -> (usize, usize) {
        (
            self.rows - self.margin_width * 2 + 2,
            self.cols - self.margin_width * 2 + 2,
        )
    }

    // Gives minimum and maximum indices where non-edge points
    // can be set in the layout grid.
    pub fn active_limits(&self) -> (usize, usize, usize, usize) {
        let pw = self.pad_width;
        let (r, c) = self.layout_dimensions();
        (pw, r - pw, pw, c - pw)
    }
}

#[derive(Debug, Clone)]
pub struct IndexedGrid<T> {
    row_indices: Vec<usize>,
    col_indices: Vec<usize>,
    data: Vec<T>,
}


impl<T> IndexedGrid<T> {
    pub fn new(points: Vec<(usize, usize, T)>, default: T) -> Self {
        let row_indices = Vec::new();
        let col_indices = Vec::new();
        for (r, c, _) in points {
            row_indices.push(r);
            col_indices.push(c);
        }
        let cap = row_indices.len() * col_indices.len();
        let data = vec![default; cap];
        row_indices.sort();
        row_indices.dedup();
        col_indices.sort();
        col_indices.dedup();
        IndexedGrid { row_indices, col_indices, data }
    }

    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[self.get_index(row, col)]
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[self.get_index(row, col)] = value;
    }

    pub fn rows(&self) -> usize {
        self.row_indices.len()
    }

    pub fn cols(&self) -> usize {
        self.col_indices.len()
    }

    pub fn set_edges(&mut self, value: T) {
        for row in [0, self.rows() - 1] {
            let offset = row * self.cols();
            for col in 0..self.cols() {
                self.data[offset + col] = value;
            }
        }
        for col in [0, self.cols() - 1] {
            for row in 1..(self.rows() - 1) {
                let idx = row * self.cols() + col;
                self.data[idx] = value;
            }
        }
    }

    pub fn get_slices(&self, dir: Dir) -> Vec<(usize, Vec<(usize, T)>)> {
        let result = Vec::new();        
        match dir {
            Dir::H(_) => {
                for r in self.row_indices {
                    let row_data = Vec::new();
                    for c in self.col_indices {
                        row_data.push((c, self.get(r, c)));
                    }
                    result.push((r, row_data));
                }
            },
            Dir::V(_) => {
                for c in self.col_indices {
                    let col_data = Vec::new();
                    for r in self.row_indices {
                        col_data.push((r, self.get(r, c)));
                    }
                    result.push((c, col_data));
                }
            },
        }
        result
    }

    // Private functions
    fn get_index(&self, row: usize, col: usize) -> usize {
        let ridx = match self.row_indices.binary_search(&row) {
            Some(r) => r,
            None => panic!("Invalid row index for IndexedGrid: {}", row),
        };
        let cidx = match self.col_indices.binary_search(&col) {
            Some(c) => c,
            None => panic!("Invalid col index for IndexedGrid: {}", col),
        };
        ridx * self.col_indices.len() + cidx
    }
}


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


pub fn guide_points(config: Config) -> IndexedGrid<Option<f32>> {
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

    // Check consistency of data
    for points in [hi_points, lo_points] {
        for (row, col) in points {
            if !active_rows.contains(&row) {
                panic!("row index {} not found in active_rows", row);
            }
            if !active_cols.contains(&col) {
                panic!("col index {} not found in active_cols", col);
            }
        }
    }
    
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
    for (r, c) in [(0, 0), (0, config.cols - 1), (config.rows - 1, 0), (config.rows - 1, config.cols - 1)] {
        all_points.push((r, c, config.margin_height));
    }

    let initial_grid = Grid::new(all_points, None);

    for (row, col, height) in all_points {
        initial_grid.set(row, col, Some(height));
    }

    initial_grid.set_edges(Some(config.margin_height));

    initial_grid
}

fn fill_slice(slice: Vec<(usize, Option<f32>)>) -> Vec<(usize, f32)> {
    let mut result = Vec::new();
    let mut slope: f32;
    let mut pos = 0;
    let limit = slice.len();
    while pos < limit {
        match slice[&pos] {
            (start_loc, Some(start_height)) => {
                let mut segment_end: usize;
                for i in pos..limit {
                    match slice[&i] {
                        (loc, Some(height)) => {
                            let hdiff = height - start_height;
                            let distance = loc as f32 - start_loc as f32;
                            assert_ne!(distance, 0.);
                            slope = hdiff / distance;
                            segment_end = i;
                            break
                        },
                        _ => (),
                    }
                }
                for i in pos..segment_end {
                    let loc, height = match slice[&i] {
                        (l, Some(h)) => (l, h),
                        (l, None) => {
                            let distance = l - start_loc;    
                            let h = start_height + distance * slope;
                            (l, h)
                        },
                    }
                    result.push((loc, height));
                }
                pos = segment_end;
            },
            _ => panic!("wrong start position while filling slice: {}", pos),
        }
    }
    match slice.last() {
        (loc, Some(height)) => result.push((loc, height)),
        _ => panic!("invalid slice passed to fill_slice()"),
    }
    result
}

fn layout_slices(grid: IndexedGrid<Option<f32>>)
        -> (Vec<(usize, Vec<(usize, f32)>)>,
            Vec<(usize, Vec<(usize, f32)>)>) {
    let hslices0 = grid.get_slices(Dir::H(true));
    let vslices0 = grid.get_slices(Dir::V(true));
    let mut hslices = Vec::new();
    let mut vslices = Vec::new();

    for (row, slice) in hslices0 {
        hslices.push((row, fill_slice(slice)));
    }
    for (col, slice) in vslices0 {
        vslices.push((col, fill_slice(slice)));
    }

    (hslices, vslices)    
}

/*
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
*/

/*
#[derive(Debug)]
struct RefGrid {
    dir: Dir,
    slices: HashMap<usize, Vec<f32>>,
}

impl RefGrid {
    fn new(dir: Dir) -> Self {
        RefGrid { dir, slices: HashMap::new() }
    }
    fn keys(&self) -> Vec<usize> {
        let kk: Vec<usize> = self.slices.keys().map(|k| *k).collect();
        kk.sort();
        kk
    }
    fn get(&self, row: usize, col: usize) -> f32 {
        match self.slices.get(&row) {
            Some(vec) => vec[col],
            None => panic!("Attempted to retrieve with nonexistent key.")
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
    fn rows(&self) -> usize { self.rows }
    fn cols(&self) -> usize { self.cols }
    fn data(&self) -> Vec<f32> { self.data }
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
        MapSystem { config, layout, ref_h, ref_v, hslices, vslices }
    }

}

fn main() {
    println!("This is just a gotdan placeholder.");
}
*/

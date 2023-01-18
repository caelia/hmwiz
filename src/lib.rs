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

use image::{GrayImage, GenericImage, ImageBuffer, Luma};

/*
const GLOBAL_MIN: f32 = 0.;
const GLOBAL_MAX: f32 = 376.;
// const GLOBAL_MAX: f32 = 313.;
// const GLOBAL_MAX: f32 = 255.;
const MAX_SLOPE: f32 = 1.;
*/

#[derive(Debug)]
enum Dir {
    H,
    V
}

#[derive(Debug, Clone, Copy)]
struct Config {
    max_height: f32,
    min_height: f32,    
    max_slope: f32,
    size: (usize, usize),
    margin_width: usize,
    margin_height: f32,
    n_hi: usize,
    hi_min: f32,
    hi_max: f32,
    n_lo: usize,
    lo_min: f32,
    lo_max: f32,
}

impl Config {
    fn default(width: usize, height: usize,
               n_hi: usize, hi_min: f32, hi_max: Option<f32>,
               n_lo: usize, lo_min: Option<f32>, lo_max: f32)
                -> Self {
        Config {
            max_height: 255.,
            min_height: 0.,    
            max_slope: 1.,
            size: (height, width),
            margin_width: 16,
            margin_height: 0.,
            n_hi,
            hi_min,
            hi_max: hi_max.unwrap_or(255.),
            n_lo,
            lo_min: lo_min.unwrap_or(0.),
            lo_max,
        }
    }
}

trait Flat2d {
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
struct IndexedGrid<T> {
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
    fn from(mut points: Vec<(usize, usize, T)>, default: T) -> Self {
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

#[derive(Debug)]
struct SampledSlices {
    rows: usize,
    cols: usize,
    data: HashMap<usize, HashMap<usize, Option<f32>>>,
}

#[derive(Debug)]
struct CompleteSlices {
    rows: usize,
    cols: usize,
    data: Vec<Vec<f32>>,
}

impl Flat2d for SampledSlices {
    type DataPoint = Option<f32>;
    fn rows(&self) -> usize { self.rows }
    fn cols(&self) -> usize { self.cols }
    fn data(&self) -> HashMap<usize, HashMap<usize, Option<f32>>> { self.data }
}

impl Flat2d for CompleteSlices {
    type DataPoint = f32;
    fn rows(&self) -> usize { self.rows }
    fn cols(&self) -> usize { self.cols }
    fn data(&self) -> Vec<Vec<f32>> { self.data }
}
// to prevent endless loops when populating layout grid
const LAYOUT_MAX_TRIES: u8 = 10;

#[derive(Debug)]
struct Layout {
    rows: usize,
    cols: usize,
    guidepoints: Vec<Option<f32>>,
    hslices: HashMap<usize, Vec<f32>>,
    vslices: HashMap<usize, Vec<f32>>,
    grid: Vec<(usize, f32)>,
}

impl Flat2d for Layout {
    type DataPoint = (usize, f32);
    fn rows(&self) -> usize { self.rows }
    fn cols(&self) -> usize { self.cols }
    fn data(&self) -> Vec<(usize, f32)> { self.grid }
}

impl Layout {
    fn new() -> Self {
        Layout {
            rows: 0,
            cols: 0,
            guidepoints: Vec::new(),
            hslices: HashMap::new(),
            vslices: HashMap::new(),
            grid: Vec::new()
        }
    }
    fn set_guide_points(&mut self, config: Config) {
        let hi_points = HashSet::new();  
        let lo_points = HashSet::new();  
        let trng = thread_rng();

        let excess_tries_msg = "
            Exceeded maximum number of tries to set unique layout points.\n
            This is probably just a fluke, but if it happens repeatedly,\n
            please file a bug report.
        ";
        for _ in 0..config.n_hi {
            let row = trng.gen_range(0..self.rows);
            let col = trng.gen_range(0..self.cols);
            let tries = 1;
            loop {
                if hi_points.insert((row, col)) {
                    break;
                }
                tries += 1;
                if tries > LAYOUT_MAX_TRIES {
                    panic!("{}", excess_tries_msg);
                }
            }
        }
        for _ in 0..config.n_lo {
            let row = trng.gen_range(0..self.rows);
            let col = trng.gen_range(0..self.cols);
            let tries = 1;
            loop {
                if lo_points.insert((row, col)) {
                    break;
                }
                tries += 1;
                if tries > LAYOUT_MAX_TRIES {
                    panic!("{}", excess_tries_msg);
                }
            }
        }
        if !(hi_points.is_disjoint(&lo_points)) {
            panic!("hi_points & lo_points set contain points in common.\n
                    This is probably just a fluke, but if it happens\n
                    repeatedly, please file a bug report.");
        }
        
        for loc in hi_points.iter() {
            let height = trng.gen_range(config.hi_min..=config.hi_max);
            self.insert(loc, height);
        }
        for loc in lo_points.iter() {
            let height = trng.gen_range(config.lo_min..=config.lo_max);
            self.insert(loc, height);
        }
    }
    fn reconcile(&mut self) {
        
    }
}

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

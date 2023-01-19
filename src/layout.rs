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
use crate::structures::{Dir, Flat2d, IndexedGrid};

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
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn data(&self) -> HashMap<usize, HashMap<usize, Option<f32>>> {
        self.data
    }
}

impl Flat2d for CompleteSlices {
    type DataPoint = f32;
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn data(&self) -> Vec<Vec<f32>> {
        self.data
    }
}
// to prevent endless loops when populating layout grid
const LAYOUT_MAX_TRIES: u8 = 10;

/*
#[derive(Debug)]
struct Layout {
    rows: usize,
    cols: usize,
    guidepoints: Vec<Option<f32>>,
    hslices: HashMap<usize, Vec<f32>>,
    vslices: HashMap<usize, Vec<f32>>,
    grid: Vec<(usize, f32)>,
}
*/
/*
impl Flat2d for Layout {
    type DataPoint = (usize, f32);
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn data(&self) -> Vec<(usize, f32)> {
        self.grid
    }
}
*/

/*
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
*/

// Meaningless comment - because my branch was inconsistent due to massive clock skew

#[derive(Debug)]
struct Layout {
    config: Config,
    hgrid: IndexedGrid<Option<f32>>,
    vgrid: IndexedGrid<Option<f32>>,
    fynal: IndexedGrid<Option<f32>>,
}

impl Layout {
    pub fn new(config: Config) -> Self {
        let (rows, cols) = config.layout_dimensions();
        let (ridx0, ridxn, cidx0, cidxn) = config.active_limits();
        let hi_points = HashSet::new();
        let lo_points = HashSet::new();
        let trng = thread_rng();

        let excess_tries_msg = "
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

        let all_points = Vec::new();
        for (r, c) in hi_points.iter() {
            let height = trng.gen_range(config.hi_min..=config.hi_max);
            all_points.push((*r, *c, Some(height)));
        }
        for (r, c) in lo_points.iter() {
            let height = trng.gen_range(config.lo_min..=config.lo_max);
            all_points.push((*r, *c, Some(height)));
        }
        // Add corners
        for (r, c) in [(0, 0), (0, cols - 1), (rows - 1, 0), (rows - 1, cols - 1)] {
            all_points.push((r, c, Some(config.margin_height)));
        }

        let hgrid = IndexedGrid::from(all_points, None);
        // Set edge points to margin height
        for row in [0, rows - 1] {
            hgrid.setup_iteration(Dir::H, None);
            for col in hgrid {
                hgrid.set(row, col, Some(0.));
            }
        }
        for col in [0, cols - 1] {
            hgrid.setup_iteration(Dir::V, None);
            for row in hgrid {
                hgrid.set(row, col, Some(0.));
            }
        }

        let vgrid = hgrid.clone();
        let fynal = hgrid.clone_blank();

        Layout {
            config,
            hgrid,
            vgrid,
            fynal,
        }
    }

    pub fn set_crossings(&mut self) {}
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

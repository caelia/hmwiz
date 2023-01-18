// #![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// #![allow(dead_code)]

mod layout;
mod config;
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

use config::Config;
use structures::{Dir, Flat2d};


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

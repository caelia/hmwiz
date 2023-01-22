#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod config;
mod crawler;
mod grid;
mod envelope;
mod layout;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Index;

use rand::prelude::*;
use rand_distr::{Distribution, Pert};

use enterpolation::bezier::{Bezier, BezierBuilder, BezierError};
use enterpolation::linear::{Linear, LinearBuilder, LinearError};
use enterpolation::{DiscreteGenerator, Generator};

use image::{GenericImage, GrayImage, ImageBuffer, Luma};

use config::Config;
use crawler::{GridCrawler, GridCrawlerArray};
use grid::{Grid, Orientation};
use layout::Point;

fn main() {
    let rows = 4096;
    let cols = 4096;
    let default: Point<f32> = Point::Empty;
    let grid = Grid::new(rows, cols, Orientation::RowMajor, default);
    println!("It's a grid!! {:?}", grid);
}

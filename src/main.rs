#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod config;
mod crawler;
mod grid;

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

fn main() {
    println!("HMWIZ FTW!!!");
}

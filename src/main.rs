#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod grid;
mod common;

use rand_distr;
use grid::{Node, Grid};
use common::*;

fn main() {
    let mut grid = Grid::new(1024, 1024, 25., None);
    grid.set_seeds(20.);
    grid.show_seeds();
}

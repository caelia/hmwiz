#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod grid;
mod common;

use rand_distr;
use grid::{Pt, Grid};
use common::*;

fn main() {
    let mut grid: Grid<DEFAULT_WIDTH, DEFAULT_HEIGHT> = Grid::default();
    grid.set_seeds();
}

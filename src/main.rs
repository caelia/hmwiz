#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod grid;
mod common;
mod util;
mod generate;

use rand_distr;
use grid::{Node, Grid};
use common::*;
use generate::generate_x4;

fn main() {
    let mut grid = Grid::new(1024, 1024, 25., None);
    grid.set_anchors();
    // grid.set_seeds();
    let (seed_rows, seed_cols) = grid.set_seeds();
    // grid.clone().dump();
    // grid.show_seeds();
    grid.adjust_seed_heights(seed_rows, seed_cols);
    // grid.show_seeds();
    generate_x4(grid, "test.png");
    // grid.dump();
}

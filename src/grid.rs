#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand_distr::{Pert, PertBuilder, Distribution};
use rand::RngExt;
use crate::common::*;

#[derive(Debug, Clone)]
pub enum Pt {
    Fixed(f32),
    Seed(f32),
    Resolved(f32),
    Proposed(f32),
    Empty,
}

impl Copy for Pt {}

#[derive(Debug, Clone)]
pub struct Grid<const W: usize, const H: usize> {
    data: Vec<Vec<Pt>>
}

impl<const W: usize, const H: usize> Grid<W, H> {
    pub fn set_seeds(&mut self) {
        let max_depth = -(MAX_ELEVATION / ((100. / WATER_PCT) - 1.));
        println!("max_depth: {}", max_depth);
        let mut seed_locations: Vec<(usize, usize)> = vec![];
        let (hmin, hmax) = (DEFAULT_MARGIN, DEFAULT_WIDTH - DEFAULT_MARGIN);
        let (vmin, vmax) = (DEFAULT_MARGIN, DEFAULT_HEIGHT - DEFAULT_MARGIN);
        let mut rng = rand::rng();
        while seed_locations.len() < DEFAULT_SEEDS {
            let col: usize = rng.random_range(hmin..hmax);
            let row: usize = rng.random_range(vmin..vmax);
            if !seed_locations.contains(&(col, row)) {
                seed_locations.push((col, row));
            }
        }

        let pert = Pert::new(max_depth, MAX_ELEVATION)
            .with_mode(DEFAULT_MEAN_ELEVATION)
            .unwrap();
        for (col, row) in seed_locations {
            let height = pert.sample(&mut rng);
            self.data[row][col] = Pt::Seed(height);
        }
    }
}
impl<const W: usize, const H: usize> Default for Grid<W, H> {
    fn default() -> Self {
        let data = (0..H).into_iter().map(|_| {
            Vec::from([Pt::Empty; W])
        }).collect();
        Self { data }
    }
}


#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand_distr::{Pert, PertBuilder, Distribution};
use rand::RngExt;
use crate::common::*;

#[derive(Debug, Clone)]
enum Dir {
    N,
    E,
    S,
    W
}

#[derive(Debug, Clone)]
pub struct NodeData {
    elevation: f32,
    roughness: f32,
    gravity: f32,
    strength: f32,
}

impl NodeData {
    fn random_nd(
        (elev_min, elev_max, elev_mode): (f32, f32, f32),
        (ruf_min, ruf_max, ruf_mode): (f32, f32, f32),
        (grav_min, grav_max, grav_mode): (f32, f32, f32),
        (strn_min, strn_max, strn_mode): (f32, f32, f32),
    ) -> NodeData {
        let mut rng = rand::rng();
        let elevation = {
            let pert = Pert::new(elev_min, elev_max)
                .with_shape(0.7)
                .with_mode(elev_mode)
                .unwrap();
            pert.sample(&mut rng)
        };
        let roughness = {
            let pert = Pert::new(ruf_min, ruf_max)
                .with_mode(ruf_mode)
                .unwrap();
            pert.sample(&mut rng)
        };
        let gravity = {
            let pert = Pert::new(grav_min, grav_max)
                .with_shape(10.)
                .with_mode(grav_mode)
                .unwrap();
            pert.sample(&mut rng)
        };
        let strength = {
            let pert = Pert::new(strn_min, strn_max)
                .with_mode(strn_mode)
                .unwrap();
            pert.sample(&mut rng)
        };
        NodeData { elevation, roughness, gravity, strength }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Anchor(NodeData),
    Seed(NodeData),
    Proposed(NodeData),
    Final(f32),
    Undefined,
    Null,
}

#[derive(Debug, Clone)]
pub struct Settings {
    max_height: f32,
    max_depth: f32,
    max_slope: f32,
    roughness: f32,
    gravity: f32,
    strength: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            max_height: 255.,
            max_depth: 0.,
            max_slope: 1.0,
            roughness: 0.5,
            gravity: 0.1,
            strength: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    data: Vec<Vec<Node>>,
    globals: Settings,
    margin: usize,
}

impl Grid {
    pub fn new(
        rows: usize,
        cols: usize,
        water_pct: f32,
        margin: Option<usize>,
    ) -> Self {
        assert!(water_pct >= 0. && water_pct <= 100.);
        let data = (0..rows).into_iter().map(|_| {
            (0..cols).into_iter().map(|_| {
                Node::Undefined
            }).collect()
        }).collect();
        let mut globals = Settings::default();
        let max_depth = -((water_pct / (100. - water_pct)) * globals.max_height);
        println!("MAX_DEPTH: {max_depth}");
        globals.max_depth = max_depth;
        let margin = margin.unwrap_or(8);

        Grid { data, globals, margin }
    }

    pub fn set_seeds(&mut self, mean_elevation: f32) {
        let mut seed_locations: Vec<(usize, usize)> = vec![];
        let height = self.data.len();
        let width = match height {
            0 => 0,
            _ => self.data[0].len(),
        };
        
        let (hmin, hmax) = (self.margin, width - self.margin);
        let (vmin, vmax) = (self.margin, height - self.margin);
        let mut rng = rand::rng();
        while seed_locations.len() < DEFAULT_SEEDS {
            let col: usize = rng.random_range(hmin..hmax);
            let row: usize = rng.random_range(vmin..vmax);
            if !seed_locations.contains(&(col, row)) {
                seed_locations.push((col, row));
            }
        }

        let pert = Pert::new(self.globals.max_depth, self.globals.max_height)
            .with_mode(mean_elevation)
            .unwrap();
        for (col, row) in seed_locations {
            let node_data = NodeData::random_nd(
                (self.globals.max_depth, self.globals.max_height, mean_elevation),
                (0., 1., self.globals.roughness),
                (-1., 1., self.globals.gravity),
                (0., 1., self.globals.strength)
            );
            self.data[row][col] = Node::Seed(node_data);
        }
    }

    pub fn show_seeds(&self) {
        self.data.iter().enumerate().for_each(|(r, row)| {
            row.iter().enumerate().for_each(|(c, node)| {
                match node {
                    Node::Seed(ndata) => {
                        println!("{:>4}, {:>4}: {}", r, c, ndata.elevation);
                    }
                    _ => (),
                }
            })
        })
    }
}

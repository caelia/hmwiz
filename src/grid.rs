#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand_distr::{Pert, PertBuilder, Distribution};
use rand::RngExt;
// use crate::common::*;
use crate::util::{wmean, distance, gravitate};

#[derive(Debug, Clone)]
pub enum Dir {
    N,
    E,
    S,
    W
}

impl Copy for Dir {}

impl Dir {
    fn flip(&self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SlopeChange {
    Continue,
    Reflect,
    Flatten,
}


#[derive(Debug, Clone, PartialEq)]
pub struct NodeData {
    pub elevation: f32,
    pub roughness: f32,
    pub gravity: f32,
    pub strength: f32,
}

impl Copy for NodeData {}

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

    fn anchor_data(elevation: f32, roughness: f32, gravity: f32, strength: f32) -> Self {
        NodeData { elevation, roughness, gravity, strength }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Anchor(NodeData),
    Seed(NodeData),
    Proposed(NodeData),
    Final(f32),
    Undefined,
    Null,
}

impl Copy for Node {}

impl Node {
    pub fn elevation(&self) -> Option<f32> {
        match self.data() {
            Some(ndata) => Some(ndata.elevation),
            None => {
                match self {
                    Node::Final(elev) => Some(*elev),
                    _ => None,
                }
            }
        }
    }

    pub fn gravity(&self) -> Option<f32> {
        match self.data() {
            Some(ndata) => Some(ndata.gravity),
            None => None,
        }
    }

    pub fn roughness(&self) -> Option<f32> {
        match self.data() {
            Some(ndata) => Some(ndata.roughness),
            None => None,
        }
    }

    pub fn strength(&self) -> Option<f32> {
        match self.data() {
            Some(ndata) => Some(ndata.strength),
            None => None,
        }
    }

    fn data(&self) -> Option<NodeData> {
        match self {
            Node::Anchor(data) | Node::Seed(data) | Node::Proposed(data) => Some(*data),
            _ => None,
        }
    }

    pub fn set_elevation(&mut self, new_elev: f32) -> bool {
        let mut msg = String::new();
        match self {
            Node::Anchor(data) | Node::Seed(data) | Node::Proposed(data) => {
                // println!("[{}] -> [{}]", data.elevation, new_elev);
                data.elevation = new_elev;
                true
            },
            Node::Final(elev) => {
                // println!("[{}] -> [{}]", elev, new_elev);
                *elev = new_elev;
                true
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    max_height: f32,
    max_depth: f32,
    mean_elev: f32,
    max_slope: f32,
    roughness: f32,
    gravity: f32,
    strength: f32,
    flattening: f32,
    seeds: usize,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            max_height: 255.,
            max_depth: 0.,
            mean_elev: 20.,
            max_slope: 1.0,
            roughness: 0.5,
            gravity: 0.1,
            strength: 0.5,
            flattening: 0.25,
            seeds: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
        // println!("MAX_DEPTH: {max_depth}");
        globals.max_depth = max_depth;
        let margin = margin.unwrap_or(8);

        Grid { data, globals, margin }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        let rows = self.data.len();
        if rows == 0 {
            (0, 0)
        } else {
            (rows, self.data[0].len())
        }
    }

    pub fn elevation_at(&self, col: usize, row: usize) -> Option<f32> {
        match self.node_at(col, row) {
            Some(node) => node.elevation(),
            None => None,
        }
    }

    pub fn dump(&self) {
        let (rows, cols) = self.dimensions();
        for r in 0..rows {
            println!(":: {:04} ::::::::::::::::", r);
            for c in 0..cols {
                match self.node_at(c, r) {
                    Some(node) => println!("<{:>3}|{:>3}> {:?}\n------------------------", c, r, node),
                    None => println!("[NONE]\n------------------------"),
                }
            }
        }
    }

    pub fn set_anchors(&mut self) {
        let (rows, cols) = self.dimensions();
        let margin = self.margin;
        let row = margin;        
        // let depth = self.globals.max_depth / 2.;
        let depth = self.globals.max_depth / 4.;
        (margin..(cols - margin)).into_iter().for_each(|c| {
            self.set_node_at(c, row, Node::Anchor(NodeData::anchor_data(depth, 0., 0., 0.)));
            // println!("[{}][{}]", c, row);
        });
        let column = margin;
        (margin + 1..(rows - margin - 1)).into_iter().for_each(|r| {
            self.set_node_at(margin, r, Node::Anchor(NodeData::anchor_data(depth, 0., 0., 0.)));
            // println!("[{}][{}]", margin, r);
            self.set_node_at(cols - margin - 1, r, Node::Anchor(NodeData::anchor_data(depth, 0., 0., 0.)));
            // println!("[{}][{}]", cols - margin - 1, r);
        });
        let row = rows - margin - 1;
        (margin..(cols - margin)).into_iter().for_each(|c| {
            self.set_node_at(c, row, Node::Anchor(NodeData::anchor_data(depth, 0., 0., 0.)));
            // println!("[{}][{}]", c, row);
        });
    }

    pub fn set_seeds(&mut self) -> (Vec<usize>, Vec<usize>){
        let mut seed_locations: Vec<(usize, usize)> = vec![];
        let height = self.data.len();
        let width = match height {
            0 => 0,
            _ => self.data[0].len(),
        };
        
        let padding = (self.margin * 2) + 1;
        let (hmin, hmax) = (padding, width - padding);
        let (vmin, vmax) = (padding, height - padding);
        let mut rng = rand::rng();
        while seed_locations.len() < self.globals.seeds {
            let col: usize = rng.random_range(hmin..hmax);
            let row: usize = rng.random_range(vmin..vmax);
            if !seed_locations.contains(&(col, row)) {
                // println!("[{:>3}][{:>3}]", col, row);
                seed_locations.push((col, row));
            }
        }

        let mut seed_rows = vec![];
        let mut seed_cols = vec![];
        for (col, row) in seed_locations {
            if !seed_rows.contains(&row) {
                seed_rows.push(row);
            }
            if !seed_cols.contains(&col) {
                seed_cols.push(col);
            }
            let node_data = NodeData::random_nd(
                (self.globals.max_depth, self.globals.max_height, self.globals.mean_elev),
                (0., 1., self.globals.roughness),
                (-1., 1., self.globals.gravity),
                (0., 1., self.globals.strength)
            );
            self.data[row][col] = Node::Seed(node_data);
        }
        (seed_rows, seed_cols)
    }

    pub fn adjust_seed_heights(&mut self, seed_rows: Vec<usize>, seed_cols: Vec<usize>) {
        let (rows, cols) = self.dimensions();
        let min_col = self.margin;
        let max_col = cols - self.margin;
        let min_row = self.margin;
        let max_row = rows - self.margin;
        assert!(max_col > min_col);
        assert!(max_row > min_row);

        for row in seed_rows {
            let mut seed_found = false;
            // Forward
            let mut col1 = min_col;
            // println!("{col1} >>>");
            if let Some(node @ Node::Anchor(_)) = self.node_at(col1, row) {
                let mut done = false;
                let mut node_here = node;
                while !done {
                    match self.next_node(col1, row, Dir::E) {
                        Some((col2, _, mut node_ahead)) => {
                            match node_ahead {
                                Node::Anchor(_) => {
                                    done = true;
                                },
                                Node::Seed(_) => {
                                    seed_found = true;
                                    let dist = distance(col1, row, col2, row);
                                    let elev_here = node_here.elevation().unwrap();
                                    let elev_ahead = node_ahead.elevation().unwrap();
                                    let ediff = elev_ahead - elev_here;
                                    let slope = ediff / dist as f32;
                                    if slope > 1.0 {
                                        node_ahead.set_elevation(
                                            elev_here + (dist as f32)
                                        );
                                        self.set_node_at(col2, row, node_ahead);
                                    } else if slope < -1.0 {
                                        node_ahead.set_elevation(
                                            elev_here - dist as f32
                                        );
                                        self.set_node_at(col2, row, node_ahead);
                                    }
                                },
                                _ => panic!("Wrong node type found at {}, {}.", col1, row),
                            }
                            col1 = col2;
                            node_here = node_ahead;
                        },
                        None => panic!("Expected Anchor node to end traversal. (r{row}f)")
                    }
                }
            } else {
                panic!("Expected Anchor node at {min_col}, {row}.");
            }

            // Reverse
            let mut col1 = max_col - 1;
            // println!("<<< {col1}");
            if let Some(node @ Node::Anchor(_)) = self.node_at(col1, row) {
                let mut done = false;
                let mut node_here = node;
                while !done {
                    match self.next_node(col1, row, Dir::W) {
                        Some((col2, _, mut node_ahead)) => {
                            match node_ahead {
                                Node::Anchor(_) => done = true,
                                Node::Seed(_) => {
                                    // println!("Reverse traverse to [SEED]");
                                    seed_found = true;
                                    let dist = distance(col1, row, col2, row);
                                    let elev_here = node_here.elevation().unwrap();
                                    let elev_ahead = node_ahead.elevation().unwrap();
                                    let ediff = elev_ahead - elev_here;
                                    let slope = ediff / dist as f32;
                                    if slope > 1.0 {
                                        node_ahead.set_elevation(
                                            elev_here + (dist as f32)
                                        );
                                        self.set_node_at(col2, row, node_ahead);
                                    } else if slope < -1.0 {
                                        node_ahead.set_elevation(
                                            elev_here - dist as f32
                                        );
                                        self.set_node_at(col2, row, node_ahead);
                                    }
                                },
                                _ => panic!("Wrong node type found at {}, {}.", col1, row),
                            }
                            col1 = col2;
                            node_here = node_ahead;
                        },
                        None => panic!("Expected Anchor node to end traversal. (r{row}r)")
                    }
                }
            } else {
                panic!("Expected Anchor node at {col1}, {row}.");
            }

            if !seed_found {
                panic!("No Seed node found in row {row}")
            }
        }

        for col in seed_cols {
            let mut seed_found = false;
            // Forward
            // println!("VVV");
            let mut row1 = min_row;
            if let Some(node @ Node::Anchor(_)) = self.node_at(col, min_row) {
                let mut done = false;
                let mut node_here = node;
                while !done {
                    match self.next_node(col, row1, Dir::S) {
                        Some((_, row2, mut node_ahead)) => {
                            match node_ahead {
                                Node::Anchor(_) => done = true,
                                Node::Seed(_) => {
                                    seed_found = true;
                                    let dist = distance(col, row1, col, row2);
                                    let elev_here = node_here.elevation().unwrap();
                                    let elev_ahead = node_ahead.elevation().unwrap();
                                    let ediff = elev_ahead - elev_here;
                                    let slope = ediff / dist as f32;
                                    if slope > 1.0 {
                                        node_ahead.set_elevation(
                                            elev_here + (dist as f32)
                                        );
                                        self.set_node_at(col, row2, node_ahead);
                                    } else if slope < -1.0 {
                                        node_ahead.set_elevation(
                                            elev_here - dist as f32
                                        );
                                        self.set_node_at(col, row2, node_ahead);
                                    }
                                },
                                _ => panic!("Wrong node type found at {}, {}.", col, row1),
                            }
                            row1 = row2;
                            node_here = node_ahead;
                        },
                        None => panic!("Expected Anchor node to end traversal. (c{col}f)")
                    }
                }
            } else {
                panic!("Expected Anchor node at {col}, {min_row}.");
            }

            // Reverse
            // println!("^^^");
            let mut row1 = max_row - 1;
            if let Some(node @ Node::Anchor(_)) = self.node_at(col, row1) {
                let mut done = false;
                let mut row1 = max_row;
                let mut node_here = node;
                while !done {
                    match self.next_node(col, row1, Dir::N) {
                        Some((_, row2, mut node_ahead)) => {
                            match node_ahead {
                                Node::Anchor(_) => done = true,
                                Node::Seed(_) => {
                                    seed_found = true;
                                    let dist = distance(col, row1, col, row2);
                                    let elev_here = node_here.elevation().unwrap();
                                    let elev_ahead = node_ahead.elevation().unwrap();
                                    let ediff = elev_ahead - elev_here;
                                    let slope = ediff / dist as f32;
                                    if slope > 1.0 {
                                        node_ahead.set_elevation(
                                            elev_here + (dist as f32)
                                        );
                                        self.set_node_at(col, row2, node_ahead);
                                    } else if slope < -1.0 {
                                        node_ahead.set_elevation(
                                            elev_here - dist as f32
                                        );
                                        self.set_node_at(col, row2, node_ahead);
                                    }
                                },
                                _ => panic!("Wrong node type found at {}, {}.", col, row1),
                            }
                            row1 = row2;
                            node_here = node_ahead;
                        },
                        None => panic!("Expected Anchor node to end traversal. (c{col}r)")
                    }
                }
            } else {
                panic!("Expected Anchor node at {col}, {max_row}.");
            }

            if !seed_found {
                panic!("No Seed node found in column {col}")
            }
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

    pub fn next_node(&self, col: usize, row: usize, dir: Dir) -> Option<(usize, usize, Node)> {
        // println!("NEXT_NODE: {col:03}/{row:03}|{dir:?}");
        let height = self.data.len();
        let width = if height == 0 {
            0
        } else {
            self.data[0].len()
        };
        assert!(col < width && row < height);
        fn usable_node(n: Node) -> bool {
            match n {
                Node::Anchor(_) | Node::Proposed(_) | Node::Seed(_) => true,
                _ => false
            }
        }
        let select_node = |c: usize, r: usize| {
            // println!("NODE @ [{c}, {r}]");
            match self.node_at(c, r) {
                Some(n) if usable_node(n) => {
                    // println!("[OK]");
                    Some((c, r, n))
                },
                // _ => None,
                Some(n) => {
                    // println!("[{n:?}]");
                    None
                },
                _ => {
                    // println!("[NONE]");
                    None
                }
            }                
        };
        match dir {
            Dir::N => {
                (0..row).into_iter().rev().find_map(|i| {
                    select_node(col, i)
                })
            },
            Dir::E => {
                ((col+1)..width).into_iter().find_map(|i| {
                    select_node(i, row)
                })
            },
            Dir::S => {
                ((row+1)..height).into_iter().find_map(|i| {
                    select_node(col, i)
                })
            },
            Dir::W => {
                (0..col).into_iter().rev().find_map(|i| {
                    select_node(i, row)
                })
            }
        }
    }

    pub fn slope_from(&self, col: usize, row: usize, dir: Dir) -> Option<f32> {
        let elev1: f32 = match self.node_at(col, row) {
            Some(Node::Anchor(data) | Node::Seed(data) | Node::Proposed(data)) => {
                data.elevation
            },
            _ => return None
        };
        let (row2, col2, elev2) = match self.next_node(col, row, dir) {
            Some((r, c, node)) => match node {
                Node::Anchor(data) | Node::Seed(data) | Node::Proposed(data) => {
                    (r, c, data.elevation)
                },
                _ => return None
            },
            _ => return None
        };
        let distance = match dir {
            Dir::N => row - row2,
            Dir::S => row2 - row,
            Dir::E => col2 - col,
            Dir::W => col - col2,
        };
        Some((elev2 - elev1) / distance as f32)
    }

    pub fn opposite_slope(&self, col: usize, row: usize, dir: Dir) -> f32 {
        match self.slope_from(col, row, dir) {
            Some(slope) => match self.elevation_at(col, row) {
                Some(this_elev) => {
                    let mean_elev = self.globals.mean_elev;
                    let extreme = if this_elev >= mean_elev {
                        self.globals.max_height
                    } else {
                        self.globals.max_depth
                    };
                    let relative_height = (this_elev - mean_elev) / (extreme - mean_elev);
                    let flatten_at_extreme = self.globals.flattening;
                    let reflect_at_extreme = 1.0 - flatten_at_extreme;
                    assert!(reflect_at_extreme >= 0.);
                    let reflect = (reflect_at_extreme - 5.) * relative_height + 5.;
                    let flatten = (flatten_at_extreme - 10.) * relative_height + 10.;
                    let slope_change = {
                        let mut rng = rand::rng();
                        let x = rng.random_range(0.0..1.0);
                        if x <= reflect {
                            SlopeChange::Reflect
                        } else if x <= reflect + flatten {
                            SlopeChange::Flatten
                        } else {
                            SlopeChange::Continue
                        }
                    };
                    match slope_change {
                        SlopeChange::Continue => -slope,
                        SlopeChange::Reflect => slope,
                        SlopeChange::Flatten => 0.,
                    }
                },
                None => 0.
            },
            None => 0.,
        }
    }

    pub fn interpolate(&mut self, col: usize, row: usize, dir: Dir, interval: usize) {
        if let Some((col_ahead, row_ahead, node_ahead)) = self.next_node(col, row, dir) {
            if let Some(node_here) = self.node_at(col, row) {
                let elevation_here = node_here.elevation().unwrap();
                let elevation_ahead = node_ahead.elevation().unwrap();
                let distance_here = distance(col, row, col_ahead, row_ahead);
                if distance_here < interval {
                    println!("WARNING: distance to next node is less than interpolation interval!");
                    return ()
                }
                let interval_ahead = distance_here - interval;

                let adj_strength_here = node_here.strength().unwrap() / (interval as f32);
                let adj_strength_ahead = node_ahead.strength().unwrap() / (interval_ahead as f32);

                let slope_here = self.opposite_slope(col, row, dir.flip());
                let slope_ahead = self.opposite_slope(col_ahead, row_ahead, dir);
                assert!(slope_here <= 1.0 && slope_here >= -1.0);
                assert!(slope_ahead <= 1.0 && slope_ahead >= -1.0);
                let gravity_here = node_here.gravity().unwrap();
                let gravity_ahead = node_ahead.gravity().unwrap();
                let slope_here = gravitate(slope_here, gravity_here, 1.0);
                let slope_ahead = gravitate(slope_ahead, gravity_ahead, 1.0);

                let elev = elevation_here + (slope_here * interval as f32);
                let elev_a = f32::min(
                    f32::max(elev, self.globals.max_depth),
                    self.globals.max_height
                );
                let elev = elevation_ahead + (slope_ahead * interval_ahead as f32);
                let elev_b = f32::min(
                    f32::max(elev, self.globals.max_depth),
                    self.globals.max_height
                );

                let elev_mode = wmean(
                    (elev_a, adj_strength_here),
                    (elev_b, adj_strength_ahead),
                );
                let ruf_mode = wmean(
                    (node_here.roughness().unwrap(), adj_strength_here),
                    (node_ahead.roughness().unwrap(), adj_strength_ahead)
                );
                let grav_mode = wmean(
                    (gravity_here, adj_strength_here),
                    (gravity_ahead, adj_strength_ahead)
                );
                let strn_mode = wmean(
                    (adj_strength_here, adj_strength_here),
                    (adj_strength_ahead, adj_strength_ahead)
                );

                let max_slope = self.globals.max_slope;
                let min_slope = -max_slope;
                let min_elev = f32::max(
                    f32::max(
                        elevation_here + min_slope * interval as f32,
                        elevation_ahead + min_slope * interval_ahead as f32
                    ),
                    self.globals.max_depth
                );
                let max_elev = f32::min(
                    f32::min(
                        elevation_here + max_slope * interval as f32,
                        elevation_ahead + max_slope * interval_ahead as f32
                    ),
                    self.globals.max_height
                );

                let node = Node::Proposed(
                    NodeData::random_nd(
                        (min_elev, max_elev, elev_mode),
                        (0., 1., ruf_mode),
                        (-1., 1., grav_mode),
                        (0., 1., strn_mode)
                    )
                );
                
                self.set_node_at(col_ahead, row_ahead, node);
            }
        }
    }

    fn node_at(&self, col: usize, row: usize) -> Option<Node> {
        match self.data.get(row) {
            Some(r) => match r.get(col) {
                Some(node) => Some(*node),
                None => None,
            }
            None => None,
        }
    }

    fn set_node_at(&mut self, col: usize, row: usize, node: Node) {
        // println!("@SET[{col}][{row}]@");
        match self.data.get_mut(row) {
            Some(r) => {
                r[col] = node;
            },
            None => (),
        }
    }
}

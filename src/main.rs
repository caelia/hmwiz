#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::ops::Index;
use rand::prelude::*;
use image::{GrayImage, Luma};

struct Corners (f32, f32, f32, f32);

#[derive(Debug)]
enum Map {
    Tile ([f32;1089]),
    Meta (Vec<f32>),
    FullMap (Vec<f32>),
}

impl Index<usize> for Map {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match self {
            Map::Tile(arr) => arr.index(index),
            Map::Meta(vec) => vec.index(index),
            Map::FullMap(vec) => vec.index(index),
        }
    }
}

impl Map {
    fn default() -> Self {
        Map::Tile([0.0;1089])
    }
    fn len(&self) -> usize {
        match self {
            Map::Tile(arr) => arr.len(),
            Map::Meta(vec) => vec.len(),
            Map::FullMap(vec) => vec.len(),
        }
    }
    fn size(&self) -> usize {
        let len = self.len() as f32;    
        let root = len.sqrt();
        assert_eq!(root.round(), root);
        root as usize
    }
    fn set(&mut self, x: usize, y: usize, value: f32) {
        let idx = x * self.size() + y;
        match self {
            Map::Tile(arr) => arr[idx] = value,
            Map::Meta(vec) => vec[idx] = value,
            Map::FullMap(vec) => vec[idx] = value,
        }
    }
}

#[derive(Debug)]
enum Step {
    Diamond,
    Square,
}

// fixed-boundary averaging function
fn avg(map: &mut Map, x: usize, y: usize, dist: usize, offsets: [(isize, isize); 4]) -> f32 {
    let size = map.size();
    let mut result = 0.0;
    let mut k = 0.0;

    for (p, q) in offsets {
        let pp = x as isize + p * dist as isize;
        let qq = y as isize + q * dist as isize;
        // println!("p: {}, pp {}, q: {}, qq: {}", p, pp, q, qq);
        if pp >= 0 && pp < size as isize && qq >= 0 && qq < size as isize {
            let upp = pp as usize;
            let uqq = qq as usize;
            result += map[upp * size + uqq];
        }
        k += 1.0;    
    }
    result / k
}

fn set_point(map: &mut Map, x: usize, y: usize, btm: f32, top: f32,
             distance: usize, variance: f32, step_type: Step) {
    let offsets = match step_type {
        Step::Diamond =>  [(-1, -1), (-1, 1), (1, 1), (1, -1)],
        Step::Square => [(-1, 0), (0, -1), (1, 0), (0, 1)],
    };
    
    let mut rng = thread_rng();
    let random_delta = rng.gen_range(-variance..=variance);
    let nuval = avg(map, x, y, distance, offsets) + random_delta;
    // map.set(x * map.size() + y, nuval);
    map.set(x, y, nuval);
}

fn ds_step(map: &mut Map, btm: f32, top: f32, distance: usize, variance: f32) {
    let size = map.size();
    let distance_ = distance / 2;
    let limit = size/distance;

    // Diamond Step
    for x_ in 0..limit {
        let x = x_ * distance + distance_;
        for y_ in 0..limit {
            let y = y_ * distance + distance_;
            set_point(map, x, y, btm, top, distance_, variance, Step::Diamond);
        }
    }

    // Square Step, rows
    for x_ in 0..limit {
        let x = x_ * distance + distance_;
        for y_ in 0..=limit {
            let y = y_ * distance;
            set_point(map, x, y, btm, top, distance_, variance, Step::Square);
        }
    }

    // Square Step, cols
    for x_ in 0..=limit {
        let x = x_ * distance;
        for y_ in 0..limit {
            let y = y_ * distance + distance_;
            set_point(map, x, y, btm, top, distance_, variance, Step::Square);
        }
    }
}

fn generate_tile(corners: Corners, btm: f32, top: Option<f32>, roughness: f32) -> Map {
    let top = match top {
        Some(n) => n,
        None => 255.0,
    };

    let mut tile = Map::default();

    let Corners(ul, ur, ll, lr) = corners;
    tile.set(0, 0, ul);
    tile.set(0, 32, ur);
    tile.set(32, 0, ll);
    tile.set(32, 32, lr);
    let mut distance = tile.size() - 1;
    let mut variance = 1.0;

    while distance > 1 {
        ds_step(&mut tile, btm, top, distance, variance);
        distance = distance / 2;
        variance = variance * roughness;
    }    
    tile
}

fn generate_map(ntiles: usize, min_roughness: f32, max_roughness: f32) -> Map {
    let mut tiles: Vec<Map> = Vec::new();
    Map::default()
}

fn main() {
    let tile = generate_tile(Corners(0.2,0.2,0.2,0.2), 0.0, Some(255.0), 0.65);
    let mut img = GrayImage::new(33, 33);
    for x in 0..33 {
        for y in 0..33 {
            let val_ = tile[x * 33 + y];
            // let val = (val_ * 128.0 + 128.0) as u8;
            let val_x = (val_ * 128.0 + 128.0);
            if val_x > 255.0 {
                panic!("Your fucking numbar are too bige! {}", val_x);
            }
            let val = val_x as u8;
            img.put_pixel(x as u32, y as u32, Luma([val]));
        }
    }
    img.save("test.png");
}

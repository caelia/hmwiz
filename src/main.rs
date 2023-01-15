#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::ops::{Index, RangeInclusive};
use rand::prelude::*;
use rand_distr::{LogNormal, Distribution};
// use rand_distr::{Normal, Distribution};
use image::{GrayImage, Luma};

struct Corners (f32, f32, f32, f32);

#[derive(Debug)]
enum MapKind{
    Primary,
    Meta,
    Final,
    Temp,
}
#[derive(Debug)]
struct Map<T> {
    kind: MapKind,
    size: usize,
    meta_size: usize,
    data: Vec<T>,
}

impl<T> Map<T> {
    fn new(kind: MapKind, size: usize, meta_size: Option<usize>) -> Self {
        let length = size * size;
        let meta_size = match meta_size {
            Some(n) => n,
            None => size,
        };
        let data: Vec<T> = Vec::with_capacity(length);
        unsafe {
            data.set_len(length);
        }
        Map { kind, size, meta_size, data }
    }
    fn primary_map(size: usize) -> Self {
        Map::new(MapKind::Primary, size, None)
    }
    fn meta_map (size: usize, meta_size: usize) -> Self {
        Map::new(MapKind::Meta, size, Some(meta_size))
    }
    fn final_map (size: usize) -> Self {
        Map::new(MapKind::Final, size, None)
    }
    fn final_map(size: usize) -> Self {
        Map::new(MapKind::Temp, size, None)
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn size(&self) -> usize {
        self.size
    }
    fn get(&self, row: usize, col: usize) -> f32 {
        let idx = row * self.size + col;
        self.data[idx]
    }
    fn set(&mut self, row: usize, col: usize, value: f32) {
        let idx = row * self.size + col;
        self.data[idx] = value;
    }
}

#[derive(Debug)]
enum Rufs {
    RVal (f32),
    RMap (Map<Option<f32>>),
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
        if pp >= 0 && pp < size as isize && qq >= 0 && qq < size as isize {
            let upp = pp as usize;
            let uqq = qq as usize;
            result += map.get(upp, uqq);
        }
        k += 1.0;    
    }
    result / k
}

fn rand_exp() -> f32 {
    let distro = LogNormal::new(-1.0, 0.5).unwrap();
    let result = distro.sample(&mut thread_rng());
    // println!("Random exponent = {}", result);
    result
}

fn random_delta(range: RangeInclusive<f32>, local_variance: f32) -> f32 {
    let mut rng = thread_rng();
    let base = rng.gen_range(range);
    base * local_variance * rand_exp().exp2()
}

// "Folds" the delta if the result would be outside the allowed range.
fn apply_delta(value: f32, delta: f32, btm: f32, top: f32) -> f32 {
    if value < btm {
        println!("WARNING - value outside of allowed range: {}", value);
        return btm;
    } else if value > top {
        println!("WARNING - value outside of allowed range: {}", value);
        return top;
    }
    let candidate = value + delta;
    if candidate < btm || candidate > top {
        println!("Tentative result outside allowed range: {}", candidate);
        value - delta
    } else {
        candidate
    }
}


fn set_point(map: &mut Map<f32>, x: usize, y: usize, btm: f32, top: f32,
             distance: usize, variance: f32, step_type: Step) {
    let offsets = match step_type {
        Step::Diamond =>  [(-1, -1), (-1, 1), (1, 1), (1, -1)],
        Step::Square => [(-1, 0), (0, -1), (1, 0), (0, 1)],
    };
    
    let average = avg(map, x, y, distance, offsets);
    /*
    let exponent = rand_exp() * variance;
    // println!("exponent = {}", exponent);
    let mult = exponent.exp2();
    println!("multiplier = {}", mult);
    */
    // let nuval = average * (rand_exp() * variance).exp2();
    // let delta = rand_exp() * variance;
    // println!("delta = {}", delta);
    // let nuval = average * mult;
    // let nuval = average + delta;
    let diff = top - btm;
    let delta = random_delta(-(diff/4.0)..=(diff/2.0), variance);
    // println!("delta = {}", delta);
    // let nuval = average + delta;
    let nuval = apply_delta(average, delta, btm, top);
    // println!("nuval = {}", nuval);
    map.set(x, y, nuval);
}

fn interpolate(map: Map<Option<f32>>, row, col) {
    
}

fn get_local_variance(rufs: &mut Map<Option<f32>>, row: usize, col: usize) -> f32 {
    match rufs.get(row, col) {
        Some(n) => n,
        None => {
            let n = interpolate(rufs, row, col);
            rufs.set(row, col, Some(n));
            n
        }
    }
}

fn ds_step(map: &mut Map<f32>, rufs: Rufs, btm: f32, top: f32, distance: usize, variance: f32) {
    let size = map.size();
    let distance_ = distance / 2;
    let limit = size/distance;

    // Diamond Step
    for row_ in 0..limit {
        let row = row_ * distance + distance_;
        for col_ in 0..limit {
            let col = col_ * distance + distance_;
            set_point(map, row, col, btm, top, distance_, variance, Step::Diamond);
        }
    }

    // Square Step, rows
    for row_ in 0..limit {
        let row = row_ * distance + distance_;
        for col_ in 0..=limit {
            let col = col_ * distance;
            set_point(map, row, col, btm, top, distance_, variance, Step::Square);
        }
    }

    // Square Step, cols
    for row_ in 0..=limit {
        let row = row_ * distance;
        for col_ in 0..limit {
            let col = col_ * distance + distance_;
            set_point(map, row, col, btm, top, distance_, variance, Step::Square);
        }
    }
}

fn fill_map(map: &mut Map<f32>, roughness: f32, rufs: Rufs, corners: Corners, btm: f32, top: f32) {
    let size = map.size();
    let limit = size - 1;
    let Corners(ul, ur, ll, lr) = corners;
    map.set(0, 0, ul);
    map.set(0, limit, ur);
    map.set(limit, 0, ll);
    map.set(limit, limit, lr);
    let mut distance = size - 1;
    let mut variance = 1.0;

    while distance > 1 {
        ds_step(map, rufs, btm, top, distance, variance);
        distance = distance / 2;
        variance = variance * rufs;
    }    
}


fn generate_primary(size: usize, corners: Corners, btm: f32, top: f32, roughness: f32) -> Map {
    let mut map = Map::primary_map(size);
    fill_map(&mut map, corners, btm, top, roughness);
    map
}

fn generate_roughness_map(size: usize,  corners: Corners, btm: f32, top: f32, roughness: f32) -> Map {
    let mut temp_map = Map::meta_map(size);
    fill_map(&mut rufs, corners, btm, top, roughness);
    rufs
}

fn main() {
    let map1 = generate_primary(1025, Corners(0.0, 0.0, 0.0, 0.0), 0.0, 376.0, 0.67);
    let mut img1 = GrayImage::new(1025, 1025);
    for x in 0..1025 {
        for y in 0..1025 {
            let val_ = map1[x * 1025 + y];
            // /*
            let val = if val_ <= 63.0 {
                0.0
            } else if val_ <= 96.0 {
                1.0
            } else if val_ <= 112.0 {
                2.0
            } else if val_ <= 120.0 {
                3.0
            } else if val_ <= 124.0 {
                4.0
            } else if val_ <= 126.0 {
                5.0
            } else {
                val_ - 121.0
            } as u8;
            // */
            // let val = val_ as u8;
            img1.put_pixel(x as u32, y as u32, Luma([val]));
        }
    }
    img1.save("test1.png");
    let map2 = generate_tile(Corners(0.0, 0.0, 0.0, 0.0), 0.0, 255.0, 0.6);
    let mut img2 = GrayImage::new(33, 33);
    for x in 0..33 {
        for y in 0..33 {
            let val = map2[x * 33 + y] as u8;
            img2.put_pixel(x as u32, y as u32, Luma([val]));
        }
    }
    img2.save("test2.png");
}

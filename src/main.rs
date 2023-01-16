#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::ops::Index;
use rand::prelude::*;
use rand_distr::{Pert, Distribution};
use image::{GrayImage, Luma};

struct Corners (f32, f32, f32, f32);

#[derive(Debug)]
enum MapKind {
    Primary,
    Meta,
    Temp,
    Final,
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
        let meta_size = match meta_size {
            Some(s) => s,
            None => size,
        };
        let length = size * size;
        let mut data = Vec::with_capacity(length);
        unsafe {
            data.set_len(length);
        }
        Map { kind, size, meta_size, data }
    }
    fn primary_map(size: usize) -> Self {
        Map::new(MapKind::Primary, size, None)
    }
    fn meta_map(size: usize, meta_size: usize) -> Self {
        Map::new(MapKind::Meta, size, Some(meta_size))
    }
    fn temp_map(size: usize) -> Self {
        Map::new(MapKind::Temp, size, None)
    }
    fn final_map(size: usize) -> Self {
        Map::new(MapKind::Final, size, None)
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn size(&self) -> usize {
        self.size
    }
    fn meta_size(&self) -> usize {
        self.meta_size
    }
    fn get(&self, row: usize, col: usize) -> &T {
        let idx = row * self.size + col;
        &self.data[idx]
    }
    fn set(&mut self, row: usize, col: usize, value: T) {
        let idx = row * self.size + col;
        self.data[idx] = value;
    }
}

#[derive(Debug)]
enum Step {
    Diamond,
    Square,
}

#[derive(Debug, Clone, Copy)]
struct Bias {
    value: Option<f32>,
}

impl Bias {
    fn new(n: f32) -> Self {
        Bias { value: Some(n) }
    }
    fn step(&mut self) {
        match self.value {
            Some(n) => {
                let nuval = -(n / 2.0);
                if nuval.abs() < 1.0 {
                    self.value = None;
                } else {
                    self.value = Some(nuval);
                }
            },
            None => (),
        }
    }
}

// fixed-boundary averaging function
fn avg(map: &mut Map<f32>, x: usize, y: usize, dist: usize, offsets: [(isize, isize); 4]) -> f32 {
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
            result += map.get(upp, uqq);
        }
        k += 1.0;    
    }
    result / k
}

fn random_target(btm: f32, top: f32, mode: f32) -> f32 {
    let distro = Pert::new(btm, top, mode).unwrap();
    let result = distro.sample(&mut thread_rng());
    // println!("Random exponent = {}", result);
    result
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
             distance: usize, variance: f32, bias: Bias, step_type: Step) {
    let offsets = match step_type {
        Step::Diamond =>  [(-1, -1), (-1, 1), (1, 1), (1, -1)],
        Step::Square => [(-1, 0), (0, -1), (1, 0), (0, 1)],
    };
    
    let average = avg(map, x, y, distance, offsets);

    let target = match bias.value {
        // Some(b) => random_target(btm, top, b),
        Some(b) => random_target(btm, top, f32::max(f32::min(average + b, top), btm)),
        None => random_target(btm, top, average),
    };

    let delta = (target - average) * variance;
    let nuval = average + delta;

    // println!("Average: {}, Target: {}, Delta: {}, Nuval: {}", average, target, delta, nuval);
    
    map.set(x, y, nuval);
}

fn ds_step(map: &mut Map<f32>, btm: f32, top: f32, distance: usize,
           variance: f32, bias: Bias) {
    let size = map.size();
    let distance_ = distance / 2;
    let limit = size/distance;

    // Diamond Step
    for x_ in 0..limit {
        let x = x_ * distance + distance_;
        for y_ in 0..limit {
            let y = y_ * distance + distance_;
            set_point(map, x, y, btm, top, distance_, variance, bias, Step::Diamond);
        }
    }

    // Square Step, rows
    for x_ in 0..limit {
        let x = x_ * distance + distance_;
        for y_ in 0..=limit {
            let y = y_ * distance;
            set_point(map, x, y, btm, top, distance_, variance, bias, Step::Square);
        }
    }

    // Square Step, cols
    for x_ in 0..=limit {
        let x = x_ * distance;
        for y_ in 0..limit {
            let y = y_ * distance + distance_;
            set_point(map, x, y, btm, top, distance_, variance, bias, Step::Square);
        }
    }
}

fn fill_map(map: &mut Map<f32>, corners: Corners, btm: f32, top: f32,
            roughness: f32, mut bias: Bias) {
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
        ds_step(map, btm, top, distance, variance, bias);
        distance = distance / 2;
        variance = variance * roughness;
        bias.step();
    }    
}

fn stretch(map: &mut Map<f32>, btm: f32, top: f32) {
    let mut lo = top;
    let mut hi = btm;
    let mut changed = false;

    for row in 0..map.size() {
        for col in 0..map.size() {
            let value = *map.get(row, col);
            if value < lo {
                lo = value;
                changed = true;
            }
            if value > hi {
                hi = value;
                changed = true;
            }
        }
    }

    if changed {
        let offset = lo - btm;
        let stretch_factor = (top - btm) / (hi - lo);
        for row in 0..map.size() {
            for col in 0..map.size() {
                let value = *map.get(row, col);
                let nuval = (value - offset) * stretch_factor;
                if nuval < btm || nuval > top {
                    println!("BAD VALUE: {}", nuval);
                }
                map.set(row, col, nuval);
            }
        }
    }
}
// TEMPORARY, I THINK
fn generate_megatile(size: usize, corners: Corners, btm: f32, top: f32,
                     roughness: f32, start_bias: f32) -> Map<f32> {
    let mut map = Map::primary_map(size);
    // let mut map = Map::primary_map(1025);
    fill_map(&mut map, corners, btm, top, roughness, Bias::new(start_bias));
    stretch(&mut map, btm, top);
    map
}


fn make_composite_map() {
    let map0 = generate_megatile(2049, Corners(0.0, 0.0, 0.0, 0.0), 0.0, 64.0, 0.67, 32.0);
    let map1 = generate_megatile(2049, Corners(0.0, 0.0, 0.0, 0.0), 0.0, 312.0, 0.51, 156.0);
    let mut img1 = GrayImage::new(2049, 2049);
    for row in 0..2049 {
        for col in 0..2049 {
            let val_ = map0.get(row, col) + map1.get(row, col);
            if val_ < 0.0 || val_ > 376.0 {
                println!("BAD VALUE: {}", val_);
            }
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
            img1.put_pixel(row as u32, col as u32, Luma([val]));
        }
    }
    let _ = img1.save("test.png");
}

fn make_one_layer_map() {
    let map = generate_megatile(2049, Corners(0.0, 0.0, 0.0, 0.0), 0.0, 376.0, 0.7, 300.0);
    let mut max_val: f32 = 0.0;
    let mut img = GrayImage::new(2049, 2049);
    for row in 0..2049 {
        for col in 0..2049 {
            let val_ = *map.get(row, col);
            if val_ < 0.0 || val_ > 376.0 {
                println!("BAD VALUE: {}", val_);
            }
            if val_ > max_val {
                max_val = val_;
            }
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
            img.put_pixel(row as u32, col as u32, Luma([val]));
        }
    }
    let _ = img.save("test.png");
}

fn make_quadrant_map() {
    let map_ul = generate_megatile(1025, Corners(0., 0., 0., 188.), 0., 376., 0.7, 300.);
    let map_ur = generate_megatile(1025, Corners(0., 0., 188., 0.), 0., 376., 0.7, 300.);
    let map_ll = generate_megatile(1025, Corners(0., 188., 0., 0.), 0., 376., 0.7, 300.);
    let map_lr = generate_megatile(1025, Corners(188., 0., 0., 0.), 0., 376., 0.7, 300.);
    let mut max_val: f32 = 0.0;
    let mut img = GrayImage::new(2050, 2050);
    for (map, hoff, voff) in [
            (map_ul, 0, 0),
            (map_ur, 1025, 0),
            (map_ll, 0, 1025),
            (map_lr, 1025, 1025)
        ] {
        for row in 0..1025 {
            for col in 0..1025 {
                let val_ = *map.get(row, col);
                if val_ < 0.0 || val_ > 376.0 {
                    println!("BAD VALUE: {}", val_);
                }
                if val_ > max_val {
                    max_val = val_;
                }
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
                img.put_pixel((row + voff) as u32, (col + hoff) as u32, Luma([val]));
            }
        }
    }
    let _ = img.save("qtest.png");
}

fn main() {
    // make_composite_map();
    // make_one_layer_map();
    make_quadrant_map();
}

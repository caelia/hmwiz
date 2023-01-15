#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::ops::Index;
use rand::prelude::*;
use rand_distr::{LogNormal, Distribution};
// use rand_distr::{Normal, Distribution};
use image::{GrayImage, Luma};

struct Corners (f32, f32, f32, f32);

#[derive(Debug)]
enum Map {
    Tile ([f32;1089]),
    Meta (Vec<Vec<f32>>),
    FullMap (Vec<f32>),
}

impl Index<usize> for Map {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match self {
            Map::Tile(arr) => arr.index(index),
            Map::Meta(vex) => {
                let vec = vex.index(index / vex.len());
                vec.index(index % vex.len())
            },
            Map::FullMap(vec) => vec.index(index),
        }
    }
}

impl Map {
    fn default() -> Self {
        Map::Tile([0.0;1089])
    }
    fn full_map(cap: usize) -> Self {
        let mut data = Vec::with_capacity(cap);
        unsafe {
            data.set_len(cap);
        }
        Map::FullMap(data)
    }
    fn meta_map(cap: usize) -> Self {
        let mut vex = Vec::new();
        for _ in 0..cap {
            let mut vec = Vec::with_capacity(cap);
            unsafe {
                vec.set_len(cap);
            }
            vex.push(vec)    
        }
        Map::Meta(vex)
    }
    fn len(&self) -> usize {
        match self {
            Map::Tile(arr) => arr.len(),
            Map::Meta(vec) => vec.len() * vec[0].len(),
            Map::FullMap(vec) => vec.len(),
        }
    }
    fn size(&self) -> usize {
        let len = self.len() as f32;
        let root = len.sqrt();
        assert_eq!(root.round(), root);
        root as usize
    }
    fn get(&self, x: usize, y: usize) -> f32 {
        match self {
            Map::Meta(vex) => {
                vex[x][y]
            },
            _ => panic!("get() doesn't handle this type of map"),
        }
    }
    fn set(&mut self, x: usize, y: usize, value: f32) {
        let idx = x * self.size() + y;
        // println!("MAP: {:?}", self);
        // println!("IDX: {}", idx);
        match self {
            Map::Tile(arr) => arr[idx] = value,
            Map::Meta(vex) => vex[x][y] = value,
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

fn rand_exp() -> f32 {
    let distro = LogNormal::new(-1.0, 0.5).unwrap();
    let result = distro.sample(&mut thread_rng());
    // println!("Random exponent = {}", result);
    result
}

fn random_delta(range: std::ops::RangeInclusive<f32>, local_variance: f32) -> f32 {
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


fn set_point(map: &mut Map, x: usize, y: usize, btm: f32, top: f32,
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

fn fill_map(map: &mut Map, corners: Corners, btm: f32, top: f32, roughness: f32) {
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
        ds_step(map, btm, top, distance, variance);
        distance = distance / 2;
        variance = variance * roughness;
    }    
}

fn generate_tile(corners: Corners, btm: f32, top: f32, roughness: f32) -> Map {
    let mut tile = Map::default();
    fill_map(&mut tile, corners, btm, top, roughness);
    tile
}

// TEMPORARY, I THINK
fn generate_megatile(corners: Corners, btm: f32, top: f32, roughness: f32) -> Map {
    // let mut map = Map::full_map(4_198_401); // 2049x2049
    let mut map = Map::full_map(1_050_625); // 1025x1025
    fill_map(&mut map, corners, btm, top, roughness);
    map
}

fn remove_middle<T>(vec: &mut Vec<T>, start: usize, end: usize) {
    for i in start..end {
        vec.remove(i);
    }
}

fn collapse_meta(map: &mut Map, n_items: usize) {
    match map {
        Map::Meta(vex) => {
            let size = vex.len();
            let start = (size - n_items) / 2;
            let end = start + n_items;
            remove_middle(vex, start, end);
            for vec in vex {
                remove_middle(vec, start, end);
            }
        },
        _ => panic!("collapse_meta can't handle this type of map.")
    }
}

fn generate_vertex_map(corners: Corners, btm: f32, top: f32, roughness: f32) -> Map {
    let mut vtxs = Map::meta_map(33);
    fill_map(&mut vtxs, corners, btm, top, roughness);
    collapse_meta(&mut vtxs, 1);
    vtxs
}

fn generate_roughness_map(corners: Corners, btm: f32, top: f32, roughness: f32) -> Map {
    let mut rufs = Map::meta_map(33);
    fill_map(&mut rufs, corners, btm, top, roughness);
    collapse_meta(&mut rufs, 2);
    rufs
}

fn generate_map(ntiles: usize, min_roughness: f32, max_roughness: f32) -> Map {
    let vtxs = generate_vertex_map(Corners(0.0, 0.0, 0.0, 0.0), 0.0, 192.0, 0.4);
    let rufs = generate_roughness_map(Corners(0.45, 0.45, 0.45, 0.45), 0.3, 0.65, 0.4);

    let mut tiles: Vec<Map> = Vec::new();
    let mut rng = thread_rng();
    // for _ in 0..4096 {
    for row in 0..31 {
        for col in 0..31 {
            println!("{}/{}", row, col);
            // let ruffness = rng.gen_range(min_roughness..=max_roughness);
            let ul = vtxs.get(row, col);
            let ur = vtxs.get(row, col+1);
            let ll = vtxs.get(row+1, col);
            let lr = vtxs.get(row+1, col+1);
            let tile = generate_tile(Corners(ul, ur, ll, lr), 0.0, 255.0, rufs.get(row, col));
            tiles.push(tile);
        }
    }
    // let mut map = Map::full_map(4_460_544);
    let mut map = Map::full_map(1_115_136);
    for grand_row in 0..31 {
        for grand_col in 0..31 {
            let tile_idx = grand_row * 31 + grand_col;
            let tile = &tiles[tile_idx];
            for row in 0..33 {
                for col in 0..33 {
                    let map_row = grand_row * 33 + row;
                    let map_col = grand_col * 33 + col;
                    // let map_pos = map_row * 2112 + map_col;
                    let tile_pos = row * 33 + col;
                    map.set(map_row, map_col, tile[tile_pos]);
                }
            }
        }
    }
    map
}

fn main() {
    // println!("{:?}", tile);
    /*
    let map = generate_map(64, 0.4, 0.7);
    let mut img = GrayImage::new(1056, 1056);
    for x in 0..1056 {
        for y in 0..1056 {
            let val = map[x * 1056 + y] as u8;
            img.put_pixel(x as u32, y as u32, Luma([val]));
        }
    }
    */
    let map1 = generate_megatile(Corners(0.0, 0.0, 0.0, 0.0), 0.0, 376.0, 0.67);
    /*
    let mut img1 = GrayImage::new(2049, 2049);
    for x in 0..2049 {
        for y in 0..2049 {
            let val_ = map1[x * 2049 + y];
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
            img1.put_pixel(x as u32, y as u32, Luma([val]));
        }
    }
    */
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

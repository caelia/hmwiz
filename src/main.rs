// #![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// #![allow(dead_code)]

use std::ops::Index;
use rand::prelude::*;
use rand_distr::{Pert, Distribution};
use image::{GrayImage, GenericImage, ImageBuffer, Luma};

const GLOBAL_MIN: f32 = 0.;
const GLOBAL_MAX: f32 = 376.;
// const GLOBAL_MAX: f32 = 313.;
// const GLOBAL_MAX: f32 = 255.;
const MAX_SLOPE: f32 = 1.;

#[derive(Debug)]
enum Dir {
    H,
    V
}

#[derive(Debug)]
struct Map {
    size: usize,
    data: Vec<f32>,
}

impl Map {
    fn new(size: usize) -> Self {
        let length = size * size;
        /*
        let mut data = Vec::with_capacity(length);
        unsafe {
            data.set_len(length);
        }
        */
        let data = vec![0.;length];
        Map { size, data }
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

    fn minmax_from(&self, nrow: usize, ncol: usize, row: usize, col: usize) -> (f32, f32) {
        let hdiff = if nrow == row {
            if ncol >= col {
                ncol - col
            } else {
                col - ncol
            }
        } else if ncol == col {
            if nrow >= row {
                nrow - row
            } else {
                row - nrow
            }
        } else {
            panic!("Can't calculate diagonal distance.");
        } as f32;
        assert!(hdiff > 0.);
        let nheight = self.get(nrow, ncol);
        // (nheight + hdiff * -MAX_SLOPE, nheight + hdiff * MAX_SLOPE)
        let minfrom = nheight + hdiff * -MAX_SLOPE;
        let maxfrom = nheight + hdiff * MAX_SLOPE;
        (minfrom, maxfrom)
    }

    fn avg_height(&self, r1: usize, c1: usize, r2: usize, c2: usize) -> f32 {
        assert!(r1 != r2 || c1 != c2);
        (self.get(r1, c1) + self.get(r2, c2)) / 2.
    }

    fn set_point(&mut self, row: usize, col: usize, ndist: usize, dir: Dir) {
        // println!("{}, {}", row, col);
        let (nrow1, ncol1, nrow2, ncol2) = match dir {
            Dir::H => (row, col - ndist, row, col + ndist),
            Dir::V => (row - ndist, col, row + ndist, col),
        };
        let (lmin1, lmax1) = self.minmax_from(nrow1, ncol1, row, col);
        let (lmin2, lmax2) = self.minmax_from(nrow2, ncol2, row, col);
        let lo = f32::max(f32::max(lmin1, lmin2), GLOBAL_MIN);
        let hi = f32::min(f32::min(lmax1, lmax2), GLOBAL_MAX);
        let avg = self.avg_height(nrow1, ncol1, nrow2, ncol2);
        // assert!(hi >= lo && avg <= hi && avg >= lo);
        match Pert::new(lo, hi, avg) {
            Ok(distro) => {
                let height = distro.sample(&mut thread_rng()); 
                self.set(row, col, height);
            },
            Err(_e) => {
                // println!("Error creating Pert distro (lo: {}, hi: {}, avg: {}", lo, hi, avg);
                self.set(row, col, avg);
            }
        }

    }


    fn fill(&mut self, init_peak: f32) { 
        let size = self.size();
        self.set(size / 2, size / 2, init_peak);

        /*
        // JUST TESTING!
        for row in [0, 2048] {
            for col in 0..2049 {
                self.set(row, col, 0.0);
            }
        }
        for row in 1..2048 {
            for col in [0, 2049] {
                self.set(row, col, 0.0);
            }
        }
        */

        let mut fill_rank = |rank: usize, mut step: usize, dir: Dir| {
            let mut ndist = step / 2;
            while step > 1 {
                let mut pos = step / 2;
                while pos < size {
                    match dir {
                        Dir::H => self.set_point(rank, pos, ndist, Dir::H),
                        Dir::V => self.set_point(pos, rank, ndist, Dir::V),
                    }
                    pos += step;
                }
                step /= 2;
                ndist = step / 2;
            }
        };

        fill_rank(size / 2, size / 2, Dir::V);

        let mut maj_step = size;
        let mut maj_pos = maj_step / 2;
        while maj_step > 1 {
            let min_step = maj_step / 2;
            while maj_pos < size {
                fill_rank(maj_pos, min_step, Dir::H);
                maj_pos += maj_step;
            }
            maj_step /= 2;
            maj_pos = maj_step / 2;
            while maj_pos < size {
                fill_rank(maj_pos, min_step, Dir::V);
                maj_pos += maj_step;
            }
            maj_pos = maj_step / 2;
        }
    }

    fn stretch(map: &mut Map, btm: f32, top: f32) {
        let mut lo = top;
        let mut hi = btm;
        let mut changed = false;

        for row in 0..map.size() {
            for col in 0..map.size() {
                let value = map.get(row, col);
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
                    let value = map.get(row, col);
                    let nuval = (value - offset) * stretch_factor;
                    if nuval < btm || nuval > top {
                        println!("BAD VALUE: {}", nuval);
                    }
                    map.set(row, col, nuval);
                }
            }
        }
    }
}


fn generate_map(size: usize, init_peak: f32) -> Map {
    let mut map = Map::new(size);
    map.fill(init_peak);
    // stretch(&mut map, btm, top);
    map
}


fn make_one_layer_map_hi_bf(filename: String) {
    let map = generate_map(2049, 312.);
    let mut max_val: f32 = 0.0;
    let mut img = GrayImage::new(2049, 2049);
    for row in 0..2049 {
        for col in 0..2049 {
            let val_ = map.get(row, col);
            if val_ < 0.0 || val_ > 376.0 {
                println!("BAD VALUE: {}", val_);
            }
            if val_ > max_val {
                max_val = val_;
            }
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
            // let val = val_ as u8;
            img.put_pixel(row as u32, col as u32, Luma([val]));
        }
    }
    let _ = img.save(filename);
}

fn make_one_layer_map_lo_bf(filename: String) {
    //let map = generate_map(2049, 276.);
    let map = generate_map(2049, 219.);
    let mut max_val: f32 = 0.0;
    let mut img = GrayImage::new(2049, 2049);
    for row in 0..2049 {
        for col in 0..2049 {
            let val_ = map.get(row, col);
            if val_ < 0.0 || val_ > 313.0 {
                println!("BAD VALUE: {}", val_);
            }
            if val_ > max_val {
                max_val = val_;
            }
            let val = if val_ <= 32.0 {
                0.0
            } else if val_ <= 48.0 {
                1.0
            } else if val_ <= 56.0 {
                2.0
            } else if val_ <= 60.0 {
                3.0
            } else if val_ <= 62.0 {
                4.0
            } else {
                val_ - 58.0
            } as u8;
            // let val = val_ as u8;
            img.put_pixel(row as u32, col as u32, Luma([val]));
        }
    }
    let _ = img.save(filename);
}

fn make_one_layer_map_no_bf(filename: String) {
    let map = generate_map(2049, 250.);
    let mut img = GrayImage::new(2049, 2049);
    for row in 0..2049 {
        for col in 0..2049 {
            let val_ = map.get(row, col);
            if val_ < 0.0 || val_ > 255.0 {
                println!("BAD VALUE: {}", val_);
            }
            let val = val_ as u8;
            img.put_pixel(row as u32, col as u32, Luma([val]));
        }
    }
    let _ = img.save(filename);
}


fn main() {
    let args = std::env::args().into_iter().collect::<Vec<String>>();
    let filename = if args.len() > 1 {
        format!("{}.png", args[1])
    } else {
        "hmwiz.png".to_string()
    };
    make_one_layer_map_hi_bf(filename);
    // make_one_layer_map_lo_bf(filename);
    // make_one_layer_map_no_bf(filename);
}

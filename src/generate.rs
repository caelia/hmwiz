use image::{Rgb, RgbImage};
// use std::path::PathBuf;
use crate::grid::{Node, Grid};

fn rgb_at(height: f32) -> (u8, u8, u8) {
    let rounded = height.round();
    if rounded <= 0. {
        (0, 0, 255)
    } else if rounded < 128. {
        (0, rounded as u8 + 127, 0)
    } else {
        let x = (rounded as u8 - 128) * 2;
        (x, 255, x)
    }
}

fn temp_rgb_at(height: f32) -> (u8, u8, u8) {
    let rounded = height.round();
    if rounded <= 0. {
        (0, 0, 0)
    } else if rounded >= 255. {
        (255, 255, 255)
    } else {
        let val = rounded as u8;
        (val, val, val)
    }
}

fn rgb_undef() -> (u8, u8, u8) {
    (127, 0, 0)
}

fn generate(grid: Grid, filename: &str) {
    let (rows, cols) = grid.dimensions();
    let mut img = RgbImage::new(cols as u32, rows as u32);
    for r in 0..rows {
        for c in 0..cols {
            let (r, g, b) = match grid.elevation_at(c, r) {
                Some(elev) => temp_rgb_at(elev),
                None => rgb_undef(),
            };
        }
    }
}

pub fn generate_x4(grid: Grid, filename: &str) {
    let (rows, cols) = grid.dimensions();
    let rows = rows as u32;
    let cols = cols as u32;
    let mut img = RgbImage::new(cols * 4, rows * 4);
    for row in 0..rows {
        for col in 0..cols {
            let (r, g, b) = match grid.elevation_at(col as usize, row as usize) {
                Some(elev) => temp_rgb_at(elev),
                None => rgb_undef(),
            };
            for xoff in 0..4 {
                for yoff in 0..4 {
                    img.put_pixel(col * 4 + xoff, row * 4 + yoff, Rgb([r, g, b]));
                }
            }
        }
    }
    let _ = img.save(filename);
}

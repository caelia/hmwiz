use rand::random_range;
use image::{ImageReader, Luma};
use image::DynamicImage as DynImg;
use std::fmt::{Display, Formatter};

use crate::common::*;

pub enum MapDiscrepancyHandling {
    Uniform,
    Random,
    Abort
}

type MDH = MapDiscrepancyHandling;

#[derive(Debug, Copy, Clone)]
pub enum Pt {
    Undef,
    Water,
    Edge,
    Seed(u8),
    IntV(u8),
    IntH(u8),
    Land(u8)
}

impl Display for Pt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Pt::Undef => "[ ]".to_string(),
            Pt::Water => "[=]".to_string(),
            Pt::Edge  => "[+]".to_string(),
            Pt::Seed(elev) |Pt::IntV(elev) |Pt::IntH(elev) => format!("<{}>", elev),
            Pt::Land(elev) => format!("[{}]", elev)
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct HeightMapBuilder<const W: usize, const H: usize> {
    basemap: [[Pt; W]; H],
    hbmap: [[f32; W]; H],
    rhmap: [[f32; W]; H],
    peaks: Vec<(usize, usize)>
}

impl<const W: usize, const H: usize> HeightMapBuilder<W, H> {
    pub fn new() -> HeightMapBuilder<W, H> {
        HeightMapBuilder {
            basemap: [[Pt::Undef; W]; H],
            hbmap: [[0.0; W]; H],
            rhmap: [[0.0; W]; H],
            peaks: vec![]
        }
    }

    pub fn load_lf(&mut self, img_file: &str) {
        let dimg = ImageReader::open(img_file).expect("Failed to open image file.")
            .decode().expect("Failed to decode image.");
        match dimg {
            DynImg::ImageLuma8(img) => {
                assert_eq!(img.width(), W as u32);
                assert_eq!(img.height(), H as u32);
                for r in 0..H {
                    for c in 0..W {
                        let Luma([val]) = img.get_pixel(c as u32, r as u32);
                        match val {
                            0 => self.basemap[r][c] = Pt::Water,
                            v => {
                                assert_eq!(*v, 255)
                            }
                        }
                    }
                }
            },
            _ => panic!("Don't know what is this image.")
        }
    }
    
    pub fn set_ideal_peaks(&mut self, mut n: u16) {
        let mut peak_locs: Vec<(usize, usize)> = vec![];
        while n > 0 {
            let c = random_range(..W);
            let r = random_range(..H);
            if peak_locs.contains(&(c, r)) {
                continue;
            }
            peak_locs.push((c, r));
            let elev = random_range(1..=MAX_ELEVATION);
            match self.basemap[r][c] {
                Pt::Undef => self.basemap[r][c] = Pt::Seed(elev),
                Pt::Water => (),
                x => panic!("Wrong cell contents: {:?}", x)
            }
            n -= 1;
        }
        self.peaks = peak_locs;
    }
}

impl<const W: usize, const H: usize> Display for HeightMapBuilder<W, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in self.basemap {
            write!(f, "[");
            for cell in row {
                write!(f, "{}", cell);
            }
            write!(f, "],\n");
        }
        Ok(())
    }
}

pub struct Crawler {
    dir: Dir,
    w: usize,
    h: usize,
    c: usize,
    r: usize,
}

impl Crawler {
    pub fn new(dir: Dir, w: usize, h: usize, c: usize, r: usize) -> Self {
        Crawler { dir, w, h, c, r }
    }

    pub fn next(&mut self) -> Option<(usize, usize)>  {
        let (next_c, next_r)= match self.dir {
            Dir::N  => {
                if self.r == 0 { return None }
                (self.c, self.r - 1)
            },
            Dir::NE => {
                if self.r == 0 || self.c == self.w - 1 { return None }
                (self.r - 1, self.c + 1)
            },
            Dir::E  => {
                if self.c == self.w - 1 { return None }
                (self.r, self.c + 1)
            },
            Dir::SE => {
                if self.r == self.h - 1 || self.c == self.w - 1 { return None }
                (self.r + 1, self.c + 1)
            },
            Dir::S  => {
                if self.r == self.h - 1 { return None }
                (self.r + 1, self.c)
            },
            Dir::SW => {
                if self.r == self.h - 1 || self.c == 0 { return None }
                (self.r + 1, self.c - 1)
                
            },
            Dir::W  => {
                if self.c == 0 { return None }
                (self.r, self.c - 1)
                
            },
            Dir::NW => {
                if self.r == 0 || self.c == 0 { return None }
                (self.r - 1, self.c - 1)
                
            },
        };
        self.c = next_c;
        self.r = next_r;

        Some((next_c, next_r))
    }
}

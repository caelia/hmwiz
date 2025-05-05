use rand::random_range;
use image::{GrayImage, Luma};
use noise::{Blend, Turbulence, RidgedMulti, Fbm, Perlin, MultiFractal};
use noise::utils::{NoiseMapBuilder, NoiseMap, PlaneMapBuilder};

use crate::common::THRESHOLD;


fn generate(width: usize, height: usize) -> NoiseMap {
    
    let perlin = Perlin::new(random_range(..u32::MAX));
    let ridged = RidgedMulti::<Perlin>::new(random_range(0..u32::MAX)).set_octaves(4);
    let fbm = Fbm::<Perlin>::default();
    let turbo = Turbulence::<_, Fbm<Perlin>>::new(fbm).set_roughness(2);
    let blend = Blend::new(perlin, ridged, turbo);

    PlaneMapBuilder::new(blend)
        .set_size(width, height)
        .set_is_seamless(true)
        .set_x_bounds(2.0, -2.0)
        .set_y_bounds(2.0, -2.0)
        .build()
}

pub fn run(projname: String, width: usize, height: usize) {
    let map= generate(width, height);

    let mut img = GrayImage::new(width as u32, height as u32);
    for r in 0..height{
        for c in 0..width{
            let val = map.get_value(c, r);

            if val > THRESHOLD {
                img.put_pixel(c as u32, r as u32, Luma([255]));
            } else {
                img.put_pixel(c as u32, r as u32, Luma([0]));
            }
        }
    }

    let _ = img.save(format!("{}-lf.png", projname));
}

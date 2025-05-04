#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::prelude::*;
use rand_distr::{Pert, Distribution};
use image::{GrayImage, GenericImage, ImageBuffer, Luma};
use noise::{Turbulence, Billow, BasicMulti, RidgedMulti, HybridMulti, Fbm, Worley, MultiFractal, Perlin};
use noise::utils::{NoiseMapBuilder, NoiseMap, PlaneMapBuilder};

const THRESHOLD: f64 = 0.1;

fn fbm_worley248() -> (NoiseMap, NoiseMap, NoiseMap) {
    let fbm1 = Fbm::<Worley>::new(0).set_octaves(2);
    let fbm2 = Fbm::<Worley>::new(0).set_octaves(4);
    let fbm3 = Fbm::<Worley>::new(0).set_octaves(8);
    (
    PlaneMapBuilder::new(fbm1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(fbm2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(fbm3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn bmf_perlin248() -> (NoiseMap, NoiseMap, NoiseMap) {
    let bmf1 = BasicMulti::<Perlin>::new(0).set_octaves(2);
    let bmf2 = BasicMulti::<Perlin>::new(0).set_octaves(4);
    let bmf3 = BasicMulti::<Perlin>::new(0).set_octaves(8);
    (
    PlaneMapBuilder::new(bmf1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(bmf2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(bmf3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn rmu_worley248() -> (NoiseMap, NoiseMap, NoiseMap) {
    let rmu1 = RidgedMulti::<Worley>::new(0).set_octaves(2);
    let rmu2 = RidgedMulti::<Worley>::new(0).set_octaves(4);
    let rmu3 = RidgedMulti::<Worley>::new(0).set_octaves(8);
    (
    PlaneMapBuilder::new(rmu1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(rmu2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(rmu3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn hm2_perlin248() -> (NoiseMap, NoiseMap, NoiseMap) {
    let hmp1 = HybridMulti::<HybridMulti::<Perlin>>::new(0).set_octaves(2);
    let hmp2 = HybridMulti::<HybridMulti::<Perlin>>::new(0).set_octaves(4);
    let hmp3 = HybridMulti::<HybridMulti::<Perlin>>::new(0).set_octaves(8);
    (
    PlaneMapBuilder::new(hmp1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(hmp2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(hmp3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn turbo_worley() -> (NoiseMap, NoiseMap, NoiseMap) {
    let fbm1 = Fbm::<Worley>::new(0).set_octaves(2);
    let fbm2 = Fbm::<Worley>::new(0).set_octaves(4);
    let fbm3 = Fbm::<Worley>::new(0).set_octaves(6);
    let turbo1 = Turbulence::<_, Fbm<Worley>>::new(fbm1).set_roughness(2);
    let turbo2 = Turbulence::<_, Fbm<Worley>>::new(fbm2).set_roughness(3);
    let turbo3 = Turbulence::<_, Fbm<Worley>>::new(fbm3).set_roughness(4);
    (
    PlaneMapBuilder::new(turbo1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(turbo2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(turbo3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn turbo_perlin() -> (NoiseMap, NoiseMap, NoiseMap) {
    // let fbm1 = Fbm::<Perlin>::new(0).set_octaves(2);
    // let fbm2 = Fbm::<Perlin>::new(0).set_octaves(4);
    // let fbm3 = Fbm::<Perlin>::new(0).set_octaves(6);
    let fbm1 = Fbm::<Perlin>::new(0).set_octaves(1);
    let fbm2 = Fbm::<Perlin>::new(0).set_octaves(3);
    let fbm3 = Fbm::<Perlin>::new(0).set_octaves(5);
    let turbo1 = Turbulence::<_, Fbm<Perlin>>::new(fbm1).set_roughness(2);
    let turbo2 = Turbulence::<_, Fbm<Perlin>>::new(fbm2).set_roughness(3);
    let turbo3 = Turbulence::<_, Fbm<Perlin>>::new(fbm3).set_roughness(4);
    (
    PlaneMapBuilder::new(turbo1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(turbo2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(turbo3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn mixed1() -> (NoiseMap, NoiseMap, NoiseMap) {
    let gen1 = Billow::<Worley>::new(0).set_octaves(2);
    let gen2 = Billow::<Perlin>::new(0).set_octaves(3);
    let gen3 = Fbm::<Perlin>::new(0).set_octaves(4);
    let turbo1 = Turbulence::<_, Billow<Worley>>::new(gen1).set_roughness(2);
    let turbo2 = Turbulence::<_, Billow<Perlin>>::new(gen2);
    // let turbo3 = Turbulence::<_, Fbm<Perlin>>::new(gen3);
    (
    PlaneMapBuilder::new(turbo1)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    PlaneMapBuilder::new(turbo2)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build(),
    // PlaneMapBuilder::new(turbo3)
    PlaneMapBuilder::new(gen3)
        .set_size(1024, 1024)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
    )
}

fn main() {
    // let (layer1, layer2, layer3) = fbm_worley248();
    // let (layer1, layer2, layer3) = bmf_perlin248();
    // let (layer1, layer2, layer3) = rmu_worley248();
    // let (layer1, layer2, layer3) = hm2_perlin248();
    // let (layer1, layer2, layer3) = turbo_worley();
    // let (layer1, layer2, layer3) = turbo_perlin();
    let (layer1, layer2, layer3) = mixed1();

    let mut img = GrayImage::new(1024, 1024);
    for r in 0..1024 {
        for c in 0..1024 {
            let val1 = layer1.get_value(c, r);
            let val2 = layer2.get_value(c, r);
            let val3 = layer3.get_value(c, r);
            
            let val = val1 * 0.45 + val2 * 0.33 + val3 * 0.22;

            if val > THRESHOLD {
                img.put_pixel(c as u32, r as u32, Luma([255]));
            } else {
                img.put_pixel(c as u32, r as u32, Luma([0]));
            }
        }
    }

    let _ = img.save("layers.png");
}

#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod common;
mod lf;
mod hm;

use common::*;


fn main() {
    let mut hmb = hm::HeightMapBuilder::<256, 256>::new();
    println!("Created hmb");
    hmb.load_lf("test-lf.png");
    println!("Loaded lf file");
    hmb.set_ideal_peaks(200);
    println!("Set peaks");
    println!("{}", hmb);
}

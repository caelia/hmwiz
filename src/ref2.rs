#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::prelude::*;
use rand_distr::{Pert, PertError};

use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::view::ContinuousView;
use plotlib::style::{LineStyle, LineJoin};

const LO_GRAV: f32 = 0.9;
const HI_GRAV: f32 = -0.4;
const MAX_SLOPE: f32 = 1.;

fn get_gravity() -> f32 {
    thread_rng().gen_range(HI_GRAV..=LO_GRAV)
}

fn scale_gravity(gravity: f32, start: usize, current: usize, end: usize) -> f32 {
    let dist_from_anchor = usize::min(current - start, end - current) as f32;
    let seg_len = (end - start) as f32;
    let mult = (dist_from_anchor / seg_len) * 2.;
    // println!("{}/{}/{}/{}/{}", gravity, start, current, end, mult);
    gravity * mult
}

fn gravitated_mode(lo: f32, hi: f32, ideal_height: f32, gravity: f32) -> f32 {
    if ideal_height <= lo {
        lo
    } else if ideal_height >= hi {
        hi
    } else {
        let delta = if gravity >= 0. {
            (lo - ideal_height) * gravity
        } else {
            (ideal_height - hi) * gravity
        };
        // println!("{}/{}/{}/{}/{}", lo, hi, ideal_height, gravity, delta);
        ideal_height + delta
    }
}

// Random highs & lows
fn test1() {
    let mut points: Vec<Option<f32>> = vec![None;1024];
    let mut trng = thread_rng();
    for _ in 0..10 {
        let x = trng.gen_range(1..1023);
        let y = trng.gen_range(0.0..=255.0);
        points[x] = Some(y);
    }
    points[0] = Some(0.);
    points[1023] = Some(0.);

    let mut gravity: f32;
    let mut seg_start = 0;
    let mut seg_end = 0;
    let mut goal_height = 0.;

    let limit = points.len();
    while seg_start < (limit - 1) {
        let start_height = match points[seg_start] {
            Some(h) => h,
            None => panic!("incorrect segment start: {}", seg_start),
        };
        let mut ok = false;
        // println!("START = {}", seg_start);
        for x in (seg_start + 1)..limit {
            match points[x] {
                Some(h) => {
                    // println!("h = {}", h);
                    seg_end = x;
                    goal_height = h;
                    ok = true;
                    break;
                },
                None => (),
            }            
        }
        if !ok {
            panic!("Something is WONG!!");
        }
        let mut height = start_height;
        let gravity = get_gravity();
        let mut trng = thread_rng();
        for x in seg_start..seg_end {
            let distance = (seg_end - x) as f32;
            let hdiff = goal_height - height;
            let ideal_slope = hdiff / distance;
            // let ideal_slope = (seg_end - x) as f32 / (goal_height - height);
            let lo = f32::max(height - MAX_SLOPE, 0.);
            let hi = f32::min(height + MAX_SLOPE, 255.);
            let local_gravity = scale_gravity(gravity, seg_start, x, seg_end);
            assert!(lo < hi);
            let ideal_height = height + ideal_slope;
            let mode = gravitated_mode(lo, hi, ideal_height, local_gravity);
            assert!(mode >= lo && mode <= hi);
            let pert = Pert::new(lo, hi, mode).unwrap();
            let new_height = pert.sample(&mut trng);
            points[x + 1] = Some(new_height);
            height = new_height;
        }
        // println!("start: {}, end: {}", seg_start, seg_end);
        seg_start = seg_end;
    }

    let mut heights = Vec::new();
    for p in points {
        match p {
            Some(h) => heights.push(h as f64),
            None => heights.push(-1.),
        }
    }
    // Trait bounds not satisfied for map
    /*
    let heights: Vec<f64> = points.map(|p| {
        match p {
            Some(h) => h as f64,
            None => -1.,
        }
    });
    */
    let exes = (0..1024).map(|x| x as f64).collect::<Vec<f64>>();
    assert!(heights.len() == exes.len() && !heights.contains(&-1.0));
    // Also doesn't work. AAARGH!
    // let data = *exes.iter().zip(heights).collect::<Vec<(f64, f64)>>();
    let mut data = Vec::new();
    for i in 0..1024 {
        data.push((exes[i], heights[i]));
    }
    
    let plot: Plot = Plot::new(data).line_style(
        LineStyle::new()
            .colour("#44EE22")
    );

    let v = ContinuousView::new()
        .add(plot)
        .x_range(0., 1024.)
        .y_range(0., 512.)
        .x_label("Distance")
        .y_label("Height");

    Page::single(&v).save("gravity1.svg").unwrap();
    // Page::single(&v).save("gravity.svg");
}

// Regularly space peaks of equal height
fn test2() {
    let mut points: Vec<Option<f32>> = vec![None;1024];
    let mut trng = thread_rng();

    points[0] = Some(0.);
    points[1023] = Some(0.);

    for idx in [260, 420, 580, 740] {
        points[idx] = Some(180.);
    }
    points[1000] = Some(0.);

    let mut gravity: f32;
    let mut seg_start = 0;
    let mut seg_end = 0;
    let mut goal_height = 0.;

    let limit = points.len();
    while seg_start < (limit - 1) {
        let start_height = match points[seg_start] {
            Some(h) => h,
            None => panic!("incorrect segment start: {}", seg_start),
        };
        let mut ok = false;
        // println!("START = {}", seg_start);
        for x in (seg_start + 1)..limit {
            match points[x] {
                Some(h) => {
                    // println!("h = {}", h);
                    seg_end = x;
                    goal_height = h;
                    ok = true;
                    break;
                },
                None => (),
            }            
        }
        if !ok {
            panic!("Something is WONG!!");
        }
        let mut height = start_height;
        let gravity = get_gravity();
        let mut trng = thread_rng();
        for x in seg_start..seg_end {
            let distance = (seg_end - x) as f32;
            let hdiff = goal_height - height;
            let ideal_slope = hdiff / distance;
            // let ideal_slope = (seg_end - x) as f32 / (goal_height - height);
            let lo = f32::max(height - MAX_SLOPE, 0.);
            let hi = f32::min(height + MAX_SLOPE, 255.);
            let local_gravity = scale_gravity(gravity, seg_start, x, seg_end);
            assert!(lo < hi);
            let ideal_height = height + ideal_slope;
            let mode = gravitated_mode(lo, hi, ideal_height, local_gravity);
            assert!(mode >= lo && mode <= hi);
            let pert = Pert::new(lo, hi, mode).unwrap();
            let new_height = pert.sample(&mut trng);
            points[x + 1] = Some(new_height);
            height = new_height;
        }
        // println!("start: {}, end: {}", seg_start, seg_end);
        seg_start = seg_end;
    }

    let mut heights = Vec::new();
    for p in points {
        match p {
            Some(h) => heights.push(h as f64),
            None => heights.push(-1.),
        }
    }
    // Trait bounds not satisfied for map
    /*
    let heights: Vec<f64> = points.map(|p| {
        match p {
            Some(h) => h as f64,
            None => -1.,
        }
    });
    */
    let exes = (0..1024).map(|x| x as f64).collect::<Vec<f64>>();
    assert!(heights.len() == exes.len() && !heights.contains(&-1.0));
    // Also doesn't work. AAARGH!
    // let data = *exes.iter().zip(heights).collect::<Vec<(f64, f64)>>();
    let mut data = Vec::new();
    for i in 0..1024 {
        data.push((exes[i], heights[i]));
    }
    
    let plot: Plot = Plot::new(data).line_style(
        LineStyle::new()
            .colour("#44EE22")
    );

    let v = ContinuousView::new()
        .add(plot)
        .x_range(0., 1024.)
        .y_range(0., 512.)
        .x_label("Distance")
        .y_label("Height");

    Page::single(&v).save("gravity2.svg").unwrap();
    // Page::single(&v).save("gravity.svg");
}

fn main() {
    test1();
    test2();
}
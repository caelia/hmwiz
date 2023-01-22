use std::collections::HashSet;
use rand::prelude::*;
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Point<T> {
    Fixed(T, bool),
    Goal(T),
    Empty,
}


const RANDOM_POINT_MAX_TRIES: usize = 10;

pub fn goal_points(config: Config) -> Vec<(usize, usize, f32)> {
    let rr = config.rows - 2 * config.margin_width + 2;
    let cc = config.cols - 2 * config.margin_width + 2;

    let r0 = config.pad_width;
    let rn = rr - config.pad_width;
    let c0 = config.pad_width;
    let cn = cc - config.pad_width;

    let mut trng = thread_rng();
    
    // Set goal point locations
    let mut hi_points = HashSet::new();
    let mut lo_points = HashSet::new();

    for i in 0..config.n_hi {
        let mut ok = false;
        for _ in 0..RANDOM_POINT_MAX_TRIES {
            let r = trng.gen_range(r0..rn);
            let c = trng.gen_range(c0..cn);
            if hi_points.insert((r, c)) {
                ok = true;
                break;
            }
        }
        if !ok {
            panic!("Unable to create sufficient goal points!");
        }
    }

    for i in 0..config.n_lo {
        let mut ok = false;
        for _ in 0..RANDOM_POINT_MAX_TRIES {
            let r = trng.gen_range(r0..rn);
            let c = trng.gen_range(c0..cn);
            if lo_points.insert((r, c)) {
                ok = true;
                break;
            }
        }
        if !ok {
            panic!("Unable to create sufficient goal points!");
        }
    }
    assert!(hi_points.is_disjoint(&lo_points));

    let mut all_points = Vec::new();

    for (r, c) in hi_points.iter() {
        let height = trng.gen_range(config.hi_min..=config.hi_max);
        all_points.push((*r, *c, height));
    }
    for (r, c) in lo_points.iter() {
        let height = trng.gen_range(config.lo_min..=config.lo_max);
        all_points.push((*r, *c, height));
    }

    all_points.sort_by(|(r1, c1, _), (r2, c2, _)| (r1, c1).partial_cmp(&(r2, c2)).unwrap());
    all_points
}

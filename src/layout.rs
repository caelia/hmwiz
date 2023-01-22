use std::collections::HashSet;
use rand::prelude::*;
use crate::config::Config;
use crate::grid::{Orientation, Grid};
use crate::envelope::{Envelope, ThreeDEnvelope};

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

    let rlimit = rr - 1;
    let climit = cc - 1;

    let mut trng = thread_rng();
    
    // Set goal point locations
    let mut hi_points = HashSet::new();
    let mut lo_points = HashSet::new();

    for i in 0..config.n_hi {
        let mut ok = false;
        for _ in 0..RANDOM_POINT_MAX_TRIES {
            let r = trng.gen_range(1..rlimit);
            let c = trng.gen_range(1..climit);
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
            let r = trng.gen_range(1..rlimit);
            let c = trng.gen_range(1..climit);
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

    // THESE VALUES SHOULD NOT BE HARDCODED - but it'll do for now.
    let envelope = ThreeDEnvelope::new(
        vec![
            (0, (config.margin_height, config.margin_height)),
            (255, (0., 255.)),
            (climit - 255, (0., 255.)),
            (climit, (config.margin_height, config.margin_height))
        ],
        vec![
            (0, (config.margin_height, config.margin_height)),
            (255, (0., 255.)),
            (rlimit - 255, (0., 255.)),
            (rlimit, (config.margin_height, config.margin_height))
        ]
    );
    for (r, c) in hi_points.iter() {
        let env_hmin, env_hmax = envelope.minmax_at_point((r, c));
        let min = (f32::max(env_hmin, config.hi_min));
        let max = (f32::min(env_hmax, config.hi_max));
        assert!(min <= max);
        let height = trng.gen_range(min..max);
        all_points.push((*r, *c, height));
    }
    for (r, c) in lo_points.iter() {
        let env_lmin, env_lmax = envelope.minmax_at_point((r, c));
        let min = (f32::max(env_lmin, config.lo_min));
        let max = (f32::min(env_lmax, config.lo_max));
        assert!(min <= max);
        let height = trng.gen_range(min..max);
        all_points.push((*r, *c, height));
    }

    all_points.sort_by(|(r1, c1, _), (r2, c2, _)| (r1, c1).partial_cmp(&(r2, c2)).unwrap());
    all_points
}

pub fn setup_grid(config: Config, goalpoints: Vec<(usize, usize, f32)>) -> Grid<Point> {
    let (rows, cols) = (config.rows, config.cols);
    let ori = if cols >= rows {
        Orientation::RowMajor
    } else {
        Orientation::ColMajor
    }
    let grid = Grid::new(rows, cols, ori, Point::Empty);
}

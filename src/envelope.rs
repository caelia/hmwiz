use std::collections::{BTreeMap, HashMap};

pub trait Envelope {
    type Point;
    fn minmax_at_point(&self, point: Self::Point) -> (f32, f32);
}

#[derive(Debug)]
pub struct TwoDEnvelope {
    guidepoints: BTreeMap<usize, (f32, f32)>,
    slopes: HashMap<(usize, usize), (f32, f32)>,
}

impl Envelope for TwoDEnvelope {
    type Point = usize;
    fn minmax_at_point(&self, point: Self::Point) -> (f32, f32) {
        let mut previous = 0;
        let mut minmax: (f32, f32);
        for (gp, limits) in self.guidepoints.iter() {
            if *gp == point {
                minmax = *limits
            } else if *gp > point {
                let slopes = match self.slopes.get(&(previous, *gp)) {
                    Some((min_slp, max_slp)) => {
                        let distance = point - previous as f32;
                        minmax = (distance * min_slp, distance * max_slp);
                    },
                    None => panic!("Invalid envelope index!"),
                };
            } else {
                previous = *gp;
            }
        }
        minmax
    }
}

impl TwoDEnvelope {
    pub fn new(mut points: Vec<(usize, (f32, f32))>) -> Self {
        assert!(points.len() >= 2);
        points.sort_by(|(i1, _), (i2, _)| i1.partial_cmp(i2).unwrap());
        let guidepoints = BTreeMap::new();
        let slopes = HashMap::new();
        for i in 0..(points.len() - 1) {
            let (loc1, (hmin1, hmax1)) = points[i];
            let (loc2, (hmin2, hmax2)) = points[i];
            let distance = loc2 - loc1;
            assert!(distance > 0);
        }
        TwoDEnvelope { guidepoints, slopes }
    }
}

#[derive(Debug)]
pub struct ThreeDEnvelope {
    h_env: TwoDEnvelope,
    v_env: TwoDEnvelope,
}

impl Envelope for ThreeDEnvelope {
    type Point = (usize, usize);
    fn minmax_at_point(&self, point: Self::Point) -> (f32, f32) {
        let (row, col) = point;
        let (hmin, hmax) = self.h_env.minmax_at_point(col);
        let (vmin, vmax) = self.v_env.minmax_at_point(row);
        let min = f32::max(hmin, vmin);
        let max = f32::min(hmax, vmax);
        assert!(min <= max);
        (min, max)
    }
}

impl ThreeDEnvelope {
    pub fn new(
            hpoints: Vec<(usize, (f32, f32))>,
            vpoints: Vec<(usize, (f32, f32))>)  -> Self {
        ThreeDEnvelope {
            h_env: TwoDEnvelope::new(hpoints),
            v_env: TwoDEnvelope::new(vpoints),
        }
    }
}

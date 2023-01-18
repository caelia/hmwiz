#[derive(Debug, Clone, Copy)]
pub struct Config {
    max_height: f32,
    min_height: f32,    
    max_slope: f32,
    size: (usize, usize),
    margin_width: usize,
    margin_height: f32,
    n_hi: usize,
    hi_min: f32,
    hi_max: f32,
    n_lo: usize,
    lo_min: f32,
    lo_max: f32,
}

impl Config {
    pub fn default(width: usize, height: usize,
               n_hi: usize, hi_min: f32, hi_max: Option<f32>,
               n_lo: usize, lo_min: Option<f32>, lo_max: f32)
                -> Self {
        Config {
            max_height: 255.,
            min_height: 0.,    
            max_slope: 1.,
            size: (height, width),
            margin_width: 16,
            margin_height: 0.,
            n_hi,
            hi_min,
            hi_max: hi_max.unwrap_or(255.),
            n_lo,
            lo_min: lo_min.unwrap_or(0.),
            lo_max,
        }
    }
}

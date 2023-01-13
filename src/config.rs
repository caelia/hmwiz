#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub max_height: f32,
    pub min_height: f32,
    pub max_slope: f32,
    pub rows: usize,
    pub cols: usize,
    pub margin_width: usize,
    pub margin_height: f32,
    pub pad_width: usize,
    pub n_hi: usize,
    pub hi_min: f32,
    pub hi_max: f32,
    pub n_lo: usize,
    pub lo_min: f32,
    pub lo_max: f32,
}

impl Config {
    pub fn default(
        rows: usize,
        cols: usize,
        n_hi: usize,
        hi_min: f32,
        hi_max: Option<f32>,
        n_lo: usize,
        lo_min: Option<f32>,
        lo_max: f32,
    ) -> Self {
        Config {
            max_height: 255.,
            min_height: 0.,
            max_slope: 1.,
            rows,
            cols,
            margin_width: 16,
            margin_height: 0.,
            pad_width: 4,
            n_hi,
            hi_min,
            hi_max: hi_max.unwrap_or(255.),
            n_lo,
            lo_min: lo_min.unwrap_or(0.),
            lo_max,
        }
    }

    // Gives layout size, which is total dimensions - margins
    // We add 2 because the edges need to overlap the inner edge
    // of the margin.
    pub fn layout_dimensions(&self) -> (usize, usize) {
        (
            self.rows - self.margin_width * 2 + 2,
            self.cols - self.margin_width * 2 + 2,
        )
    }

    // Gives minimum and maximum indices where non-edge points
    // can be set in the layout grid.
    pub fn active_limits(&self) -> (usize, usize, usize, usize) {
        let pw = self.pad_width;
        let (r, c) = self.layout_dimensions();
        (pw, r - pw, pw, c - pw)
    }
}

use clap::Parser;
use rand::prelude::*;
use rand_distr::{Pert, Distribution};

#[derive(Parser, Debug)]
#[command(name = "hmwiz", disable_help_flag = true)]
pub struct Config {
    #[arg(short = 'w', long, default_value_t = 1024)]
    pub width: usize,
    #[arg(short = 'h', long, default_value_t = 1024)]
    pub height: usize,
    #[arg(long, default_value_t = 0)]
    pub seed: u32,
    #[arg(long, default_value_t = 1.0)]
    pub continent_frequency: f64,
    #[arg(long, default_value_t = 2.209)]
    pub continent_lacunarity: f64,
    #[arg(long, default_value_t = 2.143)]
    pub mountain_lacunarity: f64,
    #[arg(long, default_value_t = 2.162)]
    pub hills_lacunarity: f64,
    #[arg(long, default_value_t = 2.314)]
    pub plains_lacunarity: f64,
    #[arg(long, default_value_t = 2.213)]
    pub badlands_lacunarity: f64,
    #[arg(long, default_value_t = 1.0)]
    pub mountains_twist: f64,
    #[arg(long, default_value_t = 1.0)]
    pub hills_twist: f64,
    #[arg(long, default_value_t = 1.0)]
    pub badlands_twist: f64,
    #[arg(long, default_value_t = 0.0)]
    pub sea_level: f64,
    #[arg(long, default_value_t = -0.375)]
    pub shelf_level: f64,
    #[arg(long, default_value_t = 0.5)]
    pub mountains_amount: f64,
    #[arg(long)]
    pub hills_amount: Option<f64>,
    #[arg(long, default_value_t = 0.313)]
    pub badlands_amount: f64,
    #[arg(long, default_value_t = 1.0)]
    pub terrain_offset: f64,
    #[arg(long, default_value_t = 1.375)]
    pub mountain_glaciation: f64,
    #[arg(long)]
    pub continent_height_scale: Option<f64>,
    #[arg(long, default_value_t = 0.023)]
    pub river_depth: f64,
    #[arg(short, long)]
    pub random: Vec<String>
}

pub fn get_config() -> Config {
    let mut cfg = Config::parse();    
    let mut all_random = false;

    fn rand_val(lo: f64, hi: f64, mode: f64) -> f64 {
        let pert = Pert::new(lo, hi, mode).unwrap();
        pert.sample(&mut thread_rng())
    }
    let random_contains = |s: &str| {
        cfg.random.contains(&s.to_string())
    };

    if cfg.random.len() == 1 && cfg.random[0] == "all" {
        all_random = true;
    }
    
    if all_random || random_contains("seed") {
        let mut rng = thread_rng();
        cfg.seed = rng.gen_range(0..=u32::MAX);
    }
    if all_random || random_contains("continent_frequency") {
        cfg.continent_frequency = rand_val(0.0, 5.0, 1.0);
    }
    if all_random || random_contains("continent_lacunarity") {
        cfg.continent_lacunarity = rand_val(1.6, 2.4, 2.0);
    }
    if all_random || random_contains("mountain_lacunarity") {
        cfg.mountain_lacunarity = rand_val(1.6, 2.4, 2.0);
    }
    if all_random || random_contains("hills_lacunarity") {
        cfg.hills_lacunarity = rand_val(1.6, 2.4, 2.0);
    }
    if all_random || random_contains("plains_lacunarity") {
        cfg.plains_lacunarity = rand_val(1.6, 2.4, 2.0);
    }
    if all_random || random_contains("badlands_lacunarity") {
        cfg.badlands_lacunarity = rand_val(1.6, 2.4, 2.0);
    }
    if all_random || random_contains("mountains_twist") {
        cfg.mountains_twist = rand_val(0.0, 5.0, 1.0);
    }
    if all_random || random_contains("hills_twist") {
        cfg.hills_twist = rand_val(0.0, 5.0, 1.0);
    }
    if all_random || random_contains("badlands_twist") {
        cfg.badlands_twist = rand_val(0.0, 5.0, 1.0);
    }
    if all_random || random_contains("sea_level") {
        cfg.sea_level = rand_val(-1.0, 1.0, 0.0);        
    }
    if all_random || random_contains("shelf_level") {
        let mode = f64::max(-1.0, cfg.sea_level - 0.1);
        cfg.shelf_level = rand_val(-1.0, cfg.sea_level, mode);
    }
    if all_random || random_contains("mountains_amount") {
        cfg.mountains_amount = rand_val(0.0, 1.0, 0.5);
    }
    if all_random || random_contains("hills_amount") {
        let lo = 0.0;
        let hi = cfg.mountains_amount - f64::EPSILON;
        let mode = f64::min(hi, (1.0 + cfg.mountains_amount) / 2.0);
        cfg.hills_amount = Some(rand_val(lo, hi, mode));
    }
    if all_random || random_contains("badlands_amount") {
        cfg.badlands_amount = rand_val(0.0, 1.0, 0.3125);
    }
    if all_random || random_contains("terrain_offset") {
        cfg.terrain_offset = rand_val(0.0, 4.0, 1.0);
    }
    if all_random || random_contains("mountain_glaciation") {
        let lo = 1.0 + f64::EPSILON;
        cfg.mountain_glaciation = rand_val(lo, 1.5, 1.2);
    }
    if all_random || random_contains("continent_height_scale") {
        let mode = (1.0 - cfg.sea_level) / 4.0;
        cfg.continent_height_scale = Some(rand_val(0.0, 1.0, mode));
    }
    if all_random || random_contains("river_depth") {
        cfg.river_depth = rand_val(0.0, 0.1, 0.023);
    }

    match cfg.hills_amount {
        Some(_) => (),
        None => cfg.hills_amount = Some((cfg.mountains_amount + 1.0) / 2.0),
    }

    match cfg.continent_height_scale {
        Some(_) => (),
        None => cfg.continent_height_scale = Some((1.0 - cfg.sea_level) / 4.0),
    }

    cfg
}

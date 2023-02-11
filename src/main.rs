mod cli;
mod mapgen;

use cli::{Config, get_config};
use mapgen::generate_map;


fn main() {
    let config: Config = get_config();
    generate_map(&config);
}
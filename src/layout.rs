use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Point<T> {
    Fixed(T, bool),
    Goal(T),
    Empty,
}

pub fn random_points(config: Config) -> Vec<(usize, usize, f32)> {}

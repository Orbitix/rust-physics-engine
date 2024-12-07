use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ball_count_2d: usize,
    pub ball_count_3d: usize,
    pub ball_radius: f32,
    pub gravity: f32,
    pub resistance: f32,
    pub bounce_amount: f32,
    pub max_speed: f32,
    pub max_pressure: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub sim_steps: i32,
    pub auto_sim_steps: bool,
    pub target_fps: i32,
    pub fps_boundary: i32,
    pub delete_dist: f32,
}

pub fn load_config(path: &str) -> Config {
    let config_content = fs::read_to_string(path).expect("Failed to read configuration file");

    toml::from_str(&config_content).expect("Failed to parse configuration file")
}

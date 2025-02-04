use rust_physics_engine::common;
mod spatial_hash;

use common::config::load_config;
use common::fps_counter::SmoothedFps;
use spatial_hash::SpatialHash;

use partial_borrow::prelude::*;

use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
struct Ball {
    id: usize,
    position: Vec2,
    velocity: Vec2,
    pressure: f32,
    color: Color,
    radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
    Normal,
    Velocity,
    Pressure,
}

struct State {
    display_mode: DisplayMode,
}

impl State {
    fn new() -> Self {
        State {
            display_mode: DisplayMode::Normal,
        }
    }

    fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Normal => DisplayMode::Velocity,
            DisplayMode::Velocity => DisplayMode::Pressure,
            DisplayMode::Pressure => DisplayMode::Normal,
        };
    }
}

fn get_color_from_vel(ball: Ball, largest_speed: f32) -> Color {
    let vel = ball.velocity;
    let speed = vel.length();

    let normalised_speed = speed / largest_speed;

    Color {
        r: (0.0),
        g: (normalised_speed),
        b: (1.0 - normalised_speed),
        a: (1.0),
    }
}

fn get_color_from_pressure(ball: Ball, largest_pressure: f32) -> Color {
    let pressure = ball.pressure;

    let mut normalised_pressure = 0.0;

    if largest_pressure != 0.0 {
        normalised_pressure = pressure / largest_pressure;
    }

    Color {
        r: (normalised_pressure),
        g: (0.0),
        b: (1.0 - normalised_pressure),
        a: (1.0),
    }
}

fn is_colliding(ball: &Ball, otherball: &Ball) -> bool {
    let dist = ball.position.distance(otherball.position);

    dist < ball.radius + otherball.radius
}

fn resolve_collision(ball: &mut Ball, otherball: &mut Ball, bounce_amount: f32, max_pressure: f32) {
    let mut pdiff = otherball.position - ball.position;

    let dist = ball.position.distance(otherball.position);

    let overlap = (ball.radius + otherball.radius) - dist;

    if overlap < 0.001 {
        return;
    }

    pdiff /= dist;

    ball.position -= pdiff * overlap / 2.0;
    otherball.position += pdiff * overlap / 2.0;

    let relative_velocity = otherball.velocity - ball.velocity;
    let dot_product = relative_velocity.dot(pdiff);

    if dot_product > 0.0 {
        return;
    }

    let force = dot_product * bounce_amount;

    ball.pressure = (ball.pressure + -force / (std::f32::consts::PI * ball.radius * ball.radius))
        .min(1.0)
        .max(0.0);
    otherball.pressure = (otherball.pressure
        + -force / (std::f32::consts::PI * otherball.radius * otherball.radius))
        .min(1.0)
        .max(0.0);

    ball.velocity += pdiff * force;
    otherball.velocity -= pdiff * force;
}

fn resolve_boundaries(ball: &mut Ball, screen_width: f32, screen_height: f32, bounce_amount: f32) {
    if ball.position.x - ball.radius < 0.0 {
        ball.position.x = ball.radius;
        if ball.velocity.x < 0.0 {
            ball.velocity.x *= -bounce_amount;
        }
    } else if ball.position.x + ball.radius > screen_width {
        ball.position.x = screen_width - ball.radius;
        if ball.velocity.x > 0.0 {
            ball.velocity.x *= -bounce_amount;
        }
    }

    if ball.position.y - ball.radius < 0.0 {
        ball.position.y = ball.radius;
        if ball.velocity.y < 0.0 {
            ball.velocity.y *= -bounce_amount;
        }
    } else if ball.position.y + ball.radius > screen_height {
        ball.position.y = screen_height - ball.radius;
        if ball.velocity.y > 0.0 {
            ball.velocity.y *= -bounce_amount;
        }
    }
}

#[macroquad::main("Physics Sim")]
// #[cfg(feature = "version_2d")]
async fn main() {
    let config = load_config("config.toml");

    let ball_count = config.ball_count_2d;
    let ball_radius = config.ball_radius;
    let gravity = config.gravity;
    let resistance = config.resistance;
    let bounce_amount = config.bounce_amount;
    let max_speed = config.max_speed;
    let max_pressure = config.max_pressure;
    let width = config.width;
    let height = config.height;
    let mut sim_steps = config.sim_steps;
    let auto_sim_steps = config.auto_sim_steps;
    let target_fps = config.target_fps;
    let fps_boundary = config.fps_boundary;
    let delete_dist = config.delete_dist;

    request_new_screen_size(width, height);

    let mut smoothed_fps = SmoothedFps::new();

    let mut colors: Vec<Color> = (0..ball_count)
        .map(|_| {
            Color::new(
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                1.0,
            )
        })
        .collect();

    let mut balls: Vec<Ball> = (0..ball_count)
        .enumerate()
        .map(|(id, _)| Ball {
            id,
            position: vec2(
                rand::gen_range(ball_radius, width - ball_radius),
                rand::gen_range(ball_radius, height - ball_radius),
            ),
            velocity: vec2(
                rand::gen_range(-100.0, 100.0),
                rand::gen_range(-100.0, 100.0),
            ),
            pressure: 0.0,
            color: colors[id],
            radius: ball_radius,
        })
        .collect();

    let mut spatial_hash: SpatialHash<usize> = SpatialHash::new((ball_radius * 2.0) + 2.0);

    let mut do_gravity = true;

    let mut display_state = State::new();

    loop {
        clear_background(BLACK);

        let mut largest_speed: f32 = 0.0;
        let mut largest_pressure: f32 = 0.0;

        let mouse_position: Vec2 = mouse_position().into();

        let screen_width = screen_width();
        let screen_height = screen_height();

        spatial_hash.clear();

        if is_mouse_button_down(MouseButton::Right) {
            let color = Color::new(
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                1.0,
            );

            let new_ball: Ball = Ball {
                id: balls.len(),
                position: mouse_position,
                velocity: vec2(
                    rand::gen_range(-100.0, 100.0),
                    rand::gen_range(-100.0, 100.0),
                ),
                color,
                pressure: 0.0,
                radius: ball_radius,
            };

            balls.push(new_ball);
            colors.push(color);
        }

        for ball in balls.iter() {
            spatial_hash.insert(ball.position, ball.id);

            if display_state.display_mode == DisplayMode::Velocity {
                if ball.velocity.length() > largest_speed {
                    largest_speed = ball.velocity.length();
                }
            }

            if display_state.display_mode == DisplayMode::Pressure {
                if ball.pressure > largest_pressure {
                    largest_pressure = ball.pressure;
                }
            }
        }

        for _ in 0..sim_steps {
            for i in 0..balls.len() {
                for &other_ball_id in spatial_hash.get_nearby_objects(balls[i].position, i).iter() {
                    if i != other_ball_id {
                        // Use index to get mutable references
                        let (ball, other_ball) = if i < other_ball_id {
                            let (left, right) = balls.split_at_mut(other_ball_id);
                            (&mut left[i], &mut right[0])
                        } else {
                            let (left, right) = balls.split_at_mut(i);
                            (&mut right[0], &mut left[other_ball_id])
                        };

                        if is_colliding(ball, other_ball) {
                            resolve_collision(ball, other_ball, bounce_amount, max_pressure);
                        } else {
                            ball.pressure = 0.0;
                            other_ball.pressure = 0.0;
                        }
                    }
                }
                resolve_boundaries(&mut balls[i], screen_width, screen_height, bounce_amount);
            }
        }

        let delta_time = get_frame_time();
        let mut rate = delta_time;

        if rate < 0.0 {
            rate = 0.01
        }

        let mouse_pressed = is_mouse_button_down(MouseButton::Left);

        if is_key_pressed(KeyCode::Space) {
            do_gravity = !do_gravity
        }

        if is_key_pressed(KeyCode::D) {
            display_state.toggle_display_mode();
        }

        for ball in balls.iter_mut() {
            if mouse_pressed {
                let mut force = mouse_position - ball.position;

                let distance = force.length();
                if distance < 0.1 {
                    force /= distance;
                }

                let attraction_strength = gravity;
                ball.velocity += force * attraction_strength * rate;
            }

            if do_gravity {
                ball.velocity.y += gravity;
            }

            match display_state.display_mode {
                DisplayMode::Normal => ball.color = colors[ball.id],
                DisplayMode::Velocity => {
                    ball.color = get_color_from_vel(*ball, largest_speed);
                }
                DisplayMode::Pressure => {
                    ball.color = get_color_from_pressure(*ball, largest_pressure);
                }
            }

            ball.velocity.x *= resistance;
            ball.velocity.y *= resistance;

            ball.velocity = ball.velocity.clamp_length_max(max_speed);

            ball.position += ball.velocity * rate;

            draw_circle(ball.position.x, ball.position.y, ball.radius, ball.color)
        }

        if is_key_down(KeyCode::F) {
            let mut to_remove: Vec<usize> = Vec::new();

            for (index, ball) in balls.iter().enumerate() {
                let dist = ball.position.distance(mouse_position);

                if dist < delete_dist {
                    to_remove.push(index);
                }
            }

            to_remove.sort_unstable_by(|a, b| b.cmp(a));
            for idx in to_remove {
                balls.remove(idx);
                colors.remove(idx);
            }

            for (idx, ball) in balls.iter_mut().enumerate() {
                ball.id = idx;
                colors[idx] = ball.color;
            }
        }

        let fps = get_fps();
        smoothed_fps.update(fps as f32);

        let avg_fps = smoothed_fps.get_average();

        draw_text(&format!("FPS: {:.2}", avg_fps), 10.0, 20.0, 30.0, WHITE);

        if auto_sim_steps {
            if fps < target_fps {
                sim_steps -= 1;
            } else if fps > (target_fps + fps_boundary) {
                sim_steps += 1;
            }
        } else {
            if is_key_pressed(KeyCode::Up) {
                sim_steps += 1;
            } else if is_key_pressed(KeyCode::Down) {
                sim_steps -= 1;
            }
        }

        sim_steps = sim_steps.clamp(1, 200);
        // sim_steps = (sim_steps as f32 + 0.1 * (target_sim_steps as f32 - sim_steps as f32)) as i32;

        draw_text(
            &format!("SIM STEPS: {}", sim_steps),
            10.0,
            50.0,
            30.0,
            WHITE,
        );

        draw_text(&format!("BALLS: {}", balls.len()), 10.0, 80.0, 30.0, WHITE);

        next_frame().await
    }
}

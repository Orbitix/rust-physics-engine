mod spatial_hash;

use spatial_hash::SpatialHash;

use partial_borrow::prelude::*;

use macroquad::prelude::*;

const BALL_COUNT: usize = 200;
const BALL_RADIUS: f32 = 10.0;
const GRAVITY: f32 = 9.81;

const UPDATE_RATE: f32 = 1.0 / 60.0;

const RESISTANCE: f32 = 0.999;
const BOUNCE_AMOUNT: f32 = 0.6;

const MAX_SPEED: f32 = 2000.0;
const MAX_PRESSURE: f32 = 1000.0;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 800.0;

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

fn get_color_from_vel(ball: Ball, max_speed: f32) -> Color {
    let vel = ball.velocity;
    let speed = vel.length();

    let normalised_speed = speed / max_speed;

    Color {
        r: (0.0),
        g: (normalised_speed),
        b: (1.0 - normalised_speed),
        a: (1.0),
    }
}

fn get_color_from_pressure(ball: Ball, max_pressure: f32) -> Color {
    let pressure = ball.pressure;

    let mut normalised_pressure = 0.0;

    if max_pressure != 0.0 {
        normalised_pressure = pressure / max_pressure;
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

fn resolve_collision(ball: &mut Ball, otherball: &mut Ball) {
    let mut pdiff = otherball.position - ball.position;

    let dist = ball.position.distance(otherball.position);

    let overlap = (ball.radius + otherball.radius) - dist;

    if overlap < 0.001 {
        return;
    }

    pdiff /= dist;

    ball.position -= pdiff * overlap / 2.0;
    otherball.position += pdiff * overlap / 2.0;

    let vdiff = otherball.velocity - ball.velocity;

    let dot_product = vdiff.x * pdiff.x + vdiff.y * pdiff.y;

    if dot_product > 0.0 {
        return;
    }

    let restitution = 1.0 - BOUNCE_AMOUNT;

    let force = dot_product * restitution;

    let area = std::f32::consts::PI * ball.radius * ball.radius;
    let other_area = std::f32::consts::PI * otherball.radius * otherball.radius;

    ball.pressure = -force / area;
    otherball.pressure = -force / other_area;

    ball.pressure = ball.pressure.min(MAX_PRESSURE);
    otherball.pressure = otherball.pressure.min(MAX_PRESSURE);

    ball.velocity += force * pdiff;
    otherball.velocity -= force * pdiff;
}

#[macroquad::main("Physics Sim")]
async fn main() {
    request_new_screen_size(WIDTH, HEIGHT);

    let colors: Vec<Color> = (0..BALL_COUNT)
        .map(|_| {
            Color::new(
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                1.0,
            )
        })
        .collect();

    let mut balls: Vec<Ball> = (0..BALL_COUNT)
        .enumerate()
        .map(|(id, _)| Ball {
            id,
            position: vec2(
                rand::gen_range(BALL_RADIUS, WIDTH - BALL_RADIUS),
                rand::gen_range(BALL_RADIUS, HEIGHT - BALL_RADIUS),
            ),
            velocity: vec2(
                rand::gen_range(-100.0, 100.0),
                rand::gen_range(-100.0, 100.0),
            ),
            pressure: 0.0,
            color: colors[id],
            radius: BALL_RADIUS,
        })
        .collect();

    let mut spatial_hash: SpatialHash<usize> = SpatialHash::new((BALL_RADIUS * 2.0) + 2.0);

    let mut do_gravity = true;

    let mut display_state = State::new();

    loop {
        clear_background(BLACK);

        let mut max_speed: f32 = 0.0;
        let mut max_pressure: f32 = 0.0;

        spatial_hash.clear();

        for ball in balls.iter() {
            spatial_hash.insert(ball.position, ball.id);

            if display_state.display_mode == DisplayMode::Velocity {
                if ball.velocity.length() > max_speed {
                    max_speed = ball.velocity.length();
                }
            }

            if display_state.display_mode == DisplayMode::Pressure {
                if ball.pressure > max_pressure {
                    max_pressure = ball.pressure;
                }
            }
        }

        for i in 0..balls.len() {
            for &other_ball_id in spatial_hash.get_nearby_objects(balls[i].position).iter() {
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
                        resolve_collision(ball, other_ball);
                    } else {
                        ball.pressure = 0.0;
                        other_ball.pressure = 0.0;
                    }
                }
            }
        }

        let delta_time = get_frame_time();
        let mut rate = delta_time;

        if rate < 0.0 {
            rate = 0.01
        }

        let mouse_position: Vec2 = mouse_position().into();

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

                let attraction_strength = 0.5 * GRAVITY;
                ball.velocity += force * attraction_strength * rate;
            }

            if do_gravity {
                ball.velocity.y += GRAVITY;
            }

            match display_state.display_mode {
                DisplayMode::Normal => ball.color = colors[ball.id],
                DisplayMode::Velocity => {
                    ball.color = get_color_from_vel(*ball, max_speed);
                }
                DisplayMode::Pressure => {
                    ball.color = get_color_from_pressure(*ball, max_pressure);
                }
            }

            ball.velocity.x *= RESISTANCE;
            ball.velocity.y *= RESISTANCE;

            ball.velocity = ball.velocity.clamp_length_max(MAX_SPEED);

            ball.position += ball.velocity * rate;

            if ball.position.x - ball.radius < 0.0 {
                ball.position.x = ball.radius;
                if ball.velocity.x < 0.0 {
                    ball.velocity.x *= -BOUNCE_AMOUNT;
                }
            }
            if ball.position.x + ball.radius > screen_width() {
                ball.position.x = screen_width() - ball.radius;
                if ball.velocity.x > 0.0 {
                    ball.velocity.x *= -BOUNCE_AMOUNT;
                }
            }

            if ball.position.y - ball.radius < 0.0 {
                ball.position.y = ball.radius;
                if ball.velocity.y < 0.0 {
                    ball.velocity.y *= -BOUNCE_AMOUNT;
                }
            }
            if ball.position.y + ball.radius > screen_height() {
                ball.position.y = screen_height() - ball.radius;
                if ball.velocity.y > 0.0 {
                    ball.velocity.y *= -BOUNCE_AMOUNT;
                }
            }

            draw_circle(ball.position.x, ball.position.y, ball.radius, ball.color)
        }

        next_frame().await
    }
}

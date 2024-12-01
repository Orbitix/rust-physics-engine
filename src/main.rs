mod spatial_hash;

use spatial_hash::SpatialHash;

use partial_borrow::prelude::*;

use macroquad::prelude::*;

const BALL_COUNT: usize = 500;
const BALL_RADIUS: f32 = 10.0;
const GRAVITY: f32 = 9.81;

const UPDATE_RATE: f32 = 1.0 / 60.0;

const RESISTANCE: f32 = 0.999;
const BOUNCE_AMOUNT: f32 = 0.6;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 800.0;

#[derive(Debug, Clone, Copy)]
struct Ball {
    id: usize,
    position: Vec2,
    velocity: Vec2,
    color: Color,
    radius: f32,
}

fn is_colliding(ball: &Ball, otherball: &Ball) -> bool {
    let dist = ball.position.distance(otherball.position);

    dist < ball.radius + otherball.radius
}

fn resolve_collision(ball: &mut Ball, otherball: &mut Ball) {
    let mut pdiff = otherball.position - ball.position;

    let dist = ball.position.distance(otherball.position);

    pdiff /= dist;

    let overlap = (ball.radius + otherball.radius) - dist;

    if overlap < 0.001 {
        return;
    }

    ball.position -= pdiff * overlap / 2.0;
    otherball.position += pdiff * overlap / 2.0;

    let vdiff = otherball.velocity - ball.velocity;

    let dot_product = vdiff.x * pdiff.x + vdiff.y * pdiff.y;

    if dot_product > 0.0 {
        return;
    }

    // let impulse = 2.0 * dot_product / (ball.radius + otherball.radius);

    let restitution = 1.0 - BOUNCE_AMOUNT;

    ball.velocity += dot_product * pdiff * restitution;
    otherball.velocity -= dot_product * pdiff * restitution;
}

#[macroquad::main("Physics Sim")]
async fn main() {
    request_new_screen_size(WIDTH, HEIGHT);

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
            color: Color::new(
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                rand::gen_range(0.0, 1.0),
                1.0,
            ),
            radius: BALL_RADIUS,
        })
        .collect();

    let mut spatial_hash: SpatialHash<usize> = SpatialHash::new((BALL_RADIUS * 2.0) + 2.0);

    let mut do_gravity = true;

    loop {
        clear_background(BLACK);

        spatial_hash.clear();

        for ball in balls.iter() {
            spatial_hash.insert(ball.position, ball.id);
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

            ball.velocity.x *= RESISTANCE;
            ball.velocity.y *= RESISTANCE;

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

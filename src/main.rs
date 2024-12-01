mod spatial_hash;
use spatial_hash::SpatialHash;

use partial_borrow::prelude::*;

use macroquad::prelude::*;

const BALL_COUNT: usize = 400;
const BALL_RADIUS: f32 = 5.0;
const GRAVITY: f32 = 0.1;

const RESISTANCE: f32 = 0.999;
const BOUNCE_AMOUNT: f32 = 0.6;

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 600.0;

#[derive(Debug, Clone, Copy)]
struct Ball {
    id: usize,
    position: Vec2,
    velocity: Vec2,
    color: Color,
    radius: f32,
}

async fn is_colliding(ball: &Ball, otherball: &Ball) -> bool {
    let dist = ball.position.distance(otherball.position);

    if dist < ball.radius + otherball.radius {
        // collision
        return true;
    }

    return false;
}

async fn resolve_collision(ball: &mut Ball, otherball: &Ball) {
    let mut pdiff = otherball.position - ball.position;

    let dist = ball.position.distance(otherball.position);

    pdiff /= dist;

    let overlap = (ball.radius + otherball.radius) - dist;

    if overlap < 0.001 {
        return;
    }

    // ball.position -= pdiff * overlap / 2.0;
    ball.position -= pdiff * overlap;
    // otherball.position += pdiff * overlap / 2.0;

    let vdiff = otherball.velocity - ball.velocity;

    let dot_product = vdiff.x * pdiff.x + vdiff.y * pdiff.y;

    if dot_product > 0.0 {
        return;
    }

    let impulse = 2.0 * dot_product / (ball.radius + otherball.radius);

    let restitution = 1.0 - BOUNCE_AMOUNT;

    ball.velocity += impulse * pdiff * restitution;
    // otherball.velocity -= impulse * pdiff * restitution;
}

#[macroquad::main("Physics Sim")]
async fn main() {
    request_new_screen_size(WIDTH, HEIGHT);

    let mut balls: Vec<Ball> = (0..BALL_COUNT)
        .enumerate()
        .map(|(id, _)| Ball {
            id,
            position: vec2(
                rand::gen_range(BALL_RADIUS, screen_width() - BALL_RADIUS),
                rand::gen_range(BALL_RADIUS, screen_height() - BALL_RADIUS),
            ),
            velocity: vec2(rand::gen_range(-2.0, 2.0), rand::gen_range(-2.0, 2.0)),
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

    loop {
        clear_background(BLACK);

        for ball in balls.iter() {
            spatial_hash.insert(ball.position, ball.id);
        }

        let ball_copy = balls.clone();

        for ball in balls.iter_mut() {
            let nearby_ball_ids = spatial_hash.get_nearby_objects(ball.position);

            for other_ball_id in nearby_ball_ids.iter() {
                if let Some(other_ball) = ball_copy.get(*other_ball_id) {
                    if is_colliding(ball, other_ball).await {
                        resolve_collision(ball, other_ball).await;
                    }
                }
            }
        }

        for ball in balls.iter_mut() {
            ball.velocity.y += GRAVITY;

            ball.velocity.x *= RESISTANCE;
            ball.velocity.y *= RESISTANCE;

            ball.position += ball.velocity;

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

        spatial_hash.clear();
        next_frame().await
    }
}

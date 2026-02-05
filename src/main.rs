mod config;
mod spatial_hash;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::sprite::Anchor;
use rand::random;
use config::{load_config, Config};
use spatial_hash::SpatialHash as SpatialHashInner;

#[derive(Resource)]
struct SpatialHash<ID>(SpatialHashInner<ID>);

#[derive(Debug, Clone, Copy, Component)]
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

const SPATIAL_HASH_PADDING: f32 = 2.0;
const MIN_DELTA_TIME: f32 = 0.01;

#[derive(Resource)]
struct DisplayState {
    display_mode: DisplayMode,
}

impl DisplayState {
    fn new() -> Self {
        DisplayState {
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

#[derive(Resource)]
struct SimulationConfig {
    gravity: f32,
    resistance: f32,
    bounce_amount: f32,
    max_speed: f32,
    max_pressure: f32,
}

#[derive(Resource)]
struct SimulationState {
    sim_steps: i32,
    do_gravity: bool,
}

impl SimulationState {
    fn new(sim_steps: i32) -> Self {
        Self {
            sim_steps,
            do_gravity: true,
        }
    }
}

#[derive(Resource)]
struct BallColors(Vec<Color>);

#[derive(Resource)]
struct BallMesh(Handle<Mesh>);

#[derive(Resource)]
struct UiState {
    fps_entity: Entity,
    sim_steps_entity: Entity,
    balls_entity: Entity,
}

#[derive(Resource, Default)]
struct MetricsState {
    fps: f32,
}

fn main() {
    let config = load_config("config.toml");
    let display_state = DisplayState::new();
    let simulation_state = SimulationState::new(config.sim_steps);
    let simulation_config = SimulationConfig {
        gravity: config.gravity,
        resistance: config.resistance,
        bounce_amount: config.bounce_amount,
        max_speed: config.max_speed,
        max_pressure: config.max_pressure,
    };

    let window_width = config.width;
    let window_height = config.height;
    let spatial_hash_cell_size = (config.ball_radius * 2.0) + SPATIAL_HASH_PADDING;

    App::new()
        .insert_resource(config)
        .insert_resource(display_state)
        .insert_resource(simulation_state)
        .insert_resource(simulation_config)
        .insert_resource(SpatialHash(SpatialHashInner::<Entity>::new(
            spatial_hash_cell_size,
        )))
        .init_resource::<MetricsState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(window_width, window_height)
                    .with_scale_factor_override(1.0),
                title: "Physics Sim".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_inputs,
                spawn_ball_on_click,
                delete_balls_on_key,
                update_spatial_hash,
                simulate,
                apply_motion,
                update_visuals,
                update_fps,
                update_sim_steps,
                update_ui,
                capture_screenshot,
            )
                .chain(),
        )
        .run();
}

fn get_color_from_vel(ball: Ball, largest_speed: f32) -> Color {
    let vel = ball.velocity;
    let speed = vel.length();

    let normalized_speed = if largest_speed > 0.0 {
        speed / largest_speed
    } else {
        0.0
    };

    Color::srgb(0.0, normalized_speed, 1.0 - normalized_speed)
}

fn get_color_from_pressure(ball: Ball, largest_pressure: f32) -> Color {
    let pressure = ball.pressure;

    let mut normalized_pressure = 0.0;

    if largest_pressure != 0.0 {
        normalized_pressure = pressure / largest_pressure;
    }

    Color::srgb(normalized_pressure, 0.0, 1.0 - normalized_pressure)
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

    let vdiff = otherball.velocity - ball.velocity;

    let dot_product = vdiff.x * pdiff.x + vdiff.y * pdiff.y;

    if dot_product > 0.0 {
        return;
    }

    let restitution = 1.0 - bounce_amount;

    let force = dot_product * restitution;

    let area = std::f32::consts::PI * ball.radius * ball.radius;
    let other_area = std::f32::consts::PI * otherball.radius * otherball.radius;

    ball.pressure = -force / area;
    otherball.pressure = -force / other_area;

    ball.pressure = ball.pressure.min(max_pressure);
    otherball.pressure = otherball.pressure.min(max_pressure);

    ball.velocity += force * pdiff;
    otherball.velocity -= force * pdiff;
}

fn resolve_boundaries(ball: &mut Ball, screen_width: f32, screen_height: f32, bounce_amount: f32) {
    let half_width = screen_width / 2.0;
    let half_height = screen_height / 2.0;

    if ball.position.x - ball.radius < -half_width {
        ball.position.x = -half_width + ball.radius;
        if ball.velocity.x < 0.0 {
            ball.velocity.x *= -bounce_amount;
        }
    } else if ball.position.x + ball.radius > half_width {
        ball.position.x = half_width - ball.radius;
        if ball.velocity.x > 0.0 {
            ball.velocity.x *= -bounce_amount;
        }
    }

    if ball.position.y - ball.radius < -half_height {
        ball.position.y = -half_height + ball.radius;
        if ball.velocity.y < 0.0 {
            ball.velocity.y *= -bounce_amount;
        }
    } else if ball.position.y + ball.radius > half_height {
        ball.position.y = half_height - ball.radius;
        if ball.velocity.y > 0.0 {
            ball.velocity.y *= -bounce_amount;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Config>,
) {
    commands.spawn(Camera2d);

    let ball_radius = config.ball_radius;
    let colors: Vec<Color> = (0..config.ball_count)
        .map(|_| Color::srgb(random::<f32>(), random::<f32>(), random::<f32>()))
        .collect();

    let circle_mesh = meshes.add(Circle::new(ball_radius));
    let half_width = config.width / 2.0;
    let half_height = config.height / 2.0;

    for (id, color) in colors.iter().copied().enumerate() {
        let position = Vec2::new(
            random::<f32>() * (config.width - 2.0 * ball_radius) - half_width + ball_radius,
            random::<f32>() * (config.height - 2.0 * ball_radius) - half_height + ball_radius,
        );
        let velocity = Vec2::new(
            random::<f32>() * 200.0 - 100.0,
            random::<f32>() * 200.0 - 100.0,
        );
        commands.spawn((
            Ball {
                id,
                position,
                velocity,
                pressure: 0.0,
                color,
                radius: ball_radius,
            },
            Mesh2d(circle_mesh.clone()),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position.extend(0.0)),
        ));
    }

    commands.insert_resource(BallColors(colors));
    commands.insert_resource(BallMesh(circle_mesh));

    let window_origin = Vec3::new(-half_width, -half_height, 0.0);
    let text_font = TextFont {
        font_size: 24.0,
        ..default()
    };

    let fps_entity = commands
        .spawn((
            Text2d::new("FPS: 0.00"),
            text_font.clone(),
            Anchor::TopLeft,
            Transform::from_translation(window_origin + Vec3::new(20.0, config.height - 20.0, 1.0)),
        ))
        .id();
    let sim_steps_entity = commands
        .spawn((
            Text2d::new(format!("SIM STEPS: {}", config.sim_steps)),
            text_font.clone(),
            Anchor::TopLeft,
            Transform::from_translation(window_origin + Vec3::new(20.0, config.height - 50.0, 1.0)),
        ))
        .id();
    let balls_entity = commands
        .spawn((
            Text2d::new(format!("BALLS: {}", config.ball_count)),
            text_font,
            Anchor::TopLeft,
            Transform::from_translation(window_origin + Vec3::new(20.0, config.height - 80.0, 1.0)),
        ))
        .id();

    commands.insert_resource(UiState {
        fps_entity,
        sim_steps_entity,
        balls_entity,
    });
}

fn handle_inputs(
    mut display_state: ResMut<DisplayState>,
    mut simulation_state: ResMut<SimulationState>,
    config: Res<Config>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        simulation_state.do_gravity = !simulation_state.do_gravity;
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        display_state.toggle_display_mode();
    }

    if config.auto_sim_steps {
        return;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        simulation_state.sim_steps += 1;
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        simulation_state.sim_steps -= 1;
    }
    simulation_state.sim_steps = simulation_state.sim_steps.clamp(1, 200);
}

fn spawn_ball_on_click(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut colors: ResMut<BallColors>,
    ball_mesh: Res<BallMesh>,
    config: Res<Config>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !mouse.pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let color = Color::srgb(random::<f32>(), random::<f32>(), random::<f32>());
    let ball_radius = config.ball_radius;
    let id = colors.0.len();
    let world_cursor = Vec2::new(
        cursor.x - window.width() / 2.0,
        window.height() / 2.0 - cursor.y,
    );

    commands.spawn((
        Ball {
            id,
            position: world_cursor,
            velocity: Vec2::new(
                random::<f32>() * 200.0 - 100.0,
                random::<f32>() * 200.0 - 100.0,
            ),
            pressure: 0.0,
            color,
            radius: ball_radius,
        },
        Mesh2d(ball_mesh.0.clone()),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(world_cursor.extend(0.0)),
    ));

    colors.0.push(color);
}

fn delete_balls_on_key(
    mut commands: Commands,
    mut colors: ResMut<BallColors>,
    config: Res<Config>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    balls: Query<(Entity, &Ball)>,
) {
    if !keyboard.pressed(KeyCode::KeyF) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let world_cursor = Vec2::new(
        cursor.x - window.width() / 2.0,
        window.height() / 2.0 - cursor.y,
    );

    let delete_dist = config.delete_dist;
    let mut to_remove: Vec<(Entity, usize)> = Vec::new();

    for (entity, ball) in balls.iter() {
        let dist = ball.position.distance(world_cursor);
        if dist < delete_dist {
            to_remove.push((entity, ball.id));
        }
    }

    // Sort by id in descending order to remove from colors vec correctly
    to_remove.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    for (entity, id) in to_remove {
        commands.entity(entity).despawn();
        if id < colors.0.len() {
            colors.0.remove(id);
        }
    }

    // Re-index remaining balls
    let mut balls_vec: Vec<(Entity, Ball)> = balls.iter().map(|(e, b)| (e, *b)).collect();
    balls_vec.sort_by_key(|(_, ball)| ball.id);
    
    for (idx, (entity, mut ball)) in balls_vec.iter_mut().enumerate() {
        ball.id = idx;
        commands.entity(*entity).insert(Ball {
            id: ball.id,
            position: ball.position,
            velocity: ball.velocity,
            pressure: ball.pressure,
            color: ball.color,
            radius: ball.radius,
        });
        if idx < colors.0.len() {
            colors.0[idx] = ball.color;
        }
    }
}

fn update_spatial_hash(
    mut hash: ResMut<SpatialHash<Entity>>,
    balls: Query<(Entity, &Ball)>,
) {
    hash.0.clear();
    for (entity, ball) in balls.iter() {
        hash.0.insert(ball.position, entity);
    }
}

fn simulate(
    mut balls: ParamSet<(Query<(Entity, &Ball)>, Query<&mut Ball>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    simulation_state: Res<SimulationState>,
    simulation_config: Res<SimulationConfig>,
    hash: Res<SpatialHash<Entity>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let screen_width = window.width();
    let screen_height = window.height();

    for _ in 0..simulation_state.sim_steps {
        let positions: Vec<(Entity, Vec2)> = balls
            .p0()
            .iter()
            .map(|(entity, ball)| (entity, ball.position))
            .collect();

        if positions.is_empty() {
            continue;
        }

        for (entity, position) in positions.iter().copied() {
            for other_entity in hash.0.get_nearby_objects(position, entity).iter().copied() {
                if entity == other_entity {
                    continue;
                }

                if let Ok([mut ball, mut other_ball]) =
                    balls.p1().get_many_mut([entity, other_entity])
                {
                    if is_colliding(&ball, &other_ball) {
                        resolve_collision(
                            &mut ball,
                            &mut other_ball,
                            simulation_config.bounce_amount,
                            simulation_config.max_pressure,
                        );
                    } else {
                        ball.pressure = 0.0;
                        other_ball.pressure = 0.0;
                    }
                }
            }

            if let Ok(mut ball) = balls.p1().get_mut(entity) {
                resolve_boundaries(
                    &mut ball,
                    screen_width,
                    screen_height,
                    simulation_config.bounce_amount,
                );
            }
        }
    }
}

fn apply_motion(
    mut balls: Query<&mut Ball>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    simulation_config: Res<SimulationConfig>,
    simulation_state: Res<SimulationState>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let mouse_pressed = mouse.pressed(MouseButton::Left);
    let cursor = window
        .cursor_position()
        .map(|cursor| Vec2::new(cursor.x - window.width() / 2.0, window.height() / 2.0 - cursor.y));

    let rate = time.delta_secs().max(MIN_DELTA_TIME);

    for mut ball in balls.iter_mut() {
        if mouse_pressed {
            if let Some(cursor) = cursor {
                let mut force = cursor - ball.position;
                let distance = force.length();
                if distance < 0.1 {
                    force /= distance;
                }
                ball.velocity += force * simulation_config.gravity * rate;
            }
        }

        if simulation_state.do_gravity {
            ball.velocity.y += simulation_config.gravity;
        }

        ball.velocity.x *= simulation_config.resistance;
        ball.velocity.y *= simulation_config.resistance;

        ball.velocity = ball.velocity.clamp_length_max(simulation_config.max_speed);
        let velocity = ball.velocity;
        ball.position += velocity * rate;
    }
}

fn update_visuals(
    mut balls: Query<(&Ball, &MeshMaterial2d<ColorMaterial>, &mut Transform)>,
    colors: Res<BallColors>,
    display_state: Res<DisplayState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut largest_speed: f32 = 0.0;
    let mut largest_pressure: f32 = 0.0;

    if display_state.display_mode == DisplayMode::Velocity {
        for (ball, _, _) in balls.iter() {
            largest_speed = largest_speed.max(ball.velocity.length());
        }
    }

    if display_state.display_mode == DisplayMode::Pressure {
        for (ball, _, _) in balls.iter() {
            largest_pressure = largest_pressure.max(ball.pressure);
        }
    }

    for (ball, material, mut transform) in balls.iter_mut() {
        let color = match display_state.display_mode {
            DisplayMode::Normal => colors.0[ball.id],
            DisplayMode::Velocity => get_color_from_vel(*ball, largest_speed),
            DisplayMode::Pressure => get_color_from_pressure(*ball, largest_pressure),
        };
        if let Some(material) = materials.get_mut(&material.0) {
            material.color = color;
        }
        transform.translation = ball.position.extend(0.0);
    }
}

fn update_fps(time: Res<Time>, mut metrics: ResMut<MetricsState>) {
    metrics.fps = 1.0 / time.delta_secs().max(0.0001);
}

fn update_sim_steps(
    mut simulation_state: ResMut<SimulationState>,
    config: Res<Config>,
    metrics: Res<MetricsState>,
) {
    if !config.auto_sim_steps {
        return;
    }

    if metrics.fps < config.target_fps as f32 {
        simulation_state.sim_steps -= 1;
    } else if metrics.fps > (config.target_fps + config.fps_boundary) as f32 {
        simulation_state.sim_steps += 1;
    }

    simulation_state.sim_steps = simulation_state.sim_steps.clamp(1, 200);
}

fn update_ui(
    ui_state: Res<UiState>,
    mut texts: Query<&mut Text2d>,
    balls: Query<&Ball>,
    simulation_state: Res<SimulationState>,
    metrics: Res<MetricsState>,
) {
    if let Ok(mut fps_text) = texts.get_mut(ui_state.fps_entity) {
        **fps_text = format!("FPS: {:.2}", metrics.fps);
    }
    if let Ok(mut sim_text) = texts.get_mut(ui_state.sim_steps_entity) {
        **sim_text = format!("SIM STEPS: {}", simulation_state.sim_steps);
    }
    if let Ok(mut balls_text) = texts.get_mut(ui_state.balls_entity) {
        **balls_text = format!("BALLS: {}", balls.iter().count());
    }
}

fn capture_screenshot(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        info!("Screenshot requested; use external tooling to capture the window in this environment.");
    }
}

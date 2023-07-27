use std::time::Duration;

use bevy::{
    prelude::{
        info, Camera, Commands, Component, Entity, GlobalTransform, Input, KeyCode,
        MouseButton, Query, ReflectResource, Res, ResMut, Resource, Vec2, With,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
    window::{PrimaryWindow, Window},
};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use rand::Rng;

use crate::render::MainCamera2d;

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidSettings {
    pub boid_radius: f32,
    pub spawn_count: u32,
    pub spawn_min_position: f32,
    pub spawn_max_position: f32,
    pub max_speed: f32,
    pub velocity_time_scale: f32,
    pub tick_time: u64,
    pub cohesion_radius: f32,
    pub alignment_radius: f32,
    pub separation_radius: f32,
    pub separation_weight: f32,
    pub collision_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub seek_weight: f32,
    pub boundary_min_x: f32,
    pub boundary_max_x: f32,
    pub boundary_min_y: f32,
    pub boundary_max_y: f32,
    pub boundary_avoidance_factor: f32,
}

impl Default for BoidSettings {
    fn default() -> Self {
        Self {
            boid_radius: 9.0,
            spawn_count: 30,
            spawn_min_position: -100.,
            spawn_max_position: 100.,
            max_speed: 10.0,
            velocity_time_scale: 0.3,     // 50.5,
            tick_time: 30,

            separation_radius: 17.8,
            separation_weight: 0.1,
            collision_weight: 0.8,
            alignment_radius: 30.0,
            alignment_weight: 0.001,

            cohesion_radius: 30.0,
            cohesion_weight: 0.00001,

            seek_weight: 0.005,

            boundary_min_x: -600.0,
            boundary_max_x: 600.0,
            boundary_min_y: -600.0,
            boundary_max_y: 600.0,

            boundary_avoidance_factor: 0.4,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct TargetPosition {
    pub position: Option<Vec2>,
}

#[derive(Resource)]
pub struct BoidTimer {
    pub timer: Timer,
}

impl Default for BoidTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(0), TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct Boid;

#[derive(Debug, Clone, Component)]
pub struct Position(pub Vec2);

#[derive(Debug, Clone, Component)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Clone, Component)]
pub struct ViewRadius(pub f32);

pub fn setup_boids(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    mut boid_timer: ResMut<BoidTimer>,
) {
    boid_timer.timer = Timer::new(
        Duration::from_millis(settings.tick_time),
        TimerMode::Repeating,
    );

    let mut rng = rand::thread_rng();
    let view_radius = 5.0;

    let mut positions: Vec<Vec2> = Vec::new();
    for _ in 0..settings.spawn_count {
        for _ in 0..10 {
            let candidate = Vec2::new(
                rng.gen_range(settings.spawn_min_position..settings.spawn_max_position),
                rng.gen_range(settings.spawn_min_position..settings.spawn_max_position),
            );

            // any overlapping?
            if !positions.iter().any(|pos| {
                let distance = pos.distance(candidate);
                distance < settings.boid_radius * 2.0
            }) {
                commands.spawn((
                    Boid,
                    Position(candidate),
                    Velocity(Vec2::ZERO),
                    ViewRadius(view_radius),
                ));
                positions.push(candidate);
            }
        }
    }
    info!("spawned {} boids", positions.len());
}

pub fn respawn_boids(
    mut commands: Commands,
    boid_timer: ResMut<BoidTimer>,
    boids: Query<Entity, With<Boid>>,
    keys: Res<Input<KeyCode>>,
    settings: Res<BoidSettings>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for entity in boids.iter() {
            commands.entity(entity).despawn();
        }
        setup_boids(commands, settings, boid_timer);
    }
}

pub fn update_target_from_mouse_click(
    buttons: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_transform: Query<(&Camera, &GlobalTransform), With<MainCamera2d>>,
    mut target_position: ResMut<TargetPosition>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        target_position.position = None;
    } else if buttons.just_pressed(MouseButton::Middle) {
        let (camera, camera_transform) = camera_transform.single();
        let window = windows.get_single().unwrap();

        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            target_position.position = Some(Vec2::new(world_position.x, world_position.y));
        }
    }
}

pub fn find_neighbors<'a>(
    boid: &(Entity, Position),
    boids: &'a [(Entity, Position)],
    radius: f32,
) -> Vec<(&'a Entity, &'a Position, f32)> {
    let mut neighbors = Vec::new();
    let (entity, position) = boid;
    for (other_entity, other_position) in boids {
        if entity != other_entity {
            let distance = position.0.distance(other_position.0);
            if distance < radius {
                neighbors.push((other_entity, other_position, distance));
            }
        }
    }
    neighbors
}

pub fn apply_boid_velocity(
    time: Res<Time>,
    settings: Res<BoidSettings>,
    mut boids: Query<
        (&mut Position, &mut Velocity),
        With<Boid>,
    >,
) {
    for (mut position, mut velocity) in boids.iter_mut() {
        position.0 += velocity.0 * (time.elapsed_seconds() * settings.velocity_time_scale);
    }
}

pub fn update_boids(
    time: Res<Time>,
    settings: Res<BoidSettings>,
    mut boids: Query<(Entity, &mut Position, &mut Velocity), With<Boid>>,
    target_position: Res<TargetPosition>,
    mut boid_timer: ResMut<BoidTimer>,
) {
    boid_timer.timer.tick(time.delta());

    if !boid_timer.timer.finished() {
        return;
    }

    let list_of_boids: Vec<(Entity, Position)> = boids
        .iter()
        .map(|(entity, position, _)| (entity, position.clone()))
        .collect();

    for boid in list_of_boids.iter() {
        let (boid_entity, boid_position) = boid;
        let mut accel = Vec2::ZERO;

        // Target seeking
        if let Some(seek_target) = target_position.position {
            let seek_velocity = seek_target - boid_position.0;
            accel += seek_velocity * settings.seek_weight;
        }

        // Collision
        let neighbors = find_neighbors(boid, &list_of_boids, settings.boid_radius * 2.0);
        let mut collision_velocity = Vec2::new(0., 0.);
        for (_, neighbor_position, _) in neighbors.iter() {
            let separation_vector = boid_position.0 - neighbor_position.0;
            collision_velocity += separation_vector;
        }
        accel += collision_velocity * settings.collision_weight;

        // 1. Separation
        let neighbors = find_neighbors(boid, &list_of_boids, settings.separation_radius);
        let mut separation_velocity = Vec2::new(0., 0.);
        for (_, neighbor_position, _) in neighbors.iter() {
            let separation_vector = boid_position.0 - neighbor_position.0;
            separation_velocity += separation_vector;
        }
        accel += separation_velocity * settings.separation_weight;

        // 2. Alignment
        let neighbors = find_neighbors(boid, &list_of_boids, settings.alignment_radius);
        let mut alignment_velocity = Vec2::new(0., 0.);
        for (neighbor_entity, _, _) in neighbors.iter() {
            if let Ok((_, _, neighbor_velocity)) = boids.get(**neighbor_entity) {
                alignment_velocity += neighbor_velocity.0;
            }
        }
        if !neighbors.is_empty() {
            alignment_velocity.x /= neighbors.len() as f32;
            alignment_velocity.y /= neighbors.len() as f32;
        }
        accel += alignment_velocity * settings.separation_weight;

        // 3. Cohesion
        let neighbors = find_neighbors(boid, &list_of_boids, settings.cohesion_radius);
        let mut average_position = Vec2::new(0., 0.);
        for (neighbor_entity, _, _) in neighbors.iter() {
            if let Ok((_, neighbor_position, _)) = boids.get(**neighbor_entity) {
                average_position += neighbor_position.0;
            }
        }
        if !neighbors.is_empty() {
            average_position.x /= neighbors.len() as f32;
            average_position.y /= neighbors.len() as f32;
        }
        accel += average_position * settings.cohesion_weight;

        // Boundary avoidance
        if boid_position.0.x < settings.boundary_min_x {
            accel.x += settings.boundary_avoidance_factor;
        }
        if boid_position.0.x > settings.boundary_max_x {
            accel.x -= settings.boundary_avoidance_factor;
        }
        if boid_position.0.y < settings.boundary_min_y {
            accel.y += settings.boundary_avoidance_factor;
        }
        if boid_position.0.y > settings.boundary_max_y {
            accel.y -= settings.boundary_avoidance_factor;
        }

        // limit top speed  of boids
        if accel.length() > settings.max_speed {
            accel = accel.normalize() * settings.max_speed;
        }

        // update velocity:
        let (_, _, mut boid_velocity) = boids.get_mut(*boid_entity).unwrap();
        boid_velocity.0 = accel;
    }
}

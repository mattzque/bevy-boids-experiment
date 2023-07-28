use std::time::Duration;

use bevy::{
    prelude::{
        info, Camera, Commands, Component, Entity, GlobalTransform, Input, KeyCode, MouseButton,
        Query, ReflectResource, Res, ResMut, Resource, Vec2, With,
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
    pub max_force: f32,
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
            spawn_count: 200,
            spawn_min_position: -500.,
            spawn_max_position: 500.,
            max_speed: 0.2,
            max_force: 0.05,
            velocity_time_scale: 0.01, // 50.5,
            tick_time: 48,

            separation_radius: 17.6,
            separation_weight: 1.0,

            collision_weight: 0.0,

            alignment_radius: 30.0,
            alignment_weight: 0.8,

            cohesion_radius: 25.0,
            cohesion_weight: 0.0002,

            seek_weight: 0.0003,

            boundary_min_x: -600.0,
            boundary_max_x: 600.0,
            boundary_min_y: -600.0,
            boundary_max_y: 600.0,

            boundary_avoidance_factor: 0.9,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct TargetPosition {
    pub position: Option<Vec2>,
}

#[derive(Resource)]
pub struct BoidTimer(Timer);

impl Default for BoidTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(0), TimerMode::Repeating))
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
    boid_timer.0 = Timer::new(
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
                    Velocity(Vec2::new(
                        rng.gen_range(-0.5..0.5),
                        rng.gen_range(-0.5..0.5),
                    )),
                    ViewRadius(view_radius),
                ));
                positions.push(candidate);
                break;
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

fn limit_vec2(vector: Vec2, max_length: f32) -> Vec2 {
    if vector.length() > max_length {
        vector.normalize() * max_length
    } else {
        vector
    }
}

/// Separation, steer away from nearby boids
///
/// Arguments:
/// position: the current position of this boid
/// velocity: the current velocity of this boid
/// boids: the position of all other boids (including itself)
/// separation_distance: how close other boids for consideration
/// max_speed: the maximum speed of this boid
/// max_force: the maximum force that can be applied to this boid
///
/// Returns: separation force vector
fn get_separation_force(
    position: Vec2,
    velocity: Vec2,
    boids: &[Vec2],
    separation_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut force = Vec2::ZERO;
    let mut count = 0;
    for other_position in boids {
        let distance = position.distance(*other_position);
        if distance > 0.0 && distance < separation_distance {
            let mut diff = position - *other_position;
            diff = diff.normalize();
            diff /= distance;
            force += diff;
            count += 1;
        }
    }
    if count > 0 {
        force /= count as f32;
    }
    if force.length() > 0.0 {
        // force = force.normalize();
        // force *= max_speed;
        force -= velocity;
        // force = limit_vec2(force, max_force);
    }
    force
}

/// Alignment, steer along with the average velocity of nearby boids
///
/// Arguments:
/// position: the current position of this boid
/// velocity: the current velocity of this boid
/// boids: the position and velocity of all other boids (including itself)
/// alignment_distance: how close other boids for consideration
/// max_speed: the maximum speed of this boid
/// max_force: the maximum force that can be applied to this boid
///
/// Returns: alignment force vector
fn get_alignment_force(
    position: Vec2,
    velocity: Vec2,
    boids: &[(Vec2, Vec2)],
    alignment_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut average_velocity = Vec2::ZERO;
    let mut count = 0;
    for (other_position, other_velocity) in boids {
        let distance = position.distance(*other_position);
        if distance > 0.0 && distance < alignment_distance {
            average_velocity += *other_velocity;
            count += 1;
        }
    }
    if count > 0 {
        average_velocity /= count as f32;
        average_velocity - velocity
    } else {
        Vec2::ZERO
    }
}

fn get_seek_force(
    position: Vec2,
    velocity: Vec2,
    target: Vec2,
) -> Vec2 {
    let mut desired = target - position;
    if desired == Vec2::ZERO {
        return Vec2::ZERO;
    }
    let mut distance = position.distance(target);
    // println!("desired 1: {:?}", desired);
    // desired = desired.normalize();
    // desired *= max_speed;
    // println!("desired 2: {:?}", desired);
    let mut steer = (desired - velocity); //  / distance;
    // println!("steer1: {:?}", steer);
    // steer = limit_vec2(steer, max_force);
    // println!("steer2: {:?}", steer);
    // println!("max_force: {:?}", max_force);
    steer
}

/// Cohesion, steer towards the average position of nearby boids
///
/// Arguments:
/// position: the current position of this boid
/// velocity: the current velocity of this boid
/// boids: the position of all other boids (including itself)
/// cohesion_distance: how close other boids for consideration
/// max_speed: the maximum speed of this boid
/// max_force: the maximum force that can be applied to this boid
///
/// Returns: cohesion force vector
fn get_cohesion_force(
    position: Vec2,
    velocity: Vec2,
    boids: &[Vec2],
    cohesion_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut average_position = Vec2::ZERO;
    let mut count = 0;
    for other_position in boids {
        let distance = position.distance(*other_position);
        if distance > 0.0 && distance < cohesion_distance {
            average_position += *other_position;
            count += 1;
        }
    }
    if count > 0 {
        average_position /= count as f32;
        if average_position.length() > 0.0 {
            get_seek_force(position, velocity, average_position)
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO
    }
}

pub fn update(
    time: Res<Time>,
    mut timer: ResMut<BoidTimer>,
    settings: Res<BoidSettings>,
    target: Res<TargetPosition>,
    mut query: Query<(Entity, &mut Position, &mut Velocity), With<Boid>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }

    let boids: Vec<Vec2> = query
        .iter()
        .map(|(_, position, _)| position.0.clone())
        .collect();

    let boids2: Vec<(Vec2, Vec2)> = query
        .iter()
        .map(|(_, position, velocity)| (position.0.clone(), velocity.0.clone()))
        .collect();

    for (entity, mut position, mut velocity) in query.iter_mut() {
        let collision_force = get_separation_force(
            position.0,
            velocity.0,
            &boids,
            settings.boid_radius,
            settings.max_speed,
            settings.max_force,
        );
        let separation_force = get_separation_force(
            position.0,
            velocity.0,
            &boids,
            settings.separation_radius,
            settings.max_speed,
            settings.max_force,
        );
        let alignment_force = get_alignment_force(
            position.0,
            velocity.0,
            &boids2,
            settings.alignment_radius,
            settings.max_speed,
            settings.max_force,
        );
        let cohesion_force = get_cohesion_force(
            position.0,
            velocity.0,
            &boids,
            settings.cohesion_radius,
            settings.max_speed,
            settings.max_force,
        );

        if cohesion_force.is_nan() {
            println!("cohesion_force: {:?}", cohesion_force);
        }

        // println!("separation_force: {:?}", separation_force);
        // println!("alignment_force: {:?}", alignment_force);
        // println!("separation_force: {:?}", cohesion_force);

        let mut acceleration = separation_force * settings.separation_weight;
        acceleration += alignment_force * settings.alignment_weight;
        acceleration += cohesion_force * settings.cohesion_weight;
        acceleration += collision_force * settings.collision_weight;

        if let Some(target_position) = target.position {
            let force = get_seek_force(position.0, velocity.0, target_position);
            acceleration += force * settings.seek_weight;
        }

        // Boundary avoidance
        if position.0.x < settings.boundary_min_x {
            acceleration.x = settings.max_force;
            //acceleration.x += settings.boundary_avoidance_factor;
        }
        if position.0.x > settings.boundary_max_x {
            acceleration.x = -settings.max_force;
            //acceleration.x -= settings.boundary_avoidance_factor;
        }
        if position.0.y < settings.boundary_min_y {
            acceleration.y = settings.max_force;
            //acceleration.y += settings.boundary_avoidance_factor;
        }
        if position.0.y > settings.boundary_max_y {
            acceleration.y = -settings.max_force;
            //acceleration.y -= settings.boundary_avoidance_factor;
        }

        acceleration = limit_vec2(acceleration, settings.max_force);

        // println!("acceleration: {:?}", acceleration);
        velocity.0 += acceleration; // * time.delta_seconds();
        velocity.0 = limit_vec2(velocity.0, settings.max_speed);

        // position.0 += velocity.0; //  * time.delta_seconds();
    }
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

/*
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
        let (_, _, mut boid_velocity) = boids.get_mut(*boid_entity).unwrap();
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

*/

use bevy::{
    prelude::{
        Commands, Component, Entity, Input, KeyCode, Query, ReflectResource, Res, Resource, Vec2,
        With,
    },
    reflect::Reflect,
    time::Time,
};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use rand::Rng;

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidSettings {
    pub spawn_count: u32,
    pub spawn_min_position: f32,
    pub spawn_max_position: f32,
    pub spawn_min_vel: f32,
    pub spawn_max_vel: f32,
    pub max_speed: f32,
    pub cohesion_radius: f32,
    pub alignment_radius: f32,
    pub separation_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub velocity_time_scale: f32,
}

impl Default for BoidSettings {
    fn default() -> Self {
        Self {
            spawn_count: 800,
            spawn_min_position: -30.,
            spawn_max_position: 30.,
            spawn_min_vel: -15.0,
            spawn_max_vel: 15.0,
            max_speed: 10.0,
            velocity_time_scale: 50.5,
            separation_radius: 22.,
            separation_weight: 32.2,
            alignment_radius: 100.0,
            alignment_weight: 1.5,
            cohesion_radius: 233.7,
            cohesion_weight: -0.3,
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

pub fn setup_boids(mut commands: Commands, settings: Res<BoidSettings>) {
    let mut rng = rand::thread_rng();
    let view_radius = 5.0;

    for _ in 0..settings.spawn_count {
        commands.spawn((
            Boid,
            Position(Vec2::new(
                rng.gen_range(settings.spawn_min_position..settings.spawn_max_position),
                rng.gen_range(settings.spawn_min_position..settings.spawn_max_position),
            )),
            Velocity(Vec2::new(
                rng.gen_range(settings.spawn_min_vel..settings.spawn_max_vel),
                rng.gen_range(settings.spawn_min_vel..settings.spawn_max_vel),
            )),
            ViewRadius(view_radius),
        ));
    }
}

pub fn respawn_boids(
    mut commands: Commands,
    boids: Query<Entity, With<Boid>>,
    keys: Res<Input<KeyCode>>,
    settings: Res<BoidSettings>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for entity in boids.iter() {
            commands.entity(entity).despawn();
        }
        setup_boids(commands, settings);
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
    mut boids: Query<(&mut Position, &Velocity), With<Boid>>,
) {
    let scale = 40.0;
    for (mut position, velocity) in boids.iter_mut() {
        position.0 += velocity.0 * time.delta_seconds() * scale;
    }
}

pub fn update_boids(
    time: Res<Time>,
    settings: Res<BoidSettings>,
    mut boids: Query<(Entity, &mut Position, &mut Velocity), With<Boid>>,
) {
    let list_of_boids: Vec<(Entity, Position)> = boids
        .iter()
        .map(|(entity, position, _)| (entity, position.clone()))
        .collect();

    for boid in list_of_boids.iter() {
        let (boid_entity, boid_position) = boid;

        // 1. Separation
        let neighbors = find_neighbors(boid, &list_of_boids, settings.separation_radius);
        let mut separation_velocity = Vec2::new(0., 0.);
        for (_, neighbor_position, _) in neighbors.iter() {
            let separation_vector = boid_position.0 - neighbor_position.0;
            separation_velocity += separation_vector;
        }

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
        let cohesion_velocity = average_position - boid_position.0;

        let mut final_velocity: Vec2 = separation_velocity * settings.separation_weight
            + alignment_velocity * settings.alignment_weight
            + cohesion_velocity * settings.cohesion_weight;

        if final_velocity.length() > settings.max_speed {
            final_velocity = final_velocity.normalize() * settings.max_speed;
        }

        if let Ok((_, _, mut boid_velocity)) = boids.get_mut(*boid_entity) {
            boid_velocity.0 =
                final_velocity * (time.delta_seconds() * settings.velocity_time_scale);
        }
    }
}

use std::f32::consts::PI;

use bevy::prelude::{
    Added, Camera2dBundle, Changed, Color, Commands, Component, Entity, Quat, Query, Res,
    Transform, Vec2, Vec3, Visibility, With,
};
use bevy_prototype_lyon::{
    prelude::{GeometryBuilder, ShapeBundle, Stroke},
    shapes,
};

use crate::boids::{Boid, BoidSettings, Position, TargetPosition, Velocity, ViewRadius};

#[derive(Component)]
pub struct MainCamera2d;

#[derive(Component)]
pub struct TargetPositionRenderable;

pub fn setup_render(mut commands: Commands, settings: Res<BoidSettings>) {
    let mut builder = GeometryBuilder::new();

    let steps = 100;
    let gap = 50.;
    let size = 5000.;
    // vertical lines
    for x in (-steps)..steps {
        builder = builder.add(&shapes::Line(
            Vec2::new(x as f32 * gap, -size),
            Vec2::new(x as f32 * gap, size),
        ));
    }
    // horizontal lines
    for y in (-steps)..steps {
        builder = builder.add(&shapes::Line(
            Vec2::new(-size, y as f32 * gap),
            Vec2::new(size, y as f32 * gap),
        ));
    }

    commands.spawn((
        ShapeBundle {
            path: builder.build(),
            ..Default::default()
        },
        Stroke::new(Color::hex("999999").unwrap(), 1.0),
    ));

    let mut builder = GeometryBuilder::new();

    let target_radius = 10.0;
    builder = builder.add(&shapes::Circle {
        radius: target_radius,
        center: Vec2::ZERO,
    });
    builder = builder.add(&shapes::Line(
        Vec2::new(-target_radius, -target_radius),
        Vec2::new(target_radius, target_radius),
    ));
    builder = builder.add(&shapes::Line(
        Vec2::new(-target_radius, target_radius),
        Vec2::new(target_radius, -target_radius),
    ));
    commands.spawn((
        ShapeBundle {
            path: builder.build(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            ..Default::default()
        },
        Stroke::new(Color::RED, 1.0),
        TargetPositionRenderable,
    ));

    // walls
    let mut builder = GeometryBuilder::new();
    let boundary = shapes::Polygon {
        points: [
            Vec2::new(settings.boundary_min_x, settings.boundary_min_y),
            Vec2::new(settings.boundary_min_x, settings.boundary_max_y),
            Vec2::new(settings.boundary_max_x, settings.boundary_max_y),
            Vec2::new(settings.boundary_max_x, settings.boundary_min_y),
        ]
        .to_vec(),
        closed: true,
    };
    builder = builder.add(&boundary);
    commands.spawn((
        ShapeBundle {
            path: builder.build(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
            ..Default::default()
        },
        Stroke::new(Color::BLUE, 1.0),
    ));
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera2d));
}

fn get_transform_for_boid(position: &Position, velocity: &Velocity) -> Transform {
    let Position(position) = position;
    let Velocity(velocity) = velocity;
    let mut degrees = velocity.angle_between(Vec2::new(0.0, 1.0));
    if degrees.is_nan() {
        degrees = 0.0_f32.to_radians();
    }
    Transform::from_translation(Vec3::new(position.x, position.y, 1.0)).with_rotation(
        Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), PI * 2. - degrees),
    )
}

pub fn spawn_boid_renderable(
    settings: Res<BoidSettings>,
    mut commands: Commands,
    boids: Query<(Entity, &Position, &Velocity, &ViewRadius), (With<Boid>, Added<Boid>)>,
) {
    for (entity, position, velocity, view_radius) in boids.iter() {
        let mut builder = GeometryBuilder::new();

        let boid_radius = settings.boid_radius;
        let boid_color = Color::BLACK;

        // circle representing the boid
        let circle = shapes::Circle {
            radius: boid_radius,
            center: Vec2::ZERO,
        };
        let line = shapes::Line(Vec2::ZERO, Vec2::new(0.0, boid_radius));

        builder = builder.add(&circle);
        builder = builder.add(&line);

        commands.entity(entity).insert((
            ShapeBundle {
                path: builder.build(),
                transform: get_transform_for_boid(position, velocity),
                ..Default::default()
            },
            Stroke::new(boid_color, 2.0),
        ));
    }
}

pub fn update_boid_renderable_transform(
    mut boids: Query<
        (Entity, &Position, &Velocity, &ViewRadius, &mut Transform),
        (With<Boid>, Changed<Position>),
    >,
) {
    for (entity, position, velocity, view_radius, mut transform) in boids.iter_mut() {
        *transform = get_transform_for_boid(position, velocity);
    }
}

pub fn update_boid_target_renderable_transform(
    target_position: Res<TargetPosition>,
    mut target: Query<(&mut Transform, &mut Visibility), (With<TargetPositionRenderable>)>,
) {
    if let Ok((mut transform, mut visibility)) = target.get_single_mut() {
        match target_position.position {
            Some(position) => {
                *visibility = Visibility::Visible;
                transform.translation = Vec3::new(position.x, position.y, 2.0)
            }
            None => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

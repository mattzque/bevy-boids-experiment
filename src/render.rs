use bevy::prelude::{
    Added, Camera2dBundle, Changed, Color, Commands, Entity, Query, Transform, Vec2, Vec3, With,
};
use bevy_prototype_lyon::{
    prelude::{GeometryBuilder, ShapeBundle, Stroke},
    shapes,
};

use crate::boids::{Boid, Position, Velocity, ViewRadius};

pub fn setup_render(mut commands: Commands) {
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
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn spawn_boid_renderable(
    mut commands: Commands,
    boids: Query<(Entity, &Position, &Velocity, &ViewRadius), (With<Boid>, Added<Boid>)>,
) {
    for (entity, Position(position), velocity, view_radius) in boids.iter() {
        let mut builder = GeometryBuilder::new();

        let boid_radius = 10.0;
        let boid_color = Color::BLACK;

        // circle representing the boid
        let circle = shapes::Circle {
            radius: boid_radius,
            center: Vec2::ZERO,
        };

        builder = builder.add(&circle);

        commands.entity(entity).insert((
            ShapeBundle {
                path: builder.build(),
                transform: Transform::from_translation(Vec3::new(position.x, position.y, 1.0)),
                ..Default::default()
            },
            Stroke::new(boid_color, 5.0),
        ));
    }
}

pub fn update_boid_renderable_transform(
    mut boids: Query<
        (Entity, &Position, &Velocity, &ViewRadius, &mut Transform),
        (With<Boid>, Changed<Position>),
    >,
) {
    for (entity, Position(position), velocity, view_radius, mut transform) in boids.iter_mut() {
        transform.translation = Vec3::new(position.x, position.y, 1.0);
    }
}

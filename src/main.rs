use bevy::{
    prelude::*,
    window::{PresentMode, Window, WindowResolution},
};
use bevy::{app::AppExit, prelude::*};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use boids::{BoidSettings, TargetPosition, BoidTimer};

mod boids;
mod render;

pub fn quit_on_escape(mut exit: EventWriter<AppExit>, key: Res<Input<KeyCode>>) {
    if key.just_pressed(KeyCode::Escape) || key.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }
}

fn main() {
    let screen_width = 1280.;
    let screen_height = 1280.;
    let window_scaling_factor = 1.0;
    let present_mode = PresentMode::AutoNoVsync; // PresentMode::AutoNoVsync
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Boids".into(),
                    resolution: WindowResolution::new(screen_width, screen_height)
                        .with_scale_factor_override(window_scaling_factor),
                    present_mode,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..Default::default()
                }),

                ..Default::default()
            }),
        )
        .add_plugins(ShapePlugin)
        .insert_resource(TargetPosition::default())
        .insert_resource(BoidTimer::default())
        .init_resource::<BoidSettings>() // `ResourceInspectorPlugin` won't initialize the resource
        .register_type::<BoidSettings>() // you need to register your type to display it
        .add_plugins(ResourceInspectorPlugin::<BoidSettings>::default())
        .add_systems(Startup, boids::setup_boids)
        .add_systems(Startup, render::setup_camera)
        .add_systems(Startup, render::setup_render)
        .add_systems(Update, render::spawn_boid_renderable)
        .add_systems(Update, render::update_boid_renderable_transform)
        .add_systems(Update, render::update_boid_target_renderable_transform)
        .add_systems(Update, boids::respawn_boids)
        .add_systems(Update, boids::update_target_from_mouse_click)
        .add_systems(Update, boids::apply_boid_velocity)
        .add_systems(Update, boids::update)
        .add_systems(Update, quit_on_escape)

        .run();
}

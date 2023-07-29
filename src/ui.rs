use bevy::{prelude::{ResMut}};
use bevy_egui::{egui::{self, Vec2}, EguiContexts};

use crate::boids::BoidSettings;


pub fn update_ui(
    mut settings: ResMut<BoidSettings>,
    mut contexts: EguiContexts
) {
    egui::Window::new("Boids Settings").show(contexts.ctx_mut(), |ui| {
        ui.style_mut().spacing.slider_width = 300.0;


        ui.add(egui::Slider::new(&mut settings.boid_radius, 3.0..=30.0).text("Boid Radius"));

        ui.add(egui::Slider::new(&mut settings.spawn_count, 1..=600).text("Spawn Count"));
        ui.add(egui::Slider::new(&mut settings.spawn_min_position, -600.0..=600.0).text("Min Spawn Position"));
        ui.add(egui::Slider::new(&mut settings.spawn_max_position, -600.0..=600.0).text("Max Spawn Position"));
        ui.add(egui::Slider::new(&mut settings.max_speed, 0.0..=2.0).text("Max Speed"));
        ui.add(egui::Slider::new(&mut settings.max_force, 0.0..=2.0).text("Max Force"));
        ui.add(egui::Slider::new(&mut settings.velocity_time_scale, 0.0..=2.0).text("Velocity Time Scale"));

        ui.add(egui::Slider::new(&mut settings.tick_time, 10..=150).text("Tick Time (ms)"));

        ui.add(egui::Slider::new(&mut settings.cohesion_radius, 5.0..=150.0).text("Cohesion Radius (px)"));
        ui.add(egui::Slider::new(&mut settings.cohesion_weight, 0.0..=10.0).text("Cohesion Weight"));

        ui.add(egui::Slider::new(&mut settings.alignment_radius, 5.0..=150.0).text("Alignment Radius (px)"));
        ui.add(egui::Slider::new(&mut settings.alignment_weight, 0.0..=10.0).text("Alignment Weight"));

        ui.add(egui::Slider::new(&mut settings.separation_radius, 5.0..=150.0).text("Separation Radius (px)"));
        ui.add(egui::Slider::new(&mut settings.separation_weight, 0.0..=10.0).text("Separation Weight"));

        ui.add(egui::Slider::new(&mut settings.collision_weight, 0.0..=10.0).text("Collision Weight"));

        ui.add(egui::Slider::new(&mut settings.seek_weight, 0.0..=10.0).text("Target Seek Weight"));

        ui.set_min_size(Vec2::new(500.0, 500.0));

    });

}
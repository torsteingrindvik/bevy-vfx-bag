#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{post_processing::wave::Wave, BevyVfxBagPlugin};

#[derive(Debug, Resource, Default)]
struct SlowerTime(Time);

fn main() {
    App::new()
        .add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

fn update(
    mut command: Commands,
    time: Res<Time>,
    query: Query<(Entity, Option<&Wave>), With<Camera>>,
) {
    if time.elapsed_seconds().fract() < 0.8 {
        if let (entity, Some(_)) = query.single() {
            command.get_or_spawn(entity).remove::<Wave>();
            info!("Is that a T-Rex approaching?!");
        }
    } else if let (entity, None) = query.single() {
        command.get_or_spawn(entity).insert(Wave {
            waves_x: 2.0,
            waves_y: 0.1,
            speed_x: 30.,
            speed_y: 20.,
            amplitude_x: 0.01,
            amplitude_y: 0.01,
        });
        info!("<GROUND SHAKE>");
    }
}

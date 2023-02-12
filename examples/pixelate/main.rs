//! This example shows the pixelation effect as well as
//! how to toggle a post processing effect at runtime.
//! All post processing effects may be toggled as such.
#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{post_processing::pixelate::Pixelate, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin::default())
        .add_startup_system(startup)
        .add_system(examples_common::print_on_change::<Pixelate>)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [t] to toggle, [up/down] to change");

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        Pixelate::default(),
    ));
}

fn update(
    mut saved_settings: Local<Pixelate>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, Option<&mut Pixelate>), With<Camera>>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        match query.single() {
            (entity, None) => {
                info!("Toggling ON");
                commands.get_or_spawn(entity).insert(*saved_settings);
            }
            (entity, Some(settings)) => {
                info!("Toggling OFF");
                commands.get_or_spawn(entity).remove::<Pixelate>();
                *saved_settings = *settings;
            }
        };
    }

    if let (_, Some(mut settings)) = query.single_mut() {
        if keyboard_input.just_pressed(KeyCode::Up) {
            settings.block_size += 1.0;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            settings.block_size -= 1.0;
        };
    }
}

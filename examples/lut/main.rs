#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{post_processing::lut::Lut, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [left|right] to change which LUT is in use");

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(Lut::default());
}

// Cycle through some preset LUTs.
fn update(
    mut choice: Local<usize>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<Camera>>,
) {
    let choice_now = if keyboard_input.just_pressed(KeyCode::Left) {
        choice.saturating_sub(1)
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        (*choice + 1).min(3)
    } else {
        *choice
    };

    if *choice != choice_now {
        let entity = query.single_mut();

        *choice = choice_now;
        match *choice {
            0 => {
                commands.get_or_spawn(entity).insert(Lut::neo());
                info!("Neo");
            }
            1 => {
                commands.get_or_spawn(entity).insert(Lut::arctic());
                info!("Arctic");
            }
            2 => {
                commands.get_or_spawn(entity).insert(Lut::slate());
                info!("Slate");
            }
            3 => {
                commands.get_or_spawn(entity).remove::<Lut>();
                info!("Disabled (default Bevy colors)");
            }
            _ => unreachable!(),
        }
    }
}

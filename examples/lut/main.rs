#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::lut::{Lut, LutPassthrough, LutPlugin, Luts},
    BevyVfxBagPlugin, PostProcessingInput,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(LutPlugin)
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(PostProcessingInput);
}

// Cycle through some preset LUTs.
fn update(
    mut choice: Local<usize>,
    mut lut: ResMut<Lut>,
    luts: Res<Luts>,
    mut text: ResMut<examples_common::ExampleText>,
    keyboard_input: Res<Input<KeyCode>>,
    mut passthrough: ResMut<LutPassthrough>,
) {
    let luts = luts.ready.iter().collect::<Vec<_>>();

    let num_luts = luts.len();
    if num_luts == 0 {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::P) {
        passthrough.0 = !passthrough.0;
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        lut.split_vertically = !lut.split_vertically;
    }

    let choice_now = if keyboard_input.just_pressed(KeyCode::Left) {
        choice.saturating_sub(1)
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        (*choice + 1).min(num_luts - 1)
    } else {
        *choice
    };

    let (name, lut3d) = &luts[choice_now];

    if *choice != choice_now {
        *choice = choice_now;
        lut.set_texture(lut3d);
    }

    text.0 = format!(
        "(←→) LUT {}/{num_luts}: {name}, [S]plit: {:?}, [P]assthrough: {:?}",
        *choice + 1,
        lut.split_vertically,
        passthrough.0
    );
}

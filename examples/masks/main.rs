#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{
    post_processing::masks::{Mask, MaskVariant},
    BevyVfxBagPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .add_system(examples_common::print_on_change::<Mask>)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [1|2|3] to change which mask is in use, [Up|Down] to change strenght, [L|H] to go low/high [PgUp/PgDown] to fade in/out the mask");

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(Mask::default());
}

fn update(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Mask, With<Camera>>) {
    let mut mask = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Key1) {
        *mask = Mask::square();
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        *mask = Mask::crt();
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        *mask = Mask::vignette();
    };

    // Let user change strength in increments via up, down arrows
    let increment = || match mask.variant {
        MaskVariant::Square => 1.,
        MaskVariant::Crt => 1000.,
        MaskVariant::Vignette => 0.05,
    };

    if keyboard_input.pressed(KeyCode::Up) {
        mask.strength += increment();
    } else if keyboard_input.pressed(KeyCode::Down) {
        mask.strength -= increment();
    };

    if keyboard_input.pressed(KeyCode::PageUp) {
        mask.fade += 0.01;
    } else if keyboard_input.pressed(KeyCode::PageDown) {
        mask.fade -= 0.01;
    };

    // Let user go to low- and high strength values directly via L and H keys
    let low = || match mask.variant {
        MaskVariant::Square => 3.,
        MaskVariant::Crt => 3000.,
        MaskVariant::Vignette => 0.1,
    };

    let high = || match mask.variant {
        MaskVariant::Square => 100.,
        MaskVariant::Crt => 500000.,
        MaskVariant::Vignette => 1.5,
    };

    if keyboard_input.just_pressed(KeyCode::L) {
        mask.strength = low();
    } else if keyboard_input.just_pressed(KeyCode::H) {
        mask.strength = high();
    };
}

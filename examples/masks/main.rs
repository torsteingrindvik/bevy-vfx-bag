#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::post_processing::{
    masks::{Mask, MaskVariant},
    PostProcessingPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PostProcessingPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [1|2|3] to change which mask is in use, [Up|Down] to change strenght, [L|H] to go low/high");

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

    // Let user change type of mask via 1, 2, 3
    let mut changed = if keyboard_input.just_pressed(KeyCode::Key1) {
        *mask = Mask::square();
        true
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        *mask = Mask::crt();
        true
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        *mask = Mask::vignette();
        true
    } else {
        false
    };

    // Let user change strength in increments via up, down arrows
    let increment = || match mask.variant {
        MaskVariant::Square => 1.,
        MaskVariant::Crt => 1000.,
        MaskVariant::Vignette => 0.05,
    };

    let increment = if keyboard_input.pressed(KeyCode::Up) {
        changed = true;
        increment()
    } else if keyboard_input.pressed(KeyCode::Down) {
        changed = true;
        -increment()
    } else {
        0.0
    };

    mask.strength += increment;

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

    mask.strength = if keyboard_input.just_pressed(KeyCode::L) {
        changed = true;
        low()
    } else if keyboard_input.just_pressed(KeyCode::H) {
        changed = true;
        high()
    } else {
        mask.strength
    };

    if changed {
        info!("{:?}, strength: {:.2}", mask.variant, mask.strength);
    }
}

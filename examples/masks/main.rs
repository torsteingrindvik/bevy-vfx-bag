#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy::render::camera::RenderTarget;
use bevy_vfx_bag::image::mask::*;
use bevy_vfx_bag::{BevyVfxBagImage, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    // Set up the base example
    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        // Add required plugin for using any effect at all
        .add_plugin(BevyVfxBagPlugin)
        // Add required resource for using masks
        .insert_resource(Mask::new_vignette())
        // Add required plugin for using masks
        .add_plugin(MaskPlugin)
        .add_startup_system(startup)
        // Shows how to change the effect at runtime
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands, image_handle: Res<BevyVfxBagImage>) {
    // Normal camera spawn
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            camera: Camera {
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .insert(UiCameraConfig { show_ui: false });
}

fn update(
    mut mask: ResMut<Mask>,
    mut text: ResMut<examples_common::ExampleText>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Let user change type of mask via 1, 2, 3
    if keyboard_input.just_pressed(KeyCode::Key1) {
        *mask = Mask::new_square();
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        *mask = Mask::new_crt();
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        *mask = Mask::new_vignette();
    }

    // Let user change strength in increments via up, down arrows
    let increment = || match mask.variant {
        MaskVariant::Square => 1.,
        MaskVariant::Crt => 1000.,
        MaskVariant::Vignette => 0.05,
    };

    let increment = if keyboard_input.just_pressed(KeyCode::Up) {
        increment()
    } else if keyboard_input.just_pressed(KeyCode::Down) {
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
        low()
    } else if keyboard_input.just_pressed(KeyCode::H) {
        high()
    } else {
        mask.strength
    };

    text.0 = format!(
        "Effect (1,2,3): {:?}, strength (↑↓, [L]ow, [H]igh): {:.2}",
        mask.variant, mask.strength
    );
}

#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::post_processing::{
    blur::Blur, chromatic_aberration::ChromaticAberration, flip::Flip, lut::Lut, masks::Mask,
    pixelate::Pixelate, raindrops::Raindrops, wave::Wave, PostProcessingPlugin, VfxOrdering,
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

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        Blur::default(),
        VfxOrdering::<Blur>::new(0.0),
        Pixelate::default(),
        VfxOrdering::<Pixelate>::new(1.0),
        ChromaticAberration::default(),
        VfxOrdering::<ChromaticAberration>::new(2.0),
        Flip::Vertical,
        VfxOrdering::<Flip>::new(2.0),
        Mask::vignette(),
        VfxOrdering::<Mask>::new(3.0),
        Wave {
            waves_x: 3.,
            speed_x: 0.5,
            amplitude_x: 0.1,
            ..Default::default()
        },
        Raindrops::default(),
        VfxOrdering::<Raindrops>::new(4.0),
        Lut::arctic(),
    ));
}

fn update(
    keyboard_input: Res<Input<KeyCode>>,
    mut blur: Query<&Blur>,
    // mut text: ResMut<examples_common::ExampleText>,
) {
    let mut pixelate_diff = 0.0;

    if keyboard_input.just_pressed(KeyCode::P) {
        // passthrough.0 = !passthrough.0;
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        pixelate_diff = 1.0;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        pixelate_diff = -1.0;
    }

    // pixelate.block_size += pixelate_diff;
    // pixelate.block_size = 1.0_f32.max(pixelate.block_size);

    // text.0 = format!(
    //     "Pixelate block size (↑↓): {:.2?}, [P]assthrough: {:?}",
    //     pixelate.block_size, passthrough.0
    // );
}

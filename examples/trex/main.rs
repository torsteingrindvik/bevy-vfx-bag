#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::wave::{Wave, WavePassthrough, WavePlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

#[derive(Debug, Resource, Default)]
struct SlowerTime(Time);

fn main() {
    App::new()
        .add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .insert_resource(Wave {
            waves_x: 2.0,
            waves_y: 0.1,
            speed_x: 30.,
            speed_y: 20.,
            amplitude_x: 0.01,
            amplitude_y: 0.01,
        })
        .add_plugin(WavePlugin)
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

fn update(
    time: Res<Time>,
    mut earthquake_passthrough: ResMut<WavePassthrough>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    let fract = time.elapsed_seconds().fract();

    let relax = fract < 0.8;

    if relax {
        text.0 = "Is that a T-Rex approaching?!".into();
    } else {
        text.0 = "<GROUND SHAKE>".into();
    }

    earthquake_passthrough.0 = relax;
}

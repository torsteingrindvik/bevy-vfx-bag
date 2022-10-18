#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::{
        chromatic_aberration::*,
        flip::{Flip, FlipPlugin},
        lut::{Lut, Lut3d, LutPlugin},
        mask::{Mask, MaskPlugin},
        rainrops::RaindropsPlugin,
        wave::{Wave, WavePlugin},
    },
    BevyVfxBagPlugin, PostProcessingInput,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        // Effect stack start: First add main plugin
        .add_plugin(BevyVfxBagPlugin)
        // Effect: Flip
        .insert_resource(Flip::Horizontal)
        .add_plugin(FlipPlugin)
        // Effect: Raindrops
        .add_plugin(RaindropsPlugin)
        // Effect: Chromatic Aberration
        .insert_resource(ChromaticAberration {
            magnitude_r: 0.003,
            magnitude_g: 0.003,
            magnitude_b: 0.003,
            ..default()
        })
        .add_plugin(ChromaticAberrationPlugin)
        // Effect: Wave
        .insert_resource(Wave {
            waves_x: 1.,
            speed_x: 0.1,
            amplitude_x: 0.07,
            waves_y: 10.,
            speed_y: 0.3,
            amplitude_y: 0.01,
        })
        .add_plugin(WavePlugin)
        // Effect: LUT
        .add_plugin(LutPlugin)
        // Effect: Masks > vignette
        .insert_resource(Mask::new_vignette())
        .add_plugin(MaskPlugin)
        // Effect stack over, on to systems.
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

#[derive(Resource)]
struct LutImage(Handle<Image>);

fn startup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(PostProcessingInput);

    commands.insert_resource(LutImage(assets.load("luts/arctic.png")));

    text.0 = "Horizontal flip > raindrops > chromatic aberration > waves > LUT > Vignette".into();
}

fn update(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    lut_image: Res<LutImage>,
    mut lut: ResMut<Lut>,
) {
    // When the .png file has loaded,
    // set it as the LUT 3D texture.
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            if handle == &lut_image.0 {
                lut.set_texture(&Lut3d::from_image(&mut assets, handle));
            }
        }
    }
}

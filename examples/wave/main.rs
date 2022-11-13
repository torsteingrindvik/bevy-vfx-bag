#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::wave::{Wave, WavePlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(WavePlugin)
        .insert_resource(Presets(vec![
            Preset::RowingOnLand,
            Preset::LazyX,
            Preset::FightingTheWind,
            Preset::RoughSeas,
            Preset::Earthquake,
        ]))
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

#[derive(Debug, Clone, Copy)]
enum Preset {
    // That's what it looks like.
    RowingOnLand,

    // Relaxing horizontal waves.
    LazyX,

    // You can do it!
    FightingTheWind,

    // Stay on your feet!
    RoughSeas,

    // Great quad workout!
    Earthquake,
}

#[derive(Resource)]
struct Presets(Vec<Preset>);

impl From<Preset> for Wave {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::RowingOnLand => Self {
                waves_x: 1.,
                waves_y: 20.,
                speed_x: 1.3,
                speed_y: 20.,
                amplitude_x: 0.25,
                amplitude_y: 0.005,
            },
            Preset::LazyX => Self {
                waves_x: 1.,
                speed_x: 0.5,
                amplitude_x: 0.1,
                ..default()
            },
            Preset::FightingTheWind => Self {
                waves_y: 10.,
                speed_y: 14.,
                amplitude_y: 0.002,
                ..default()
            },
            Preset::RoughSeas => Self {
                waves_x: 1.,
                waves_y: 2.,
                speed_x: 1.,
                speed_y: 1.,
                amplitude_x: 0.03,
                amplitude_y: 0.04,
            },
            Preset::Earthquake => Self {
                waves_x: 2.0,
                waves_y: 0.1,
                speed_x: 1.,
                speed_y: 13.,
                amplitude_x: 0.02,
                amplitude_y: 0.03,
            },
        }
    }
}

// Cycle through some presets of interesting wave effects
// by pressing some input keys.
fn update(
    mut preset_index: Local<isize>,
    presets: Res<Presets>,
    keyboard_input: Res<Input<KeyCode>>,
    mut wave: ResMut<Wave>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    if keyboard_input.just_pressed(KeyCode::Left) {
        *preset_index -= 1;
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        *preset_index += 1;
    }
    let num_presets = presets.0.len() as isize;
    *preset_index = preset_index.clamp(0, num_presets - 1);

    let preset = presets.0[*preset_index as usize];

    *wave = preset.into();

    text.0 = format!(
        "(Press ←→) Preset {}/{num_presets}: {preset:?}",
        *preset_index + 1
    );
}

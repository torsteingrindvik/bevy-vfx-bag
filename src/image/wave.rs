use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin},
};

use crate::{new_effect_state, setup_effect, EffectState, HasEffectState};

/// This plugin allows creating a wave across the image.
/// A wave can be customized in the X and Y axes for interesting effects.
pub struct WavePlugin;

/// Wave parameters.
///
/// Note that the parameters for the X axis causes a wave effect
/// towards the left- and right sides of the screen.
/// For example, if we have 1 wave in the X axis,
/// we will have one part of the screen stretched towards the right
/// horizontally, and one part stretched towards the left.
#[derive(Default, Debug, Copy, Clone, Resource, ShaderType)]
pub struct Wave {
    /// How many waves in the x axis.
    pub waves_x: f32,

    /// How many waves in the y axis.
    pub waves_y: f32,

    /// How fast the x axis waves oscillate.
    pub speed_x: f32,

    /// How fast the y axis waves oscillate.
    pub speed_y: f32,

    /// How much displacement the x axis waves cause.
    pub amplitude_x: f32,

    /// How much displacement the y axis waves cause.
    pub amplitude_y: f32,
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "79fa38f9-ca04-4e59-83f9-da0de45afc04"]
struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    wave: Wave,

    state: EffectState,
}

impl HasEffectState for WaveMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for WaveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wave.wgsl".into()
    }
}

impl FromWorld for WaveMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let wave = world.get_resource::<Wave>().expect("Wave resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            state,
            wave: *wave,
        }
    }
}

fn update_wave(mut wave_materials: ResMut<Assets<WaveMaterial>>, wave: Res<Wave>) {
    if !wave.is_changed() {
        return;
    }

    for (_, material) in wave_materials.iter_mut() {
        material.wave = *wave;
    }
}

impl Plugin for WavePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("WavePlugin build").entered();

        app.init_resource::<Wave>()
            .init_resource::<WaveMaterial>()
            .add_plugin(Material2dPlugin::<WaveMaterial>::default())
            .add_startup_system(setup_effect::<WaveMaterial>)
            .add_system(update_wave);
    }
}

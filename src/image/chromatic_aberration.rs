use std::f32::consts::PI;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin},
};

use crate::{new_effect_state, setup_effect, EffectState, HasEffectState};

/// This plugin allows using chromatic aberration.
/// This offsets the RGB channels with some magnitude
/// and direction seperately.
pub struct ChromaticAberrationPlugin;

/// Chromatic aberration parameters.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct ChromaticAberration {
    /// The direction (in UV space) the red channel is offset in.
    /// Will be normalized.
    pub dir_r: Vec2,

    /// How far (in UV space) the red channel should be displaced.
    pub magnitude_r: f32,

    /// The direction (in UV space) the green channel is offset in.
    /// Will be normalized.
    pub dir_g: Vec2,

    /// How far (in UV space) the green channel should be displaced.
    pub magnitude_g: f32,

    /// The direction (in UV space) the blue channel is offset in.
    /// Will be normalized.
    pub dir_b: Vec2,

    /// How far (in UV space) the blue channel should be displaced.
    pub magnitude_b: f32,
}

impl Default for ChromaticAberration {
    fn default() -> Self {
        let one_third = (2. / 3.) * PI;

        Self {
            dir_r: Vec2::from_angle(0. * one_third),
            magnitude_r: 0.01,
            dir_g: Vec2::from_angle(1. * one_third),
            magnitude_g: 0.01,
            dir_b: Vec2::from_angle(2. * one_third),
            magnitude_b: 0.01,
        }
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "1c857de0-74e6-42a4-a1b4-0e0f1564a880"]
struct ChromaticAberrationMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    chromatic_aberration: ChromaticAberration,

    state: EffectState,
}

impl HasEffectState for ChromaticAberrationMaterial {
    fn state(&self) -> EffectState {
        self.state.clone()
    }
}

impl Material2d for ChromaticAberrationMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chromatic-aberration.wgsl".into()
    }
}

impl FromWorld for ChromaticAberrationMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let ca = world
            .get_resource::<ChromaticAberration>()
            .expect("Chromatic Aberration resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            chromatic_aberration: *ca,
            state,
        }
    }
}

fn update_chromatic_aberration(
    mut chromatic_aberration_materials: ResMut<Assets<ChromaticAberrationMaterial>>,
    chromatic_aberration: Res<ChromaticAberration>,
) {
    if !chromatic_aberration.is_changed() {
        return;
    }

    let mut ca = *chromatic_aberration;

    ca.dir_r = ca.dir_r.normalize_or_zero();
    ca.dir_g = ca.dir_g.normalize_or_zero();
    ca.dir_b = ca.dir_b.normalize_or_zero();

    for (_, material) in chromatic_aberration_materials.iter_mut() {
        material.chromatic_aberration = ca;
    }
}

impl Plugin for ChromaticAberrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("ChromaticAberrationPlugin build").entered();

        app.init_resource::<ChromaticAberration>()
            .init_resource::<ChromaticAberrationMaterial>()
            .add_plugin(Material2dPlugin::<ChromaticAberrationMaterial>::default())
            .add_startup_system(setup_effect::<ChromaticAberrationMaterial>)
            .add_system(update_chromatic_aberration);
    }
}

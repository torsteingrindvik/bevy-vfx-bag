use std::f32::consts::PI;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use crate::{
    load_asset_if_no_dev_feature, new_effect_state, passthrough, setup_effect, shader_ref,
    EffectState, HasEffectState, Passthrough,
};

const CHROMATIC_ABERRATION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9124131622872249345);

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

/// If this effect should not be enabled, i.e. it should just
/// pass through the input image.
#[derive(Debug, Resource, Default, PartialEq, Eq, Hash, Clone)]
pub struct ChromaticAberrationPassthrough(pub bool);

impl Passthrough for ChromaticAberrationPassthrough {
    fn passthrough(&self) -> bool {
        self.0
    }
}

impl From<&ChromaticAberrationMaterial> for ChromaticAberrationPassthrough {
    fn from(material: &ChromaticAberrationMaterial) -> Self {
        Self(material.passthrough)
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "1c857de0-74e6-42a4-a1b4-0e0f1564a880"]
#[bind_group_data(ChromaticAberrationPassthrough)]
struct ChromaticAberrationMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    chromatic_aberration: ChromaticAberration,

    state: EffectState,

    passthrough: bool,
}

impl HasEffectState for ChromaticAberrationMaterial {
    fn state(&self) -> EffectState {
        self.state.clone()
    }
}

impl Material2d for ChromaticAberrationMaterial {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(
            CHROMATIC_ABERRATION_SHADER_HANDLE,
            "shaders/chromatic-aberration.wgsl"
        )
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        passthrough(descriptor, &key);

        Ok(())
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
            passthrough: false,
        }
    }
}

fn update_chromatic_aberration(
    mut chromatic_aberration_materials: ResMut<Assets<ChromaticAberrationMaterial>>,
    chromatic_aberration: Res<ChromaticAberration>,

    passthrough: Res<ChromaticAberrationPassthrough>,
) {
    if !chromatic_aberration.is_changed() && !passthrough.is_changed() {
        return;
    }

    let mut ca = *chromatic_aberration;

    ca.dir_r = ca.dir_r.normalize_or_zero();
    ca.dir_g = ca.dir_g.normalize_or_zero();
    ca.dir_b = ca.dir_b.normalize_or_zero();

    for (_, material) in chromatic_aberration_materials.iter_mut() {
        material.chromatic_aberration = ca;
        material.passthrough = passthrough.0;
    }
}

impl Plugin for ChromaticAberrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("ChromaticAberrationPlugin build").entered();

        load_asset_if_no_dev_feature!(
            app,
            CHROMATIC_ABERRATION_SHADER_HANDLE,
            "../../assets/shaders/chromatic-aberration.wgsl"
        );

        app.init_resource::<ChromaticAberration>()
            .init_resource::<ChromaticAberrationMaterial>()
            .init_resource::<ChromaticAberrationPassthrough>()
            .add_plugin(Material2dPlugin::<ChromaticAberrationMaterial>::default())
            .add_startup_system(setup_effect::<ChromaticAberrationMaterial>)
            .add_system(update_chromatic_aberration);
    }
}

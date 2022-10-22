use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin},
};

use crate::{new_effect_state, setup_effect, EffectState, HasEffectState};

const FLIP_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6628355331557851282);

/// This plugin allows flipping the rendered scene horizontally and/or vertically.
/// Add this plugin to the [`App`] in order to use it.
pub struct FlipPlugin;

/// Which way to flip the texture.
#[derive(Debug, Default, Copy, Clone, Resource)]
pub enum Flip {
    /// Don't flip.
    #[default]
    None,

    /// Flip horizontally.
    Horizontal,

    /// Flip vertically.
    Vertical,

    /// Flip both axes.
    HorizontalVertical,
}

impl From<Flip> for FlipUniform {
    fn from(flip: Flip) -> Self {
        let uv = match flip {
            Flip::None => [0.0, 0.0],
            Flip::Horizontal => [1.0, 0.0],
            Flip::Vertical => [0.0, 1.0],
            Flip::HorizontalVertical => [1.0, 1.0],
        };

        Self { x: uv[0], y: uv[1] }
    }
}

#[derive(Debug, Clone, ShaderType)]
struct FlipUniform {
    x: f32,
    y: f32,
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "70bc3d3b-46e2-40ea-bedc-e0d73ffdd3fd"]
struct FlipMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    flip: FlipUniform,

    state: EffectState,
}

impl HasEffectState for FlipMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for FlipMaterial {
    fn fragment_shader() -> ShaderRef {
        if cfg!(feature = "dev") {
            "shaders/flip.wgsl".into()
        } else {
            FLIP_SHADER_HANDLE.typed().into()
        }
    }
}

impl FromWorld for FlipMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let flip = world.get_resource::<Flip>().expect("Flip resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            flip: FlipUniform::from(*flip),
            state,
        }
    }
}

fn update_flip(mut flip_materials: ResMut<Assets<FlipMaterial>>, flip: Res<Flip>) {
    if !flip.is_changed() {
        return;
    }

    for (_, material) in flip_materials.iter_mut() {
        material.flip = (*flip).into();
    }
}

impl Plugin for FlipPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("FlipPlugin build").entered();

        if !cfg!(feature = "dev") {
            use bevy::asset::load_internal_asset;
            load_internal_asset!(
                app,
                FLIP_SHADER_HANDLE,
                "../../assets/shaders/flip.wgsl",
                Shader::from_wgsl
            );
        }

        app.init_resource::<Flip>()
            .init_resource::<FlipMaterial>()
            .add_plugin(Material2dPlugin::<FlipMaterial>::default())
            .add_startup_system(setup_effect::<FlipMaterial>)
            .add_system(update_flip);
    }
}

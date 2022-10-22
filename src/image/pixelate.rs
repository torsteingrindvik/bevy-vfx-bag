//! Credits to Ben Cloward, see [the YouTube video](https://www.youtube.com/watch?v=x95xhWCxBb4).

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin},
};

use crate::{new_effect_state, setup_effect, shader_ref, EffectState, HasEffectState};

const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12823700631286738286);

/// This plugin allows pixelating the scene.
/// Add this plugin to the [`App`] in order to use it.
pub struct PixelatePlugin;

/// Blur parameters.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct Pixelate {
    /// How many pixels in the width and height in a block after pixelation.
    /// One block has a constant color within it.
    ///
    /// The shader sets a lower bound to 1.0, since that would not change the outcome.
    pub block_size: f32,
}

impl Default for Pixelate {
    fn default() -> Self {
        Self { block_size: 4.0 }
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "6370db4a-50ad-11ed-9fb9-3fa3e5f909b7"]
struct PixelateMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    pixelate: Pixelate,

    state: EffectState,
}

impl Material2d for PixelateMaterial {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(PIXELATE_SHADER_HANDLE, "shaders/pixelate.wgsl")
    }
}

impl HasEffectState for PixelateMaterial {
    fn state(&self) -> EffectState {
        self.state.clone()
    }
}

impl FromWorld for PixelateMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let pixelate = world.get_resource::<Pixelate>().expect("Pixelate resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            state,
            pixelate: *pixelate,
        }
    }
}

fn update_pixelate(
    mut pixelate_materials: ResMut<Assets<PixelateMaterial>>,
    pixelate: Res<Pixelate>,
) {
    if !pixelate.is_changed() {
        return;
    }

    for (_, material) in pixelate_materials.iter_mut() {
        material.pixelate = *pixelate;
    }
}

impl Plugin for PixelatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("PixelatePlugin build").entered();

        if !cfg!(feature = "dev") {
            use bevy::asset::load_internal_asset;
            load_internal_asset!(
                app,
                PIXELATE_SHADER_HANDLE,
                "../../assets/shaders/pixelate.wgsl",
                Shader::from_wgsl
            );
        }

        app.init_resource::<Pixelate>()
            .init_resource::<PixelateMaterial>()
            .add_plugin(Material2dPlugin::<PixelateMaterial>::default())
            .add_startup_system(setup_effect::<PixelateMaterial>)
            .add_system(update_pixelate);
    }
}

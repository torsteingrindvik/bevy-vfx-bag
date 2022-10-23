//! Credits to Ben Cloward, see [the YouTube video](https://www.youtube.com/watch?v=x95xhWCxBb4).

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
    load_asset_if_no_dev_feature, new_effect_state, setup_effect, shader_ref, EffectState,
    HasEffectState,
};

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

/// If this effect should not be enabled, i.e. it should just
/// pass through the input image.
#[derive(Debug, Resource, Default, PartialEq, Eq, Hash, Clone)]
pub struct PixelatePassthrough(pub bool);

impl From<&PixelateMaterial> for PixelatePassthrough {
    fn from(material: &PixelateMaterial) -> Self {
        Self(material.passthrough)
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "6370db4a-50ad-11ed-9fb9-3fa3e5f909b7"]
#[bind_group_data(PixelatePassthrough)]
struct PixelateMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    pixelate: Pixelate,

    state: EffectState,

    passthrough: bool,
}

impl Material2d for PixelateMaterial {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(PIXELATE_SHADER_HANDLE, "shaders/pixelate.wgsl")
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.0 {
            descriptor
                .fragment
                .as_mut()
                .expect("Should have fragment state")
                .shader_defs
                .push("PASSTHROUGH".into());
        }

        Ok(())
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
            passthrough: false,
        }
    }
}

fn update_pixelate(
    mut pixelate_materials: ResMut<Assets<PixelateMaterial>>,
    pixelate: Res<Pixelate>,
    passthrough: Res<PixelatePassthrough>,
) {
    if !pixelate.is_changed() && !passthrough.is_changed() {
        return;
    }

    for (_, material) in pixelate_materials.iter_mut() {
        material.pixelate = *pixelate;
        material.passthrough = passthrough.0;
    }
}

impl Plugin for PixelatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("PixelatePlugin build").entered();

        load_asset_if_no_dev_feature!(
            app,
            PIXELATE_SHADER_HANDLE,
            "../../assets/shaders/pixelate.wgsl"
        );

        app.init_resource::<Pixelate>()
            .init_resource::<PixelateMaterial>()
            .init_resource::<PixelatePassthrough>()
            .add_plugin(Material2dPlugin::<PixelateMaterial>::default())
            .add_startup_system(setup_effect::<PixelateMaterial>)
            .add_system(update_pixelate);
    }
}

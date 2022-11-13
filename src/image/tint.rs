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

const TINT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12511098424072325900);

/// This plugin allows tinting the image using [`Color`].
/// Add this plugin to the [`App`] in order to use it.
pub struct TintPlugin;

/// Tint paramters.
#[derive(Debug, Default, Copy, Clone, Resource, ShaderType)]
pub struct Tint {
    /// Tint color.
    pub color: Color,
}

/// If this effect should not be enabled, i.e. it should just
/// pass through the input image.
#[derive(Debug, Resource, Default, PartialEq, Eq, Hash, Clone)]
pub struct TintPassthrough(pub bool);

impl Passthrough for TintPassthrough {
    fn passthrough(&self) -> bool {
        self.0
    }
}

impl From<&TintMaterial> for TintPassthrough {
    fn from(material: &TintMaterial) -> Self {
        Self(material.passthrough)
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "7fd3e1f2-57c8-11ed-a0de-8772f8221456"]
#[bind_group_data(TintPassthrough)]
struct TintMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    tint: Tint,

    state: EffectState,

    passthrough: bool,
}

impl HasEffectState for TintMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for TintMaterial {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(TINT_SHADER_HANDLE, "shaders/tint.wgsl")
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

impl FromWorld for TintMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let tint = *world.get_resource::<Tint>().expect("Tint resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            tint,
            state,
            passthrough: false,
        }
    }
}

fn update_tint(
    mut tint_materials: ResMut<Assets<TintMaterial>>,
    passthrough: Res<TintPassthrough>,
    tint: Res<Tint>,
) {
    if !tint.is_changed() && !passthrough.is_changed() {
        return;
    }

    for (_, material) in tint_materials.iter_mut() {
        material.tint = *tint;
        material.passthrough = passthrough.0;
    }
}

impl Plugin for TintPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("TintPlugin build").entered();

        load_asset_if_no_dev_feature!(app, TINT_SHADER_HANDLE, "../../assets/shaders/tint.wgsl");

        app.init_resource::<Tint>()
            .init_resource::<TintMaterial>()
            .init_resource::<TintPassthrough>()
            .add_plugin(Material2dPlugin::<TintMaterial>::default())
            .add_startup_system(setup_effect::<TintMaterial>)
            .add_system(update_tint);
    }
}

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

const BLUR_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 14860840837524393207);

/// This plugin allows blurring the scene.
/// Add this plugin to the [`App`] in order to use it.
pub struct BlurPlugin;

/// Blur parameters.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct Blur {
    /// How blurry the output image should be.
    /// If `0.0`, no blur is applied.
    /// `1.0` is "fully blurred", but higher values will produce interesting results.
    pub amount: f32,

    /// How far away the blur should sample points away from the origin point
    /// when blurring.
    /// This is in UV coordinates, so small (positive) values are expected (`0.01` is a good start).
    pub kernel_radius: f32,
}

impl Default for Blur {
    fn default() -> Self {
        Self {
            amount: Default::default(),
            kernel_radius: 0.01,
        }
    }
}

/// If this effect should not be enabled, i.e. it should just
/// pass through the input image.
#[derive(Debug, Resource, Default, PartialEq, Eq, Hash, Clone)]
pub struct BlurPassthrough(pub bool);

impl Passthrough for BlurPassthrough {
    fn passthrough(&self) -> bool {
        self.0
    }
}

impl From<&BlurMaterial> for BlurPassthrough {
    fn from(material: &BlurMaterial) -> Self {
        Self(material.passthrough)
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "1b35a535-d428-4822-aba5-15e104ea80b5"]
#[bind_group_data(BlurPassthrough)]
struct BlurMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    blur: Blur,

    state: EffectState,

    passthrough: bool,
}

impl Material2d for BlurMaterial {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(BLUR_SHADER_HANDLE, "shaders/blur.wgsl")
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

impl HasEffectState for BlurMaterial {
    fn state(&self) -> EffectState {
        self.state.clone()
    }
}

impl FromWorld for BlurMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let blur = world.get_resource::<Blur>().expect("Blur resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            blur: *blur,
            state,
            passthrough: false,
        }
    }
}

fn update_blur(
    mut blur_materials: ResMut<Assets<BlurMaterial>>,
    blur: Res<Blur>,
    passthrough: Res<BlurPassthrough>,
) {
    if !blur.is_changed() && !passthrough.is_changed() {
        return;
    }

    for (_, material) in blur_materials.iter_mut() {
        material.blur = *blur;
        material.passthrough = passthrough.0;
    }
}

impl Plugin for BlurPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("BlurPlugin build").entered();

        load_asset_if_no_dev_feature!(app, BLUR_SHADER_HANDLE, "../../assets/shaders/blur.wgsl");

        app.init_resource::<Blur>()
            .init_resource::<BlurMaterial>()
            .init_resource::<BlurPassthrough>()
            .add_plugin(Material2dPlugin::<BlurMaterial>::default())
            .add_startup_system(setup_effect::<BlurMaterial>)
            .add_system(update_blur);
    }
}

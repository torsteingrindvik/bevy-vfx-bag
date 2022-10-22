//! Credits to Ben Cloward, see [the YouTube video](https://www.youtube.com/watch?v=HcMFgJas0yg).

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use crate::{new_effect_state, setup_effect, EffectState, HasEffectState};

const MASKS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12949814029375825065);

/// This plugin allows adding a mask effect to a texture.
pub struct MaskPlugin;

/// This resource controls the parameters of the effect.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum MaskVariant {
    /// Rounded square type mask.
    ///
    /// One use of this mask is to post-process _other_ effects which might
    /// have artifacts around the edges.
    /// This mask can then attenuate that effect and thus remove the effects of the
    /// artifacts.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    3.0 almost loses the square shape.
    /// High end:   100.0 has almost sharp, thin edges.
    Square,

    /// Rounded square type mask, but more oval like a CRT television.
    ///
    /// This effect can be used as a part of a retry-style effect.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    3000.0 almost loses the CRT shape.
    /// High end:   500000.0 "inflates" the effect a bit.
    Crt,

    /// Vignette mask.
    ///
    /// This effect can be used to replicate the classic photography
    /// light attenuation seen at the edges of photos.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    0.10 gives a very subtle effect.
    /// High end:   1.50 is almost a spotlight in the middle of the screen.
    Vignette,
}

/// This resource controls the parameters of the effect.
#[derive(Debug, Resource, Clone)]
pub struct Mask {
    /// The strength parameter of the mask in use.
    ///
    /// See [`MaskVariant`] for guidelines on which range of values make sense
    /// for the variant in use.
    ///
    /// Run the masks example to experiment with these values interactively.
    pub strength: f32,

    /// Which [`MaskVariant`] to produce.
    pub variant: MaskVariant,
}

impl Mask {
    /// Create a new square mask with a reasonable strength value.
    pub fn new_square() -> Self {
        Self {
            strength: 20.,
            variant: MaskVariant::Square,
        }
    }

    /// Create a new CRT mask with a reasonable strength value.
    pub fn new_crt() -> Self {
        Self {
            strength: 80000.,
            variant: MaskVariant::Crt,
        }
    }

    /// Create a new vignette mask with a reasonable strength value.
    pub fn new_vignette() -> Self {
        Self {
            strength: 0.66,
            variant: MaskVariant::Vignette,
        }
    }
}

impl From<&MaskMaterial> for MaskVariant {
    fn from(mask_material: &MaskMaterial) -> Self {
        mask_material.variant
    }
}

#[derive(AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "9ca04144-a3e1-40b4-93a7-91424159f612"]
#[bind_group_data(MaskVariant)]
struct MaskMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    strength: f32,

    variant: MaskVariant,

    state: EffectState,
}

impl HasEffectState for MaskMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for MaskMaterial {
    fn fragment_shader() -> ShaderRef {
        if cfg!(feature = "dev") {
            "shaders/masks.wgsl".into()
        } else {
            MASKS_SHADER_HANDLE.typed().into()
        }
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let def = match key.bind_group_data {
            MaskVariant::Square => "SQUARE",
            MaskVariant::Crt => "CRT",
            MaskVariant::Vignette => "VIGNETTE",
        };
        descriptor
            .fragment
            .as_mut()
            .expect("Should have fragment state")
            .shader_defs
            .push(def.into());

        Ok(())
    }
}

impl FromWorld for MaskMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let mask = world.get_resource::<Mask>().expect("Mask resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            strength: mask.strength,
            variant: mask.variant,
            state,
        }
    }
}

fn update_mask(mut mask_materials: ResMut<Assets<MaskMaterial>>, mask: Res<Mask>) {
    if !mask.is_changed() {
        return;
    }

    for (_, material) in mask_materials.iter_mut() {
        material.variant = mask.variant;
        material.strength = mask.strength;
    }
}

impl Plugin for MaskPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("MaskPlugin build").entered();

        if !cfg!(feature = "dev") {
            use bevy::asset::load_internal_asset;
            load_internal_asset!(
                app,
                MASKS_SHADER_HANDLE,
                "../../assets/shaders/masks.wgsl",
                Shader::from_wgsl
            );
        }

        app.init_resource::<MaskMaterial>()
            .add_plugin(Material2dPlugin::<MaskMaterial>::default())
            .add_startup_system(setup_effect::<MaskMaterial>)
            .add_system(update_mask);
    }
}

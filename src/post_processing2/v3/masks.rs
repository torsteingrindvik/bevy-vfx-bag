use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType},
    },
};
use bevy::{render::render_resource::ShaderRef, sprite::Material2d};

use crate::shader_ref;

use super::post_processing_plugin;

pub(crate) const MASK_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1649866799156783187);

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_material)
            .add_plugin(post_processing_plugin::Plugin::<Mask, MaskSettings>::default());
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Mask>>,
    cameras: Query<(Entity, &MaskSettings), (With<Camera>, Without<Handle<Mask>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Mask {
            mask: MaskUniform::from(*settings),
            variant: settings.variant,
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
pub(crate) struct MaskUniform {
    pub(crate) strength: f32,
}

impl From<MaskSettings> for MaskUniform {
    fn from(mask: MaskSettings) -> Self {
        Self {
            strength: mask.strength,
        }
    }
}

/// TODO
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MaskKey(MaskVariant);

impl From<&Mask> for MaskKey {
    fn from(material: &Mask) -> Self {
        Self(material.variant)
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "1f0d1510-7a04-11ed-bf61-b3427e523ca2"]
#[bind_group_data(MaskKey)]
pub struct Mask {
    #[uniform(0)]
    pub(crate) mask: MaskUniform,

    pub(crate) variant: MaskVariant,
}

impl Material2d for Mask {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(MASK_SHADER_HANDLE, "shaders/masks3.wgsl")
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let def = match key.bind_group_data.0 {
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
        info!("Specializing mask shader with {:?}", def);

        Ok(())
    }
}

/// This controls the parameters of the effect.
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

/// TODO
#[derive(Debug, Copy, Clone, Component)]
pub struct MaskSettings {
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

impl MaskSettings {
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

impl Default for MaskSettings {
    fn default() -> Self {
        Self::new_vignette()
    }
}

impl ExtractComponent for MaskSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

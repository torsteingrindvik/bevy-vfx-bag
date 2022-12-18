use crate::shader_ref;

use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
    },
    sprite::Material2d,
};

use super::post_processing_plugin;

pub(crate) const BLUR_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11044253213698850613);

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_material)
            .add_plugin(post_processing_plugin::Plugin::<Blur, BlurSettings>::default());
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Blur>>,
    cameras: Query<(Entity, &BlurSettings), (With<Camera>, Without<Handle<Blur>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Blur { blur: *settings });
        commands.entity(entity).insert(material_handle);
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "915653cc-7bba-11ed-b16d-8bf250a29317"]
pub struct Blur {
    #[uniform(0)]
    pub(crate) blur: BlurSettings,
}

impl Material2d for Blur {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(BLUR_SHADER_HANDLE, "shaders/blur3.wgsl")
    }
}

/// Blur settings.
#[derive(Debug, Copy, Clone, Component, ShaderType)]
pub struct BlurSettings {
    /// How blurry the output image should be.
    /// If `0.0`, no blur is applied.
    /// `1.0` is "fully blurred", but higher values will produce interesting results.
    pub amount: f32,

    /// How far away the blur should sample points away from the origin point
    /// when blurring.
    /// This is in UV coordinates, so small (positive) values are expected (`0.01` is a good start).
    pub kernel_radius: f32,
}

impl Default for BlurSettings {
    fn default() -> Self {
        Self {
            amount: 0.5,
            kernel_radius: 0.01,
        }
    }
}

impl ExtractComponent for BlurSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

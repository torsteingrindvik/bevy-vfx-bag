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

pub(crate) const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11093977931118718560);

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_material)
            .add_plugin(post_processing_plugin::Plugin::<Pixelate, PixelateSettings>::default());
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Pixelate>>,
    cameras: Query<(Entity, &PixelateSettings), (With<Camera>, Without<Handle<Pixelate>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Pixelate {
            pixelate: PixelateUniform {
                block_size: settings.block_size,
            },
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
pub(crate) struct PixelateUniform {
    pub(crate) block_size: f32,
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "485141dc-7890-11ed-9cf4-ab2aa4ee03b0"]
pub struct Pixelate {
    #[uniform(0)]
    pub(crate) pixelate: PixelateUniform,
}

impl Material2d for Pixelate {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(PIXELATE_SHADER_HANDLE, "shaders/pixelate3.wgsl")
    }
}

/// TODO
#[derive(Debug, Component, Clone, Copy)]
pub struct PixelateSettings {
    pub(crate) block_size: f32,
}

impl Default for PixelateSettings {
    fn default() -> Self {
        Self { block_size: 8.0 }
    }
}

impl ExtractComponent for PixelateSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

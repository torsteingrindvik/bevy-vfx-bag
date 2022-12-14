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

pub(crate) const FLIP_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1649866799156783187);

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_material)
            .add_plugin(post_processing_plugin::Plugin::<Flip, FlipSettings>::default());
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Flip>>,
    cameras: Query<(Entity, &FlipSettings), (With<Camera>, Without<Handle<Flip>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Flip {
            flip: FlipUniform::from(*settings),
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
pub(crate) struct FlipUniform {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl From<FlipSettings> for FlipUniform {
    fn from(flip: FlipSettings) -> Self {
        let uv = match flip {
            FlipSettings::None => [0.0, 0.0],
            FlipSettings::Horizontal => [1.0, 0.0],
            FlipSettings::Vertical => [0.0, 1.0],
            FlipSettings::HorizontalVertical => [1.0, 1.0],
        };

        Self { x: uv[0], y: uv[1] }
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "cbb6349e-7a00-11ed-8a99-63067e99f73e"]
pub struct Flip {
    #[uniform(0)]
    pub(crate) flip: FlipUniform,
}

impl Material2d for Flip {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(FLIP_SHADER_HANDLE, "shaders/flip3.wgsl")
    }
}

/// Which way to flip the texture.
#[derive(Debug, Default, Copy, Clone, Component)]
pub enum FlipSettings {
    /// Don't flip.
    None,

    /// Flip horizontally.
    #[default]
    Horizontal,

    /// Flip vertically.
    Vertical,

    /// Flip both axes.
    HorizontalVertical,
}

impl ExtractComponent for FlipSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

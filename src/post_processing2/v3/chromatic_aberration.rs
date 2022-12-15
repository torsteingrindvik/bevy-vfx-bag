use crate::shader_ref;
use std::f32::consts::PI;

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

pub(crate) const CHROMATIC_ABERRATION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4357337502039082134);

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_material)
            .add_plugin(post_processing_plugin::Plugin::<
                ChromaticAberration,
                ChromaticAberrationSettings,
            >::default());
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<ChromaticAberration>>,
    cameras: Query<
        (Entity, &ChromaticAberrationSettings),
        (With<Camera>, Without<Handle<ChromaticAberration>>),
    >,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(ChromaticAberration {
            chromatic_aberration: *settings,
        });
        commands.entity(entity).insert(material_handle);
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "c3ca158a-7bbc-11ed-aa78-8bb169d04615"]
pub struct ChromaticAberration {
    #[uniform(0)]
    pub(crate) chromatic_aberration: ChromaticAberrationSettings,
}

impl Material2d for ChromaticAberration {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(
            CHROMATIC_ABERRATION_SHADER_HANDLE,
            "shaders/chromatic-aberration3.wgsl"
        )
    }
}

/// Chromatic Aberration settings.
#[derive(Debug, Copy, Clone, Component, ShaderType)]
pub struct ChromaticAberrationSettings {
    /// The direction (in UV space) the red channel is offset in.
    /// Will be normalized.
    pub dir_r: Vec2,

    /// How far (in UV space) the red channel should be displaced.
    pub magnitude_r: f32,

    /// The direction (in UV space) the green channel is offset in.
    /// Will be normalized.
    pub dir_g: Vec2,

    /// How far (in UV space) the green channel should be displaced.
    pub magnitude_g: f32,

    /// The direction (in UV space) the blue channel is offset in.
    /// Will be normalized.
    pub dir_b: Vec2,

    /// How far (in UV space) the blue channel should be displaced.
    pub magnitude_b: f32,
}

impl Default for ChromaticAberrationSettings {
    fn default() -> Self {
        let one_third = (2. / 3.) * PI;

        Self {
            dir_r: Vec2::from_angle(0. * one_third),
            magnitude_r: 0.01,
            dir_g: Vec2::from_angle(1. * one_third),
            magnitude_g: 0.01,
            dir_b: Vec2::from_angle(2. * one_third),
            magnitude_b: 0.01,
        }
    }
}

impl ExtractComponent for ChromaticAberrationSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

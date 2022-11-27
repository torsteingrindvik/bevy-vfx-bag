use crate::{load_shader, post_processing2::util::PostProcessingPlugin};
use bevy::{
    asset::load_internal_asset,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

use super::util;

const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17030123524594138798);

/// Pixelation parameters.
#[derive(Component, Clone)]
pub struct Pixelate {
    /// Whether the effect should run or not.
    pub enabled: bool,

    /// How many pixels in the width and height in a block after pixelation.
    /// One block has a constant color within it.
    ///
    /// The shader sets a lower bound to 1.0, since that would not change the outcome.
    pub block_size: f32,
}

impl Default for Pixelate {
    fn default() -> Self {
        Self {
            enabled: true,
            block_size: 4.0,
        }
    }
}

impl ExtractComponent for Pixelate {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = PixelateUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            Some(PixelateUniform {
                block_size: item.block_size,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, ShaderType, Component, Clone)]
pub struct PixelateUniform {
    block_size: f32,
}

/// This plugin allows pixelating the image.
pub struct PixelatePlugin;

impl Plugin for PixelatePlugin {
    fn build(&self, app: &mut App) {
        let handle = load_shader!(app, PIXELATE_SHADER_HANDLE, "shaders/pixelate.wgsl");

        app.add_plugin(PostProcessingPlugin::<Pixelate>::new("Pixelate", handle));
    }
}

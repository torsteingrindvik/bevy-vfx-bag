use std::f32::consts::PI;

use crate::post_processing2::prelude::*;

const CHROMATIC_ABERRATION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17264544029431947860);

/// Chromatic aberration parameters.
#[derive(Component, Clone)]
pub struct ChromaticAberration {
    /// Whether the effect should run or not.
    pub enabled: bool,

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

impl Default for ChromaticAberration {
    fn default() -> Self {
        let one_third = (2. / 3.) * PI;

        Self {
            enabled: true,
            dir_r: Vec2::from_angle(0. * one_third),
            magnitude_r: 0.01,
            dir_g: Vec2::from_angle(1. * one_third),
            magnitude_g: 0.01,
            dir_b: Vec2::from_angle(2. * one_third),
            magnitude_b: 0.01,
        }
    }
}

impl ExtractComponent for ChromaticAberration {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = ChromaticAberrationUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            Some(ChromaticAberrationUniform {
                dir_r: item.dir_r,
                magnitude_r: item.magnitude_r,
                dir_g: item.dir_g,
                magnitude_g: item.magnitude_g,
                dir_b: item.dir_b,
                magnitude_b: item.magnitude_b,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, ShaderType, Component, Clone)]
pub struct ChromaticAberrationUniform {
    dir_r: Vec2,
    magnitude_r: f32,
    dir_g: Vec2,
    magnitude_g: f32,
    dir_b: Vec2,
    magnitude_b: f32,
}

/// This plugin allows adding chromatic aberration to the image.
pub struct ChromaticAberrationPlugin;

impl Plugin for ChromaticAberrationPlugin {
    fn build(&self, app: &mut App) {
        let handle = load_shader!(
            app,
            CHROMATIC_ABERRATION_SHADER_HANDLE,
            "shaders/chromatic-aberration.wgsl"
        );

        app.add_plugin(PostProcessingPlugin::<ChromaticAberration>::new(
            "Chromatic Aberration",
            handle,
        ));
    }
}

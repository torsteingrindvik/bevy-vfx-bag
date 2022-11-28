use crate::post_processing2::prelude::*;

const FLIP_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2678970587461377083);

/// Which way to flip the image.
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    /// Don't flip.
    None,

    /// Flip horizontally.
    Horizontal,

    /// Flip vertically.
    Vertical,

    /// Flip both axes.
    HorizontalVertical,
}

/// Flip parameters.
#[derive(Component, Clone)]
pub struct Flip {
    /// Whether the effect should run or not.
    pub enabled: bool,

    /// Which way to flip the image.
    pub direction: Direction,
}

impl Default for Flip {
    fn default() -> Self {
        Self {
            enabled: true,
            direction: Direction::None,
        }
    }
}

impl ExtractComponent for Flip {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = FlipUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            let uv = match item.direction {
                Direction::None => [0.0, 0.0],
                Direction::Horizontal => [1.0, 0.0],
                Direction::Vertical => [0.0, 1.0],
                Direction::HorizontalVertical => [1.0, 1.0],
            };

            Some(FlipUniform { x: uv[0], y: uv[1] })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, ShaderType, Component, Clone)]
pub struct FlipUniform {
    x: f32,
    y: f32,
}

/// This plugin allows flipping the image.
pub struct FlipPlugin;

impl Plugin for FlipPlugin {
    fn build(&self, app: &mut App) {
        let handle = load_shader!(app, FLIP_SHADER_HANDLE, "shaders/flip.wgsl");

        app.add_plugin(PostProcessingPlugin::<Flip>::new("Flip", handle));
    }
}

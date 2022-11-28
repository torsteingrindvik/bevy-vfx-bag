use crate::post_processing2::prelude::*;

const BLUR_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7487827790503578582);

/// Blur parameters.
#[derive(Component, Clone)]
pub struct Blur {
    /// Whether the effect should run or not.
    pub enabled: bool,

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
            enabled: true,
            amount: 1.0,
            kernel_radius: 0.01,
        }
    }
}

impl ExtractComponent for Blur {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = BlurUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            Some(BlurUniform {
                amount: item.amount,
                kernel_radius: item.kernel_radius,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, ShaderType, Component, Clone)]
pub struct BlurUniform {
    pub amount: f32,
    pub kernel_radius: f32,
}

/// This plugin allows blurring the image.
pub struct BlurPlugin;

impl Plugin for BlurPlugin {
    fn build(&self, app: &mut App) {
        let handle = load_shader!(app, BLUR_SHADER_HANDLE, "shaders/blur.wgsl");

        app.add_plugin(PostProcessingPlugin::<Blur>::new("Blur", handle));
    }
}

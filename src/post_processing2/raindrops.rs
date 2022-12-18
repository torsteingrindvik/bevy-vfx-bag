use crate::post_processing2::prelude::*;

const RAINDROPS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 18027540742851346464);

/// Raindrops parameters.
#[derive(Component, Clone)]
pub struct Raindrops {
    /// Whether the effect should run or not.
    pub enabled: bool,

    /// How fast the drops animate on screen.
    /// The default scaling allows for a gentle pitter-patter.
    pub time_scaling: f32,

    /// How much displacement each droplet has.
    pub intensity: f32,

    /// The overall size of the droplets.
    pub zoom: f32,
}

impl Default for Raindrops {
    fn default() -> Self {
        Self {
            enabled: true,
            time_scaling: 0.8,
            intensity: 0.03,
            zoom: 1.0,
        }
    }
}

impl ExtractComponent for Raindrops {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = RaindropsUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            Some(RaindropsUniform {
                time_scaling: item.time_scaling,
                intensity: item.intensity,
                zoom: item.zoom,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, ShaderType, Component, Clone)]
pub struct RaindropsUniform {
    pub time_scaling: f32,
    pub intensity: f32,
    pub zoom: f32,
}

/// This plugin allows adding raindrops to the image.
pub struct RaindropsPlugin;

impl Plugin for RaindropsPlugin {
    fn build(&self, app: &mut App) {
        let handle = load_shader!(app, RAINDROPS_SHADER_HANDLE, "shaders/raindrops.wgsl");

        app.add_plugin(PostProcessingPlugin::<Raindrops>::new("Raindrops", handle));
    }
}

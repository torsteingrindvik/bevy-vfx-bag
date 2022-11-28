use crate::post_processing2::prelude::*;

const WAVE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 16596876537040042100);

/// Wavw parameters.
#[derive(Component, Clone)]
pub struct Wave {
    /// Whether the effect should run or not.
    pub enabled: bool,

    /// How many waves in the x axis.
    pub waves_x: f32,

    /// How many waves in the y axis.
    pub waves_y: f32,

    /// How fast the x axis waves oscillate.
    pub speed_x: f32,

    /// How fast the y axis waves oscillate.
    pub speed_y: f32,

    /// How much displacement the x axis waves cause.
    pub amplitude_x: f32,

    /// How much displacement the y axis waves cause.
    pub amplitude_y: f32,
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            enabled: true,
            waves_x: 2.0,
            waves_y: 5.0,
            speed_x: 0.1,
            speed_y: 0.1,
            amplitude_x: 0.2,
            amplitude_y: 0.2,
        }
    }
}

impl ExtractComponent for Wave {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = WaveUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            Some(WaveUniform {
                waves_x: item.waves_x,
                waves_y: item.waves_y,
                speed_x: item.speed_x,
                speed_y: item.speed_y,
                amplitude_x: item.amplitude_x,
                amplitude_y: item.amplitude_y,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, ShaderType, Component, Clone)]
pub struct WaveUniform {
    waves_x: f32,
    waves_y: f32,
    speed_x: f32,
    speed_y: f32,
    amplitude_x: f32,
    amplitude_y: f32,
}

/// This plugin allows making waves across the image.
pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        let handle = load_shader!(app, WAVE_SHADER_HANDLE, "shaders/wave.wgsl");

        app.add_plugin(PostProcessingPlugin::<Wave>::new("Wave", handle));
    }
}

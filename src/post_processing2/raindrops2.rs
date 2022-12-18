use bevy::render::render_resource::AsBindGroup;

use crate::{load_image, post_processing2::prelude::*};

const RAINDROPS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 14785543643812289755);

const RAINDROPS_TEXTURE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 9363411587132811616);

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

#[derive(Debug, Clone, ShaderType, Component)]
#[doc(hidden)]
pub struct RaindropsUniform {
    time_scaling: f32,
    intensity: f32,
    zoom: f32,
}

#[derive(Debug, Clone, AsBindGroup, TypeUuid)]
#[uuid = "56a4bb14-70df-11ed-86ab-77eb86ef2583"]
pub(crate) struct RaindropsTexture {
    // #[uniform(0)]
    // raindrops: RaindropsUniform,
    #[texture(0)]
    #[sampler(1)]
    color_texture: Option<Handle<Image>>,
}
// #[doc(hidden)]
// #[derive(Debug, ShaderType, Component, Clone)]
// pub struct RaindropsUniform {
//     pub time_scaling: f32,
//     pub intensity: f32,
//     pub zoom: f32,
// }

/// This plugin allows adding raindrops to the image.
pub struct RaindropsPlugin;

#[doc(hidden)]
#[derive(Debug, Resource)]
pub struct RaindropsTextureHandle(Handle<Image>);

impl Plugin for RaindropsPlugin {
    fn build(&self, app: &mut App) {
        let shader_handle = load_shader!(app, RAINDROPS_SHADER_HANDLE, "shaders/raindrops.wgsl");
        let texture_handle = load_image!(app, RAINDROPS_TEXTURE_HANDLE, "textures/raindrops.tga");

        // let texture_handle = app
        //     .world
        //     .get_resource::<AssetServer>()
        //     .expect("AssetServer should be available")
        //     .load("textures/raindrops.tga");

        // let mut raindrops_texture = app
        //     .world
        //     .get_resource_mut::<Assets<RaindropsTexture>>()
        //     .expect("Should be available");

        // let asset_server = app
        //     .world
        //     .get_resource::<AssetServer>()
        //     .expect("AssetServer should be available");

        // raindrops_texture.add(RaindropsTexture {
        //     color_texture: Some(texture_handle),
        // });
        // asset_server.load(path);

        // assets.

        app.insert_resource(RaindropsTextureHandle(texture_handle))
            .add_plugin(PostProcessingPlugin::<Raindrops>::new(
                "Raindrops",
                shader_handle,
            ));
    }
}

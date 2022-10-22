//! Credits to Ben Cloward, see [the YouTube video](https://www.youtube.com/watch?v=Ftpf87brKWg).

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{
            AddressMode, AsBindGroup, SamplerDescriptor, ShaderRef, ShaderType, TextureFormat,
            TextureViewDescriptor, TextureViewDimension,
        },
        texture::ImageSampler,
    },
    sprite::{Material2d, Material2dPlugin},
};

use crate::{new_effect_state, setup_effect, EffectState, HasEffectState};

const RAINDROPS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5536809487310763617);

/// This plugin allows adding raindrops to an image.
pub struct RaindropsPlugin;

/// Blur parameters.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct Raindrops {
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
            time_scaling: 0.8,
            intensity: 0.03,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "3812649b-8a23-420a-bf03-a87ab11b7c78"]
struct RaindropsMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    raindrops_image: Option<Handle<Image>>,

    #[uniform(4)]
    raindrops: Raindrops,

    state: EffectState,
}

impl HasEffectState for RaindropsMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for RaindropsMaterial {
    fn fragment_shader() -> ShaderRef {
        if cfg!(feature = "dev") {
            "shaders/raindrops.wgsl".into()
        } else {
            RAINDROPS_SHADER_HANDLE.typed().into()
        }
    }
}

impl FromWorld for RaindropsMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);

        let raindrops = world
            .get_resource::<Raindrops>()
            .expect("Raindrops resource");
        let raindrops_image = world
            .get_resource::<RaindropsImage>()
            .expect("Raindrops Image resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            raindrops_image: Some(raindrops_image.0.clone_weak()),
            raindrops: *raindrops,
            state,
        }
    }
}

/// Stores the handle to the texture having the raindrops.
/// We need this because we need to do fixups after loading this texture.
/// Specifically, it needs a different sampler address mode.
/// Having the handle stored allows us to see if this is the one that has
/// been loaded when doing fixups.
#[derive(Debug, Resource)]
struct RaindropsImage(Handle<Image>);

impl FromWorld for RaindropsImage {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world
            .get_resource::<AssetServer>()
            .expect("Should have AssetServer");

        Self(asset_server.load("textures/raindrops.tga"))
    }
}

fn update_raindrops(
    mut raindrop_materials: ResMut<Assets<RaindropsMaterial>>,
    raindrops: Res<Raindrops>,
) {
    if !raindrops.is_changed() {
        return;
    }

    for (_, material) in raindrop_materials.iter_mut() {
        material.raindrops = *raindrops;
    }
}

// Raindrops texture needs to use repeat address mode.
fn fixup_texture(
    mut done: Local<bool>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    raindrops_texture: ResMut<RaindropsImage>,
    mut raindrop_materials: ResMut<Assets<RaindropsMaterial>>,
) {
    if *done {
        return;
    }

    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            if *handle == raindrops_texture.0 {
                *done = true;

                let image = assets
                    .get_mut(handle)
                    .expect("Handle should point to asset");

                image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    label: Some("Raindrops Sampler"),
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    ..default()
                });

                let format = TextureFormat::Rgba8Unorm;
                image.texture_descriptor.format = format;

                image.texture_view_descriptor = Some(TextureViewDescriptor {
                    label: Some("Raindrops TextureViewDescriptor"),
                    format: Some(format),
                    dimension: Some(TextureViewDimension::D2),
                    ..default()
                });

                for (_, material) in raindrop_materials.iter_mut() {
                    material.raindrops_image = Some(handle.clone());
                }
            }
        }
    }
}

impl Plugin for RaindropsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("RaindropsPlugin build").entered();

        if !cfg!(feature = "dev") {
            use bevy::asset::load_internal_asset;
            load_internal_asset!(
                app,
                RAINDROPS_SHADER_HANDLE,
                "../../assets/shaders/raindrops.wgsl",
                Shader::from_wgsl
            );
        }

        app.init_resource::<Raindrops>()
            .init_resource::<RaindropsImage>()
            .init_resource::<RaindropsMaterial>()
            .add_plugin(Material2dPlugin::<RaindropsMaterial>::default())
            .add_startup_system(setup_effect::<RaindropsMaterial>)
            .add_system(fixup_texture)
            .add_system(update_raindrops);
    }
}

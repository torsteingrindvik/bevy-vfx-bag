//! Credits to Ben Cloward, see [the YouTube video](https://www.youtube.com/watch?v=Ftpf87brKWg).

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AddressMode, AsBindGroup, RenderPipelineDescriptor, SamplerDescriptor, ShaderRef,
            ShaderType, SpecializedMeshPipelineError, TextureFormat, TextureViewDescriptor,
            TextureViewDimension,
        },
        texture::{CompressedImageFormats, ImageSampler},
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use crate::{
    load_asset_if_no_dev_feature, new_effect_state, passthrough, setup_effect, shader_ref,
    EffectState, HasEffectState, Passthrough,
};

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

/// If this effect should not be enabled, i.e. it should just
/// pass through the input image.
#[derive(Debug, Resource, Default, PartialEq, Eq, Hash, Clone)]
pub struct RaindropsPassthrough(pub bool);

impl Passthrough for RaindropsPassthrough {
    fn passthrough(&self) -> bool {
        self.0
    }
}

impl From<&RaindropsMaterial> for RaindropsPassthrough {
    fn from(material: &RaindropsMaterial) -> Self {
        Self(material.passthrough)
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "3812649b-8a23-420a-bf03-a87ab11b7c78"]
#[bind_group_data(RaindropsPassthrough)]
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

    passthrough: bool,
}

impl HasEffectState for RaindropsMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for RaindropsMaterial {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(RAINDROPS_SHADER_HANDLE, "shaders/raindrops.wgsl")
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        passthrough(descriptor, &key);

        Ok(())
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
            passthrough: false,
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
        let image_handle = if cfg!(feature = "dev") {
            let asset_server = world
                .get_resource::<AssetServer>()
                .expect("Should have AssetServer");

            asset_server.load("textures/raindrops.tga")
        } else {
            use bevy::render::texture::ImageType;
            let mut image_assets = world
                .get_resource_mut::<Assets<Image>>()
                .expect("Should have Assets<Image>");

            let image_bytes = include_bytes!("../../assets/textures/raindrops.tga");
            let image = Image::from_buffer(
                image_bytes,
                ImageType::Extension("tga"),
                CompressedImageFormats::NONE,
                true,
            )
            .expect("Raindrops tga should load properly");

            image_assets.add(image)
        };

        Self(image_handle)
    }
}

fn update_raindrops(
    mut raindrop_materials: ResMut<Assets<RaindropsMaterial>>,
    raindrops: Res<Raindrops>,
    passthrough: Res<RaindropsPassthrough>,
) {
    if !raindrops.is_changed() && !passthrough.is_changed() {
        return;
    }

    for (_, material) in raindrop_materials.iter_mut() {
        material.raindrops = *raindrops;
        material.passthrough = passthrough.0;
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

        load_asset_if_no_dev_feature!(
            app,
            RAINDROPS_SHADER_HANDLE,
            "../../assets/shaders/raindrops.wgsl"
        );

        app.init_resource::<Raindrops>()
            .init_resource::<RaindropsImage>()
            .init_resource::<RaindropsMaterial>()
            .init_resource::<RaindropsPassthrough>()
            .add_plugin(Material2dPlugin::<RaindropsMaterial>::default())
            .add_startup_system(setup_effect::<RaindropsMaterial>)
            .add_system(fixup_texture)
            .add_system(update_raindrops);
    }
}

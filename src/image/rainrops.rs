//! All credits to Ben Cloward for this effect.
//! See [this video](https://www.youtube.com/watch?v=Ftpf87brKWg).

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
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{BevyVfxBagImage, BevyVfxBagRenderLayer, ShouldResize};

/// This plugin allows adding raindrops to an image.
pub struct RaindropsPlugin;

/// Blur parameters.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct Raindrops {
    /// Todo
    pub hmm: f32,
}

impl Default for Raindrops {
    fn default() -> Self {
        Self { hmm: 1.0 }
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
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
}

impl Material2d for RaindropsMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/raindrops.wgsl".into()
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut raindrop_materials: ResMut<Assets<RaindropsMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    raindrops: Res<Raindrops>,
    images: ResMut<Assets<Image>>,
) {
    let image = images
        .get(&*image_handle)
        .expect("BevyVfxBagImage should exist");

    let extent = image.texture_descriptor.size;

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        extent.width as f32,
        extent.height as f32,
    ))));

    let material_handle = raindrop_materials.add(RaindropsMaterial {
        source_image: image_handle.clone(),
        raindrops_image: None,
        raindrops: *raindrops,
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        render_layer.0,
        ShouldResize,
    ));

    debug!("OK");
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

        app.init_resource::<Raindrops>()
            .init_resource::<RaindropsImage>()
            .add_plugin(Material2dPlugin::<RaindropsMaterial>::default())
            .add_startup_system(setup)
            .add_system(fixup_texture)
            .add_system(update_raindrops);
    }
}

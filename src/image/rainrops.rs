//! All credits to Ben Cloward for this effect.
//! See [this video](https://www.youtube.com/watch?v=Ftpf87brKWg).

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
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
    raindrops_image: Handle<Image>,

    #[uniform(4)]
    raindrops: Raindrops,
}

impl Material2d for RaindropsMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/raindrops.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut raindrop_materials: ResMut<Assets<RaindropsMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    raindrops: Res<Raindrops>,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let image = images
        .get(&*image_handle)
        .expect("BevyVfxBagImage should exist");

    let extent = image.texture_descriptor.size;

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        extent.width as f32,
        extent.height as f32,
    ))));

    let raindrops_image_handle = asset_server.load("textures/raindrops.tga");

    let material_handle = raindrop_materials.add(RaindropsMaterial {
        source_image: image_handle.clone(),
        raindrops_image: raindrops_image_handle,
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

impl Plugin for RaindropsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("RaindropsPlugin build").entered();

        app.init_resource::<Raindrops>()
            .add_plugin(Material2dPlugin::<RaindropsMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_raindrops);
    }
}

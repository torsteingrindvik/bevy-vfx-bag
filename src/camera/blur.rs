use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{BevyVfxBagImage, BevyVfxBagRenderLayer};

/// This plugin allows blurring the scene.
/// Add this plugin to the [`App`] in order to use it.
pub struct BlurPlugin;

/// How much to blur.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct Blur {
    /// How blurry.
    /// TODO: Range docs
    pub amount: f32,

    /// How far away the blur should sample points away from the origin point
    /// when blurring.
    /// This is in UV coordinates, so small (positive) values are expected.
    pub kernel_radius: f32,
}

impl Default for Blur {
    fn default() -> Self {
        Self {
            amount: Default::default(),
            kernel_radius: 0.01,
        }
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "70bc3d3b-46e2-40ea-bedc-e0d73ffdd3fd"]
struct BlurMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    blur: Blur,
}

impl Material2d for BlurMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blur.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut blur_materials: ResMut<Assets<BlurMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    blur: Res<Blur>,
    images: Res<Assets<Image>>,
) {
    let image = images
        .get(&*image_handle)
        .expect("BevyVfxBagImage should exist");

    let extent = image.texture_descriptor.size;

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        extent.width as f32,
        extent.height as f32,
    ))));

    let material_handle = blur_materials.add(BlurMaterial {
        source_image: image_handle.clone(),
        blur: *blur,
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
    ));

    debug!("OK");
}

fn update_blur(mut blur_materials: ResMut<Assets<BlurMaterial>>, blur: Res<Blur>) {
    if !blur.is_changed() {
        return;
    }

    for (_, material) in blur_materials.iter_mut() {
        material.blur = *blur;
    }
}

impl Plugin for BlurPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("BlurPlugin build").entered();

        app.init_resource::<Blur>()
            .add_plugin(Material2dPlugin::<BlurMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_blur);
    }
}

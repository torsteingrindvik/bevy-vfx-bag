use std::f32::consts::PI;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{BevyVfxBagImage, BevyVfxBagRenderLayer, ShouldResize};

/// This plugin allows using chromatic aberration.
/// This offsets the RGB channels with some magnitude
/// and direction seperately.
pub struct ChromaticAberrationPlugin;

/// Chromatic aberration parameters.
#[derive(Debug, Copy, Clone, Resource, ShaderType)]
pub struct ChromaticAberration {
    /// The direction (in UV space) the red channel is offset in.
    /// Will be normalized.
    pub dir_r: Vec2,

    /// How far (in UV space) the red channel should be displaced.
    pub magnitude_r: f32,

    /// The direction (in UV space) the green channel is offset in.
    /// Will be normalized.
    pub dir_g: Vec2,

    /// How far (in UV space) the green channel should be displaced.
    pub magnitude_g: f32,

    /// The direction (in UV space) the blue channel is offset in.
    /// Will be normalized.
    pub dir_b: Vec2,

    /// How far (in UV space) the blue channel should be displaced.
    pub magnitude_b: f32,
}

impl Default for ChromaticAberration {
    fn default() -> Self {
        let one_third = (2. / 3.) * PI;

        Self {
            dir_r: Vec2::from_angle(0. * one_third),
            magnitude_r: 0.01,
            dir_g: Vec2::from_angle(1. * one_third),
            magnitude_g: 0.01,
            dir_b: Vec2::from_angle(2. * one_third),
            magnitude_b: 0.01,
        }
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "1c857de0-74e6-42a4-a1b4-0e0f1564a880"]
struct ChromaticAberrationMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    chromatic_aberration: ChromaticAberration,
}

impl Material2d for ChromaticAberrationMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chromatic-aberration.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wave_materials: ResMut<Assets<ChromaticAberrationMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    chromatic_aberration: Res<ChromaticAberration>,
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

    let material_handle = wave_materials.add(ChromaticAberrationMaterial {
        source_image: image_handle.clone(),
        chromatic_aberration: *chromatic_aberration,
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

// TODO: Normalize
fn update_chromatic_aberration(
    mut chromatic_aberration_materials: ResMut<Assets<ChromaticAberrationMaterial>>,
    chromatic_aberration: Res<ChromaticAberration>,
) {
    if !chromatic_aberration.is_changed() {
        return;
    }

    for (_, material) in chromatic_aberration_materials.iter_mut() {
        material.chromatic_aberration = *chromatic_aberration;
    }
}

impl Plugin for ChromaticAberrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("ChromaticAberrationPlugin build").entered();

        app.init_resource::<ChromaticAberration>()
            .add_plugin(Material2dPlugin::<ChromaticAberrationMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_chromatic_aberration);
    }
}

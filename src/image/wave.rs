use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{BevyVfxBagImage, BevyVfxBagRenderLayer, ShouldResize};

/// This plugin allows creating a wave across the image.
/// A wave can be customized in the X and Y axes for interesting effects.
pub struct WavePlugin;

/// Wave parameters.
///
/// Note that the parameters for the X axis causes a wave effect
/// towards the left- and right sides of the screen.
/// For example, if we have 1 wave in the X axis,
/// we will have one part of the screen stretched towards the right
/// horizontally, and one part stretched towards the left.
#[derive(Default, Debug, Copy, Clone, Resource, ShaderType)]
pub struct Wave {
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

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "79fa38f9-ca04-4e59-83f9-da0de45afc04"]
struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    wave: Wave,
}

impl Material2d for WaveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wave.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wave_materials: ResMut<Assets<WaveMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    wave: Res<Wave>,
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

    let material_handle = wave_materials.add(WaveMaterial {
        source_image: image_handle.clone(),
        wave: *wave,
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

fn update_wave(mut wave_materials: ResMut<Assets<WaveMaterial>>, wave: Res<Wave>) {
    if !wave.is_changed() {
        return;
    }

    for (_, material) in wave_materials.iter_mut() {
        material.wave = *wave;
    }
}

impl Plugin for WavePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("WavePlugin build").entered();

        app.init_resource::<Wave>()
            .add_plugin(Material2dPlugin::<WaveMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_wave);
    }
}

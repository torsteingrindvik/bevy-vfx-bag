use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{BevyVfxBagImage, BevyVfxBagRenderLayer};

/// This plugin allows adding a vignette effect to a given camera.
/// Add this plugin to the [`App`] in order to use it.
/// Then, add the [`Vignette`] component to the camera you want the effect to apply to.
pub struct VignettePlugin;

/// This component enables a vignette effect on the camera it is insert onto.
/// Assumes the [`VignettePlugin`] has been added to the [`App`].
#[derive(Debug, Component, Clone)]
pub struct Vignette {
    /// The radius of the effect.
    /// A radius of 1.0 will cover the entire screen (in both axes).
    /// A radius of less than 1.0 will start shrinking the effect towards the center of the screen.
    pub radius: f32,

    /// The distance of the smooth transition between the effect and the scene.
    /// Note that this will add to the radius of the effect.
    pub feathering: f32,

    /// The color of the vignette.
    /// Note that the alpha channel is used by the effect.
    pub color: Color,
}

impl Vignette {
    /// Create a vignette effect with the given parameters.
    pub fn new(radius: f32, feathering: f32, color: Color) -> Self {
        Self {
            radius,
            feathering,
            color,
        }
    }
}

impl Default for Vignette {
    fn default() -> Self {
        let mut color = Color::BLACK;
        color.set_a(0.8);

        Self {
            radius: 1.0,
            feathering: 0.1,
            color,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "9ca04144-a3e1-40b4-93a7-91424159f612"]
struct VignetteMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,
}

impl Material2d for VignetteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vignette.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut vignette_materials: ResMut<Assets<VignetteMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
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

    let material_handle = vignette_materials.add(VignetteMaterial {
        source_image: image_handle.clone(),
    });

    // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
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
}

impl Plugin for VignettePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<VignetteMaterial>::default())
            .add_startup_system(setup);
    }
}

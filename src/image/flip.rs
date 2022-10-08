use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{quad::window_sized_quad, BevyVfxBagImage, BevyVfxBagRenderLayer, ShouldResize};

/// This plugin allows flipping the rendered scene horizontally and/or vertically.
/// Add this plugin to the [`App`] in order to use it.
pub struct FlipPlugin;

/// Which way to flip the texture.
#[derive(Debug, Default, Copy, Clone, Resource)]
pub enum Flip {
    /// Don't flip.
    #[default]
    None,

    /// Flip horizontally.
    Horizontal,

    /// Flip vertically.
    Vertical,

    /// Flip both axes.
    HorizontalVertical,
}

impl From<Flip> for FlipUniform {
    fn from(flip: Flip) -> Self {
        let uv = match flip {
            Flip::None => [0.0, 0.0],
            Flip::Horizontal => [1.0, 0.0],
            Flip::Vertical => [0.0, 1.0],
            Flip::HorizontalVertical => [1.0, 1.0],
        };

        Self { x: uv[0], y: uv[1] }
    }
}

#[derive(Debug, Clone, ShaderType)]
struct FlipUniform {
    x: f32,
    y: f32,
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "70bc3d3b-46e2-40ea-bedc-e0d73ffdd3fd"]
struct FlipMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Option<Handle<Image>>,

    #[uniform(2)]
    flip: FlipUniform,
}

impl Material2d for FlipMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/flip.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flip_materials: ResMut<Assets<FlipMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    flip: Res<Flip>,
    windows: Res<Windows>,
) {
    let flip_material = FlipMaterial {
        source_image: Some(image_handle.clone()),
        flip: (*flip).into(),
    };

    let material_handle = flip_materials.add(flip_material);

    let mesh = window_sized_quad(windows.primary());

    // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
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
}

fn update_flip(mut flip_materials: ResMut<Assets<FlipMaterial>>, flip: Res<Flip>) {
    if !flip.is_changed() {
        return;
    }

    for (_, material) in flip_materials.iter_mut() {
        material.flip = (*flip).into();
    }
}

impl Plugin for FlipPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("FlipPlugin build").entered();

        app.init_resource::<Flip>()
            .add_plugin(Material2dPlugin::<FlipMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_flip);
    }
}

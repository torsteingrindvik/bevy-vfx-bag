#![allow(clippy::too_many_arguments)] // Bevy fns tend to have many args.
#![deny(clippy::unwrap_used)] // Let's try to explain invariants when we unwrap (so use expect).
#![deny(missing_docs)] // Let's try to have good habits.
#![doc = include_str!("../README.md")]

use bevy::prelude::{App, *};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::Mesh2dHandle;
use bevy::window::WindowResized;

use crate::quad::window_sized_quad;

/// Effects which are added to an image.
/// This image might be the output of a render pass of an app.
pub mod image;

/// Helpers for making quads.
pub mod quad;

/// This resource holds the image handle of the image which will be used for
/// sampling before applying effects.
/// Typically, the [`bevy::render::camera::RenderTarget`] of the camera that wants post processing
/// should use this.
#[derive(Debug, Resource)]
pub struct BevyVfxBagImage(Handle<Image>);

#[derive(Debug, Component)]
pub(crate) struct ShouldResize;

fn make_image(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("BevyVfxBag Main Image"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    image.resize(size);
    image
}

impl FromWorld for BevyVfxBagImage {
    fn from_world(world: &mut World) -> Self {
        let (width, height) = {
            let windows = world.resource::<Windows>();
            let primary = windows.get_primary().expect("Should have primary window");

            (primary.physical_width(), primary.physical_height())
        };

        let mut image_assets = world.resource_mut::<Assets<Image>>();

        let image = make_image(width, height);
        let image_handle = image_assets.add(image);

        Self(image_handle)
    }
}

impl std::ops::Deref for BevyVfxBagImage {
    type Target = Handle<Image>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The base plugin which sets up base resources and
/// systems which other plugins in this crate rely on.
pub struct BevyVfxBagPlugin;

/// The render layer post processing effects are rendered to.
/// Defaults to the last possible layer.
#[derive(Debug, Resource)]
pub struct BevyVfxBagRenderLayer(pub RenderLayers);

impl Default for BevyVfxBagRenderLayer {
    fn default() -> Self {
        Self(RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8))
    }
}

/// The priority of the camera post processing effects are rendered to.
/// Defaults to `1`, the one after the default camera.
#[derive(Debug, Resource)]
pub struct BevyVfxBagPriority(isize);

impl Default for BevyVfxBagPriority {
    fn default() -> Self {
        Self(1)
    }
}

fn setup(
    mut commands: Commands,
    priority: Res<BevyVfxBagPriority>,
    render_layer: Res<BevyVfxBagRenderLayer>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                priority: priority.0,
                ..default()
            },
            ..default()
        },
        render_layer.0,
    ));
}

fn update(
    windows: Res<Windows>,
    mut resize_reader: EventReader<WindowResized>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    needs_new_quad: Query<&Mesh2dHandle, With<ShouldResize>>,
) {
    let main_window_id = windows.get_primary().expect("Should have window").id();

    for event in resize_reader.iter() {
        if event.id != main_window_id {
            continue;
        }

        let window = windows.get(event.id).expect("Main window should exist");

        for resize_me in &needs_new_quad {
            let mesh = window_sized_quad(window);

            *mesh_assets.get_mut(&resize_me.0).expect("Should find mesh") = mesh;
        }
    }
}

impl Plugin for BevyVfxBagPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyVfxBagImage>()
            .init_resource::<BevyVfxBagRenderLayer>()
            .init_resource::<BevyVfxBagPriority>()
            .add_startup_system(setup)
            .add_system(update);
    }
}

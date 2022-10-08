#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]

//! This crate allows you to add grapical effects to your Bevy applications.

use bevy::prelude::{App, *};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;

/// Effects which are added to an image.
/// This image might be the output of a render pass of an app.
pub mod image;

/// This resource holds the image handle of the image which will be used for
/// sampling before applying effects.
/// Typically, the [`RenderTarget`] of the camera that wants post processing
/// should use this.
#[derive(Debug, Resource)]
pub struct BevyVfxBagImage(Handle<Image>);

impl FromWorld for BevyVfxBagImage {
    fn from_world(world: &mut World) -> Self {
        let windows = world.resource::<Windows>();

        let window = windows.primary();
        let size = Extent3d {
            width: window.physical_width(),
            height: window.physical_height(),
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

        let mut images = world.resource_mut::<Assets<Image>>();

        Self(images.add(image))
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

impl Plugin for BevyVfxBagPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyVfxBagImage>()
            .init_resource::<BevyVfxBagRenderLayer>()
            .init_resource::<BevyVfxBagPriority>()
            .add_startup_system(setup);
    }
}

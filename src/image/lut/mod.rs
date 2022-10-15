//! 3D LUTs for color grading.
//! See the [GPU Gems 2 article](https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-24-using-lookup-tables-accelerate-color)
//!
//! # How to create your own
//! First run the `make-neutral-lut` example to create a neutral LUT.
//! Then simply open that in an image editor and make modifications using filters, and save it when you're happy.
//!
//! One trick is to take the neutral LUT and place it on top of a screenshot of your game.
//! Then adjust colors of the whole combined image until you're happy.
//! Then extract the LUT from your game screenshot and save it in the exact
//! same size as it was originally.
//! You can now use it to transform your game look into what you had in your image editor.
//!
//! Note that the LUT assets in this crate were created by simply loading the neutral LUT
//! into the Windows Photos application, and applying the filters there (hence the names).

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
            TextureViewDescriptor, TextureViewDimension,
        },
        texture::{CompressedImageFormats, ImageType},
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::{BevyVfxBagImage, BevyVfxBagRenderLayer, ShouldResize};

/// This plugin allows using look-up textures for color grading.
pub struct LutPlugin;

/// LUT parameters.
#[derive(Debug, Clone, Resource)]
pub struct Lut {
    /// The look-up texture.
    texture: Lut3d,

    /// If the look-up texture should appear on top of the
    /// output image.
    pub show_overlay: bool,
}

impl Lut {
    /// Set the LUT.
    pub fn set_texture(&mut self, texture: &Lut3d) {
        self.texture = texture.clone();
    }
}

impl FromWorld for Lut {
    fn from_world(world: &mut World) -> Self {
        let neutral = world
            .get_resource::<LutNeutral>()
            .expect("LutPlugin should init LutNeutral");
        Self {
            texture: neutral.0.clone(),
            show_overlay: true,
        }
    }
}

/// The LUT needs to be 3D.
/// This requires loading it in a special way.
#[derive(Debug, Clone, Resource)]
pub struct Lut3d(Handle<Image>);

const LUT3D_SIZE: Extent3d = Extent3d {
    width: 64,
    height: 64,
    depth_or_array_layers: 64,
};

impl Lut3d {
    /// Create a new 3D LUT.
    ///
    /// It's assumed the output file from running
    /// `cargo r --example make-neutral-lut` has been used.
    /// That has the correct size and its layout is correct for being used as
    /// a 3D texture.
    ///
    /// 3D textures of other sizes and formats might be supported if there is
    /// a need for it.
    pub fn new(images: &mut Assets<Image>, texture_data: &[u8]) -> Self {
        Self::new_sized(images, texture_data, LUT3D_SIZE)
    }

    /// Create a new 3D LUT.
    ///
    /// This has the same assumptions as [`Lut3d::new`].
    pub fn from_image(images: &mut Assets<Image>, image_handle: &Handle<Image>) -> Self {
        let image = images
            .get(image_handle)
            .expect("Handle should refer to image")
            .clone();

        Self::new_from_image(images, image)
    }

    fn new_from_image_sized(images: &mut Assets<Image>, mut image: Image, size: Extent3d) -> Self {
        image.texture_descriptor.dimension = TextureDimension::D3;
        image.texture_descriptor.size = size;
        image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

        image.texture_view_descriptor = Some(TextureViewDescriptor {
            label: Some("LUT TextureViewDescriptor"),
            format: Some(image.texture_descriptor.format),
            dimension: Some(TextureViewDimension::D3),
            ..default()
        });

        let handle = images.add(image);

        Self(handle)
    }

    fn new_from_image(images: &mut Assets<Image>, image: Image) -> Self {
        Self::new_from_image_sized(images, image, LUT3D_SIZE)
    }

    fn new_sized(images: &mut Assets<Image>, texture_data: &[u8], size: Extent3d) -> Self {
        let image = Image::from_buffer(
            texture_data,
            ImageType::Extension("png"), // todo
            CompressedImageFormats::NONE,
            // If `true` the output the mapping is very dark.
            // If not, it's much closer to the original.
            false,
        )
        .expect("Should be able to load image from buffer");

        Self::new_from_image_sized(images, image, size)
    }
}

/// The neutral LUT.
#[derive(Debug, Clone, Resource)]
pub struct LutNeutral(Lut3d);

impl FromWorld for LutNeutral {
    fn from_world(world: &mut World) -> Self {
        let mut images = world
            .get_resource_mut::<Assets<Image>>()
            .expect("Assets<Image> should exist");

        let data = include_bytes!("neutral.png");

        Self(Lut3d::new(&mut *images, data))
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "abb36dfa-9b2c-4150-8a50-f85c594c797e"]
struct LutMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[texture(2, dimension = "3d")]
    #[sampler(3)]
    lut: Handle<Image>,
}

impl Material2d for LutMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lut.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lut_materials: ResMut<Assets<LutMaterial>>,
    image_handle: Res<BevyVfxBagImage>,
    render_layer: Res<BevyVfxBagRenderLayer>,
    lut: Res<Lut>,
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

    let material_handle = lut_materials.add(LutMaterial {
        source_image: image_handle.clone(),
        lut: lut.texture.0.clone(),
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

fn update_lut(mut lut_materials: ResMut<Assets<LutMaterial>>, lut: Res<Lut>) {
    if !lut.is_changed() {
        return;
    }

    for (_, material) in lut_materials.iter_mut() {
        material.lut = lut.texture.0.clone()
    }
}

impl Plugin for LutPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("LUT build").entered();

        app.add_plugin(Material2dPlugin::<LutMaterial>::default())
            // Initialize the fallback neutral LUT in case the user
            // has not initialized [`Lut`]
            .init_resource::<LutNeutral>()
            // Now initialize [`Lut`], which will then use [`LutNeutral`] if it must.
            .init_resource::<Lut>()
            .add_startup_system(setup)
            .add_system(update_lut);
    }
}

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
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, TextureDimension, TextureFormat, TextureViewDescriptor,
            TextureViewDimension,
        },
        texture::{CompressedImageFormats, ImageSampler, ImageType},
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
    utils::HashMap,
};

use crate::{
    load_asset_if_no_dev_feature, new_effect_state, passthrough, setup_effect, EffectState,
    HasEffectState, Passthrough,
};

const LUT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 182690283339630534);

/// This plugin allows using look-up textures for color grading.
pub struct LutPlugin;

/// LUT parameters.
#[derive(Debug, Clone, Resource)]
pub struct Lut {
    /// The look-up texture.
    texture: Lut3d,

    /// If we should show the original image on one half of the screen,
    /// and the LUT color graded output on the other half.
    /// If `false`, only the color graded output is shown.
    pub split_vertically: bool,
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
            split_vertically: false,
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

        image.sampler_descriptor = ImageSampler::linear();

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

impl<'a> From<&'a Lut> for Option<&'a Handle<Image>> {
    fn from(lut: &'a Lut) -> Self {
        Some(&lut.texture.0)
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, Resource)]
#[uuid = "abb36dfa-9b2c-4150-8a50-f85c594c797e"]
#[bind_group_data(LutMaterialKey)]
struct LutMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[texture(2, dimension = "3d")]
    #[sampler(3)]
    lut: Lut,

    state: EffectState,

    passthrough: bool,
}

impl HasEffectState for LutMaterial {
    fn state(&self) -> crate::EffectState {
        self.state.clone()
    }
}

impl Material2d for LutMaterial {
    fn fragment_shader() -> ShaderRef {
        if cfg!(feature = "dev") {
            "shaders/lut.wgsl".into()
        } else {
            LUT_SHADER_HANDLE.typed().into()
        }
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        passthrough(descriptor, &key);

        let mut push_def = |def: &str| {
            descriptor
                .fragment
                .as_mut()
                .expect("Should have fragment state")
                .shader_defs
                .push(def.into())
        };

        if key.bind_group_data.split_vertically {
            push_def("SPLIT_VERTICALLY");
        }

        Ok(())
    }
}

impl FromWorld for LutMaterial {
    fn from_world(world: &mut World) -> Self {
        let state = new_effect_state(world);
        let lut = world.get_resource::<Lut>().expect("Lut resource");

        Self {
            source_image: state.input_image_handle.clone_weak(),
            state,
            lut: lut.clone(),
            passthrough: false,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct LutMaterialKey {
    split_vertically: bool,
    passthrough: bool,
}

impl Passthrough for LutMaterialKey {
    fn passthrough(&self) -> bool {
        self.passthrough
    }
}

/// If this effect should not be enabled, i.e. it should just
/// pass through the input image.
#[derive(Debug, Resource, Default, PartialEq, Eq, Hash, Clone)]
pub struct LutPassthrough(pub bool);

impl Passthrough for LutPassthrough {
    fn passthrough(&self) -> bool {
        self.0
    }
}

impl From<&LutMaterial> for LutMaterialKey {
    fn from(lut_material: &LutMaterial) -> Self {
        Self {
            split_vertically: lut_material.lut.split_vertically,
            passthrough: lut_material.passthrough,
        }
    }
}

fn update_lut(
    mut lut_materials: ResMut<Assets<LutMaterial>>,
    lut: Res<Lut>,
    passthrough: Res<LutPassthrough>,
) {
    if !lut.is_changed() && !passthrough.is_changed() {
        return;
    }

    for (_, material) in lut_materials.iter_mut() {
        material.lut = lut.clone();
        material.passthrough = passthrough.0;
    }
}

/// LUTs available for use.
///
/// They can be used by calling `.set_texture` on [`Lut`].
///
/// By loading a new image and inserting it into the handles in this resource,
/// the image will automatically be transformed into a [`Lut3d`] when loaded
/// and moved into the `ready` field.
///
/// The LUTs shipped with the plugin itself (applied using `.set_texture`) are:
///     - "Neutral"
///     - "Arctic"
///     - "Burlesque"
///     - "Denim"
///     - "Neo"
///     - "Rouge"
///     - "Sauna"
///     - "Slate"
#[derive(Debug, Resource)]
pub struct Luts {
    /// Handles to images which have not yet been loaded
    /// and moved into `ready`.
    pub handles: HashMap<Handle<Image>, &'static str>,

    /// [`Lut3d`]s which are ready for use.
    /// See [`Lut::set_texture`].
    pub ready: HashMap<&'static str, Lut3d>,
}

impl FromWorld for Luts {
    #[cfg(feature = "dev")]
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            handles: HashMap::from_iter(vec![
                (assets.load("luts/neutral.png"), "Neutral"),
                (assets.load("luts/arctic.png"), "Arctic"),
                (assets.load("luts/burlesque.png"), "Burlesque"),
                (assets.load("luts/denim.png"), "Denim"),
                (assets.load("luts/neo.png"), "Neo"),
                (assets.load("luts/rouge.png"), "Rouge"),
                (assets.load("luts/sauna.png"), "Sauna"),
                (assets.load("luts/slate.png"), "Slate"),
            ]),
            ready: HashMap::new(),
        }
    }

    #[cfg(not(feature = "dev"))]
    fn from_world(world: &mut World) -> Self {
        let mut assets = world.resource_mut::<Assets<Image>>();

        macro_rules! load {
            ($assets: ident, $name: literal, $image_path: literal) => {
                (
                    $assets.add(
                        Image::from_buffer(
                            include_bytes!($image_path),
                            ImageType::Extension("png"), // todo
                            CompressedImageFormats::NONE,
                            // If `true` the output the mapping is very dark.
                            // If not, it's much closer to the original.
                            false,
                        )
                        .expect("Should be able to load image from buffer"),
                    ),
                    $name,
                )
            };
        }

        Self {
            handles: HashMap::from_iter(vec![
                load!(assets, "Neutral", "../../../assets/luts/neutral.png"),
                load!(assets, "Arctic", "../../../assets/luts/arctic.png"),
                load!(assets, "Burlesque", "../../../assets/luts/burlesque.png"),
                load!(assets, "Denim", "../../../assets/luts/denim.png"),
                load!(assets, "Neo", "../../../assets/luts/neo.png"),
                load!(assets, "Rouge", "../../../assets/luts/rouge.png"),
                load!(assets, "Sauna", "../../../assets/luts/sauna.png"),
                load!(assets, "Slate", "../../../assets/luts/slate.png"),
            ]),
            ready: HashMap::new(),
        }
    }
}

fn add_created_3d_luts(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    mut luts: ResMut<Luts>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            if let Some(lut_name) = luts.handles.remove(handle) {
                luts.ready
                    .insert(lut_name, Lut3d::from_image(&mut assets, handle));
            }
        }
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

        let data = include_bytes!("../../../assets/luts/neutral.png");

        Self(Lut3d::new(&mut images, data))
    }
}

impl Plugin for LutPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let _span = debug_span!("LUT build").entered();

        load_asset_if_no_dev_feature!(app, LUT_SHADER_HANDLE, "../../../assets/shaders/lut.wgsl");

        app
            // Custom LUTs
            .init_resource::<Luts>()
            // The "no-op" LUT
            .init_resource::<LutNeutral>()
            // The user config
            .init_resource::<Lut>()
            .init_resource::<LutMaterial>()
            .init_resource::<LutPassthrough>()
            .add_plugin(Material2dPlugin::<LutMaterial>::default())
            .add_startup_system(setup_effect::<LutMaterial>)
            .add_system(add_created_3d_luts)
            .add_system(update_lut);
    }
}

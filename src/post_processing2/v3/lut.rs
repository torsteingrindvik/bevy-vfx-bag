use crate::{load_image, load_lut, shader_ref};
use bevy::{
    asset::LoadState,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
            TextureViewDescriptor, TextureViewDimension,
        },
        texture::ImageSampler,
    },
    sprite::Material2d,
    utils::HashMap,
};

use super::post_processing_plugin;

#[derive(Debug, Default)]
struct IsFixed(bool);

#[derive(Resource, Deref, DerefMut)]
struct Handles(HashMap<LutVariant, (Handle<Image>, IsFixed)>);

pub(crate) const LUT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10304902298789658536);

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let handles = Handles(HashMap::from_iter(vec![
            (LutVariant::Arctic, load_lut!(app, "luts/arctic.png", "png")),
            (
                LutVariant::Burlesque,
                load_lut!(app, "luts/burlesque.png", "png"),
            ),
            (LutVariant::Denim, load_lut!(app, "luts/denim.png", "png")),
            (LutVariant::Neo, load_lut!(app, "luts/neo.png", "png")),
            (
                LutVariant::Neutral,
                load_lut!(app, "luts/neutral.png", "png"),
            ),
            (LutVariant::Rouge, load_lut!(app, "luts/rouge.png", "png")),
            (LutVariant::Sauna, load_lut!(app, "luts/sauna.png", "png")),
            (LutVariant::Slate, load_lut!(app, "luts/slate.png", "png")),
        ]));

        app.insert_resource(handles)
            .add_system(fix_material)
            .add_system(add_material)
            .add_plugin(post_processing_plugin::Plugin::<Lut, LutSettings>::default());
    }
}

fn fix_material(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    mut handles: ResMut<Handles>,
    mut materials: ResMut<Assets<Lut>>,
    // handle: &Handle<Image>,
    // assets: &mut Assets<Image>,
    // materials: &mut Assets<Lut>,
    // variant: &LutVariant,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            info!("Handle to asset created: {:?}", handle);
            if let Some((variant, (handle, _))) = handles
                .iter()
                .find(|(_, (lut_handle, _))| lut_handle == handle)
            {
                let image = assets
                    .get_mut(handle)
                    .expect("Handle should point to asset");

                // The LUT is a 3d texture. It has 64 layers, each of which is a 64x64 image.
                image.texture_descriptor.size = Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 64,
                };
                image.texture_descriptor.dimension = TextureDimension::D3;
                image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

                image.texture_view_descriptor = Some(TextureViewDescriptor {
                    label: Some("LUT TextureViewDescriptor"),
                    format: Some(image.texture_descriptor.format),
                    dimension: Some(TextureViewDimension::D3),
                    ..default()
                });

                // The default sampler may change depending on the image plugin setup,
                // so be explicit here.
                image.sampler_descriptor = ImageSampler::linear();

                // To avoid borrowing issues, we need to clone the handle and variant.
                // let handle = handle.clone();
                let variant = *variant;

                handles
                    .get_mut(&variant)
                    .expect("LUT variant should exist")
                    .1 = IsFixed(true);

                for (_, _material) in materials.iter_mut() {
                    // This mutable "access" is needed to trigger the usage of the new sampler.
                    // TODO: I don't think we need this?
                    // Since materials actually don't exist until added in `add_material`?
                    debug!("Material is pointing to: {:?}", _material.lut);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Lut>>,
    asset_server: Res<AssetServer>,
    handles: Res<Handles>,
    cameras: Query<(Entity, &LutSettings), (With<Camera>, Without<Handle<Lut>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let (handle, is_fixed) = &handles
            .get(&settings.variant)
            .expect("Should not be able to provide invalid variant");

        if !is_fixed.0 {
            continue;
        }

        let state = asset_server.get_load_state(handle);

        info!("LUT state: {:?}", state);

        if !matches!(state, LoadState::Loaded) {
            continue;
        }

        let material_handle = assets.add(Lut {
            lut: handle.clone(),
        });
        commands.entity(entity).insert(material_handle);
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "de05b53e-7a14-11ed-89c7-e315bbf02c20"]
pub struct Lut {
    // TODO, this works now, no need for high indices
    #[texture(7, dimension = "3d")]
    #[sampler(8)]
    pub(crate) lut: Handle<Image>,
}

impl Material2d for Lut {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(LUT_SHADER_HANDLE, "shaders/lut3.wgsl")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LutVariant {
    Arctic,
    Burlesque,
    Denim,
    Neo,
    Neutral,
    Rouge,
    Sauna,
    Slate,
}

/// TODO
#[derive(Debug, Clone, Component)]
pub struct LutSettings {
    pub(crate) variant: LutVariant,
}

impl Default for LutSettings {
    fn default() -> Self {
        Self::new_arctic()
    }
}

impl LutSettings {
    pub(crate) fn new_variant(variant: LutVariant) -> Self {
        Self { variant }
    }

    /// TODO
    pub fn new_arctic() -> Self {
        Self::new_variant(LutVariant::Arctic)
    }

    /// TODO
    pub fn new_neutral() -> Self {
        Self::new_variant(LutVariant::Neutral)
    }
}

impl ExtractComponent for LutSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
}

use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AddressMode, AsBindGroup, SamplerDescriptor, ShaderRef, ShaderType},
        texture::ImageSampler,
    },
    sprite::Material2d,
};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let texture = RaindropsTexture(load_image!(app, "textures/raindrops.tga", "tga", true));

        app.insert_resource(texture)
            .add_system(fix_material)
            .add_system(add_material)
            .add_plugin(
                post_processing_plugin::Plugin::<Raindrops, RaindropsSettings>::default(),
            );
    }
}

#[derive(Resource, Deref, DerefMut)]
struct RaindropsTexture(Handle<Image>);

use crate::{load_image, shader_ref};

use super::post_processing_plugin;

pub(crate) const RAINDROPS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10304902298789658536);

fn fix_material(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    // TODO: Probably why --release fails, we have to "tag" it after fixup
    mut raindrops_handle: ResMut<RaindropsTexture>,
    mut materials: ResMut<Assets<Raindrops>>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            info!("Handle to asset created: {:?}", handle);

            if *handle == **raindrops_handle {
                // fixup_raindrops(handle, &mut assets, &mut raindrop_materials);
                let image = assets
                    .get_mut(handle)
                    .expect("Handle should point to asset");

                image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    label: Some("Repeat Sampler"),
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    ..default()
                });

                for (_, _material) in materials.iter_mut() {
                    // This mutable "access" is needed to trigger the usage of the new sampler.
                    info!("Material is pointing to: {:?}", _material.color_texture);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Raindrops>>,

    // TODO: Could we change this to be a component, then add an "Unfixed" marker?
    // Then remove that after fixing, and this system generally uses Without<Unfixed>?
    handle: Res<RaindropsTexture>,

    cameras: Query<(Entity, &RaindropsSettings), (With<Camera>, Without<Handle<Raindrops>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Raindrops {
            color_texture: handle.0.clone(),
            raindrops: RaindropsUniform {
                time_scaling: settings.time_scaling,
                intensity: settings.intensity,
                zoom: settings.zoom,
            },
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
pub(crate) struct RaindropsUniform {
    pub(crate) time_scaling: f32,
    pub(crate) intensity: f32,
    pub(crate) zoom: f32,
}

impl Default for RaindropsUniform {
    fn default() -> Self {
        Self {
            time_scaling: 0.8,
            intensity: 0.03,
            zoom: 1.0,
        }
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "4fba30ae-73e6-11ed-8575-9b008b9044f0"]
pub struct Raindrops {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) color_texture: Handle<Image>,

    #[uniform(2)]
    pub(crate) raindrops: RaindropsUniform,
}

impl Material2d for Raindrops {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(RAINDROPS_SHADER_HANDLE, "shaders/raindrops3.wgsl")
    }
}

/// TODO
#[derive(Debug, Component, Clone, Copy)]
pub struct RaindropsSettings {
    pub(crate) time_scaling: f32,
    pub(crate) intensity: f32,
    pub(crate) zoom: f32,
}

impl Default for RaindropsSettings {
    fn default() -> Self {
        Self {
            time_scaling: 0.8,
            intensity: 0.03,
            zoom: 1.0,
        }
    }
}

impl ExtractComponent for RaindropsSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

use std::fmt::Display;

use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::RenderAssets,
    render_phase::AddRenderCommand,
    render_resource::{
        AddressMode, BindingResource, Sampler, SamplerBindingType, SamplerDescriptor,
        TextureSampleType, TextureViewDimension,
    },
    RenderSet,
};
pub(crate) use bevy::{
    asset::load_internal_asset,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
            BindingType, BufferBindingType, CachedRenderPipelineId, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
};

use crate::post_processing::{DrawPostProcessingEffect, UniformBindGroup};

use super::{Order, PostProcessingPhaseItem};

pub(crate) const RAINDROPS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3481202994982538867);

#[derive(Resource, ExtractResource, Deref, DerefMut, Clone)]
struct RaindropsTextureHandle(Handle<Image>);

#[derive(Resource)]
pub(crate) struct RaindropsData {
    pub pipeline_id: CachedRenderPipelineId,
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
}

impl FromWorld for RaindropsData {
    fn from_world(world: &mut World) -> Self {
        let (raindrops_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Raindrops",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(Raindrops::min_size()),
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
            ],
            RAINDROPS_SHADER_HANDLE.typed(),
        );

        let raindrops_sampler = world
            .get_resource::<RenderDevice>()
            .expect("Should have render device")
            .create_sampler(&SamplerDescriptor {
                label: Some("Raindrops Sampler"),
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..default()
            });

        RaindropsData {
            pipeline_id,
            layout: raindrops_layout,
            sampler: raindrops_sampler,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            RAINDROPS_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/",
                "raindrops.wgsl"
            ),
            Shader::from_wgsl
        );

        let asset_server = app.world.resource::<AssetServer>();
        let texture_handle: Handle<Image> = asset_server.load("textures/raindrops.tga");

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<Raindrops>::default())
            .add_plugin(UniformComponentPlugin::<Raindrops>::default())
            .add_plugin(ExtractResourcePlugin::<RaindropsTextureHandle>::default())
            .insert_resource(RaindropsTextureHandle(texture_handle));

        super::render_app(app)
            .add_system(
                super::extract_post_processing_camera_phases::<Raindrops>
                    .in_schedule(ExtractSchedule),
            )
            .init_resource::<RaindropsData>()
            .init_resource::<UniformBindGroup<Raindrops>>()
            .add_system(prepare.in_set(RenderSet::Prepare))
            .add_system(queue.in_set(RenderSet::Queue))
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingEffect<Raindrops>>();
    }
}

fn prepare(
    data: Res<RaindropsData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &Order<Raindrops>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions
            .read()
            .id::<DrawPostProcessingEffect<Raindrops>>();

        phase.add(PostProcessingPhaseItem {
            entity,
            sort_key: (*order).into(),
            draw_function,
            pipeline_id: data.pipeline_id,
        });
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    data: Res<RaindropsData>,
    texture_handle: Res<RaindropsTextureHandle>,
    mut bind_group: ResMut<UniformBindGroup<Raindrops>>,
    uniforms: Res<ComponentUniforms<Raindrops>>,
    images: Res<RenderAssets<Image>>,
    views: Query<Entity, With<Raindrops>>,
) {
    bind_group.inner = None;

    if let (Some(uniforms), Some(raindrops_image)) =
        (uniforms.binding(), images.get(&texture_handle))
    {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("Raindrops Uniform Bind Group"),
                layout: &data.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&raindrops_image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&data.sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: uniforms.clone(),
                    },
                ],
            }));
        }
    }
}

/// Raindrops settings.
#[derive(Debug, Component, Clone, Copy, ShaderType)]
pub struct Raindrops {
    /// How quickly the raindrops animate.
    pub speed: f32,

    /// How much the raindrops warp the image.
    pub warping: f32,

    /// How zoomed in the raindrops texture is.
    pub zoom: f32,
}

impl Default for Raindrops {
    fn default() -> Self {
        Self {
            speed: 0.8,
            warping: 0.03,
            zoom: 1.0,
        }
    }
}

impl Display for Raindrops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Raindrops speed: {}, warping: {}, zoom: {}",
            self.speed, self.warping, self.zoom
        )
    }
}

impl ExtractComponent for Raindrops {
    type Query = (&'static Self, &'static Camera);
    type Filter = ();
    type Out = Self;

    fn extract_component((settings, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        if !camera.is_active {
            return None;
        }

        Some(*settings)
    }
}

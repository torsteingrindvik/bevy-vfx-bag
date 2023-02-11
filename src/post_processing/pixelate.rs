use std::fmt::Display;

use bevy::render::RenderSet;
pub(crate) use bevy::{
    asset::load_internal_asset,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
            BindingType, BufferBindingType, CachedRenderPipelineId, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
};

use crate::post_processing::{DrawPostProcessingEffect, UniformBindGroup};

use super::{PostProcessingPhaseItem, VfxOrdering};

pub(crate) const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11093977931118718560);

#[derive(Resource)]
pub(crate) struct PixelateData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayout,
}

impl FromWorld for PixelateData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Pixelate",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Pixelate::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            PIXELATE_SHADER_HANDLE.typed(),
        );

        PixelateData {
            pipeline_id,
            uniform_layout,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PIXELATE_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/",
                "pixelate.wgsl"
            ),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<Pixelate>::default())
            .add_plugin(UniformComponentPlugin::<Pixelate>::default());

        super::render_app(app)
            .add_system_to_schedule(
                ExtractSchedule,
                super::extract_post_processing_camera_phases::<Pixelate>,
            )
            .init_resource::<PixelateData>()
            .init_resource::<UniformBindGroup<Pixelate>>()
            .add_system(prepare.in_set(RenderSet::Prepare))
            .add_system(queue.in_set(RenderSet::Queue))
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingEffect<Pixelate>>();
    }
}

fn prepare(
    data: Res<PixelateData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &VfxOrdering<Pixelate>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions
            .read()
            .id::<DrawPostProcessingEffect<Pixelate>>();

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
    data: Res<PixelateData>,
    mut bind_group: ResMut<UniformBindGroup<Pixelate>>,
    uniforms: Res<ComponentUniforms<Pixelate>>,
    views: Query<Entity, With<Pixelate>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("Pixelate Uniform Bind Group"),
                layout: &data.uniform_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            }));
        }
    }
}

/// TODO
#[derive(Debug, ShaderType, Component, Clone, Copy)]
pub struct Pixelate {
    /// TODO
    pub block_size: f32,
}

impl Default for Pixelate {
    fn default() -> Self {
        Self { block_size: 8.0 }
    }
}

impl Display for Pixelate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pixelate block size: {}", self.block_size)
    }
}

impl ExtractComponent for Pixelate {
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

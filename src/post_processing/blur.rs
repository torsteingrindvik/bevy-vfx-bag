use std::fmt::Display;

use bevy::{
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
        RenderSet,
    },
};

use crate::post_processing::{DrawPostProcessingEffect, UniformBindGroup};

use super::{Order, PostProcessingPhaseItem};

pub(crate) const BLUR_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11044253213698850613);

#[derive(Resource)]
pub(crate) struct BlurData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayout,
}

impl FromWorld for BlurData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Blur",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Blur::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            BLUR_SHADER_HANDLE.typed(),
        );

        BlurData {
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
            BLUR_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "blur.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<Blur>::default())
            .add_plugin(UniformComponentPlugin::<Blur>::default());

        super::render_app(app)
            .add_system_to_schedule(
                ExtractSchedule,
                super::extract_post_processing_camera_phases::<Blur>,
            )
            .init_resource::<BlurData>()
            .init_resource::<UniformBindGroup<Blur>>()
            .add_system(prepare.in_set(RenderSet::Prepare))
            .add_system(queue.in_set(RenderSet::Queue))
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingEffect<Blur>>();
    }
}

fn prepare(
    data: Res<BlurData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &Order<Blur>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions.read().id::<DrawPostProcessingEffect<Blur>>();

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
    data: Res<BlurData>,
    mut bind_group: ResMut<UniformBindGroup<Blur>>,
    uniforms: Res<ComponentUniforms<Blur>>,
    views: Query<Entity, With<Blur>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("Blur Uniform Bind Group"),
                layout: &data.uniform_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            }));
        }
    }
}

/// Blur settings.
#[derive(Debug, Copy, Clone, Component, ShaderType)]
pub struct Blur {
    /// How blurry the output image should be.
    /// If `0.0`, no blur is applied.
    /// `1.0` is "fully blurred", but higher values will produce interesting results.
    pub amount: f32,

    /// How far away the blur should sample points away from the origin point
    /// when blurring.
    /// This is in UV coordinates, so small (positive) values are expected (`0.01` is a good start).
    pub kernel_radius: f32,
}

impl Default for Blur {
    fn default() -> Self {
        Self {
            amount: 0.5,
            kernel_radius: 0.01,
        }
    }
}

impl Display for Blur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blur amount: {}, radius: {}",
            self.amount, self.kernel_radius
        )
    }
}

impl ExtractComponent for Blur {
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

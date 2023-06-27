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

use crate::post_processing::UniformBindGroup;

use super::{DrawPostProcessingEffect, Order, PostProcessingPhaseItem};

const WAVE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1792660281364049744);

/// Wave parameters.
///
/// Note that the parameters for the X axis causes a wave effect
/// towards the left- and right sides of the screen.
/// For example, if we have 1 wave in the X axis,
/// we will have one part of the screen stretched towards the right
/// horizontally, and one part stretched towards the left.
#[derive(Default, Debug, Copy, Clone, Component, ShaderType)]
pub struct Wave {
    /// How many waves in the x axis.
    pub waves_x: f32,

    /// How many waves in the y axis.
    pub waves_y: f32,

    /// How fast the x axis waves oscillate.
    pub speed_x: f32,

    /// How fast the y axis waves oscillate.
    pub speed_y: f32,

    /// How much displacement the x axis waves cause.
    pub amplitude_x: f32,

    /// How much displacement the y axis waves cause.
    pub amplitude_y: f32,

    /// Padding.
    pub _padding: Vec2,
}

#[derive(Resource)]
pub(crate) struct WaveData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayout,
}

impl FromWorld for WaveData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Wave",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Wave::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            WAVE_SHADER_HANDLE.typed(),
        );

        WaveData {
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
            WAVE_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "wave.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<Wave>::default())
            .add_plugin(UniformComponentPlugin::<Wave>::default());

        super::render_app(app)
            .add_system(
                super::extract_post_processing_camera_phases::<Wave>.in_schedule(ExtractSchedule),
            )
            .init_resource::<WaveData>()
            .init_resource::<UniformBindGroup<Wave>>()
            .add_system(prepare.in_set(RenderSet::Prepare))
            .add_system(queue.in_set(RenderSet::Queue))
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingEffect<Wave>>();
    }
}

fn prepare(
    data: Res<WaveData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &Order<Wave>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions.read().id::<DrawPostProcessingEffect<Wave>>();

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
    data: Res<WaveData>,
    mut bind_group: ResMut<UniformBindGroup<Wave>>,
    uniforms: Res<ComponentUniforms<Wave>>,
    views: Query<Entity, With<Wave>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("Wave Uniform Bind Group"),
                layout: &data.uniform_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            }));
        }
    }
}

impl ExtractComponent for Wave {
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

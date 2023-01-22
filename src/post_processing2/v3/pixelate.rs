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
        RenderStage,
    },
};

use crate::post_processing2::v3::{DrawWithDynamicUniform, UniformBindGroup};

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
                    min_binding_size: Some(PixelateUniform::min_size()),
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
                "pixelate3.wgsl"
            ),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<PixelateSettings>::default())
            .add_plugin(UniformComponentPlugin::<PixelateUniform>::default());

        super::render_app(app)
            .add_system_to_stage(
                RenderStage::Extract,
                super::extract_post_processing_camera_phases::<PixelateSettings>,
            )
            .init_resource::<PixelateData>()
            .init_resource::<UniformBindGroup<PixelateUniform>>()
            .add_system_to_stage(RenderStage::Prepare, prepare)
            .add_system_to_stage(RenderStage::Queue, queue)
            .add_render_command::<PostProcessingPhaseItem, DrawWithDynamicUniform<PixelateUniform>>(
            );
    }
}

fn prepare(
    data: Res<PixelateData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &VfxOrdering<PixelateSettings>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions
            .read()
            .id::<DrawWithDynamicUniform<PixelateUniform>>();

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
    mut bind_group: ResMut<UniformBindGroup<PixelateUniform>>,
    uniforms: Res<ComponentUniforms<PixelateUniform>>,
    views: Query<Entity, With<PixelateUniform>>,
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
#[derive(Debug, ShaderType, Clone, Component)]
pub struct PixelateUniform {
    pub(crate) block_size: f32,
}

/// TODO
#[derive(Debug, Component, Clone, Copy)]
pub struct PixelateSettings {
    pub(crate) block_size: f32,
}

impl Default for PixelateSettings {
    fn default() -> Self {
        Self { block_size: 8.0 }
    }
}

impl ExtractComponent for PixelateSettings {
    type Query = (&'static Self, &'static Camera);
    type Filter = ();
    type Out = PixelateUniform;

    fn extract_component((settings, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        if !camera.is_active {
            return None;
        }

        Some(PixelateUniform {
            block_size: settings.block_size,
        })
    }
}

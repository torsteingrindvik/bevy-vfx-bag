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
        RenderStage,
    },
};

use crate::post_processing2::v3::{DrawWithDynamicUniform, UniformBindGroup};

use super::{PostProcessingPhaseItem, VfxOrdering};

pub(crate) const FLIP_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1649866799156783187);

#[derive(Resource)]
pub(crate) struct FlipData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayout,
}

impl FromWorld for FlipData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Flip",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(FlipUniform::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            FLIP_SHADER_HANDLE.typed(),
        );

        FlipData {
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
            FLIP_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "flip3.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<FlipSettings>::default())
            .add_plugin(UniformComponentPlugin::<FlipUniform>::default());

        super::render_app(app)
            .add_system_to_stage(
                RenderStage::Extract,
                super::extract_post_processing_camera_phases::<FlipSettings>,
            )
            .init_resource::<FlipData>()
            .init_resource::<UniformBindGroup<FlipUniform>>()
            .add_system_to_stage(RenderStage::Prepare, prepare)
            .add_system_to_stage(RenderStage::Queue, queue)
            .add_render_command::<PostProcessingPhaseItem, DrawWithDynamicUniform<FlipUniform>>();
    }
}

fn prepare(
    data: Res<FlipData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &VfxOrdering<FlipSettings>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions
            .read()
            .id::<DrawWithDynamicUniform<FlipUniform>>();

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
    data: Res<FlipData>,
    mut bind_group: ResMut<UniformBindGroup<FlipUniform>>,
    uniforms: Res<ComponentUniforms<FlipUniform>>,
    views: Query<Entity, With<FlipUniform>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("Flip Uniform Bind Group"),
                layout: &data.uniform_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            }));
        }
    }
}

/// The uniform representation of [`FlipSettings`].
#[derive(Debug, ShaderType, Clone, Component)]
pub struct FlipUniform {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl From<FlipSettings> for FlipUniform {
    fn from(flip: FlipSettings) -> Self {
        let uv = match flip {
            FlipSettings::None => [0.0, 0.0],
            FlipSettings::Horizontal => [1.0, 0.0],
            FlipSettings::Vertical => [0.0, 1.0],
            FlipSettings::HorizontalVertical => [1.0, 1.0],
        };

        Self { x: uv[0], y: uv[1] }
    }
}

/// Which way to flip the texture.
#[derive(Debug, Default, Copy, Clone, Component)]
pub enum FlipSettings {
    /// Don't flip.
    None,

    /// Flip horizontally.
    #[default]
    Horizontal,

    /// Flip vertically.
    Vertical,

    /// Flip both axes.
    HorizontalVertical,
}

impl ExtractComponent for FlipSettings {
    type Query = (&'static Self, &'static Camera);
    type Filter = ();
    type Out = FlipUniform;

    fn extract_component((settings, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        if !camera.is_active {
            return None;
        }

        Some((*settings).into())
    }
}

use std::marker::PhantomData;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
            CachedRenderPipelineId, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, ShaderDefVal, ShaderStages, ShaderType, TextureFormat,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        RenderStage,
    },
    utils::FloatOrd,
};

use super::{DrawPostProcessing, PostProcessingPhaseItem, SetTextureSamplerGlobals};

pub(crate) const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11093977931118718560);

/// TODO
struct SetDynamicUniform<U: Component + ShaderType, const I: usize>(PhantomData<U>);
impl<P: PhaseItem, U: Component + ShaderType, const I: usize> RenderCommand<P>
    for SetDynamicUniform<U, I>
{
    type Param = SRes<PixelateData>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<DynamicUniformIndex<U>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        uniform_index: ROQueryItem<'w, Self::ItemWorldQuery>,
        pixelate_data: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = pixelate_data.into_inner().uniform_bind_group.as_ref() {
            pass.set_bind_group(I, bind_group, &[uniform_index.index()]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

type DrawPixelate<U> = (
    // The pipeline must be set in order to use the correct bind group,
    // access the correct shaders, and so on.
    SetItemPipeline,
    // Common to post processing items is that they all use the same
    // first bind group, which has the input texture (the scene) and
    // the sampler for that.
    SetTextureSamplerGlobals<0>,
    // TODO
    SetDynamicUniform<U, 1>,
    // Lastly we draw vertices.
    // This is simple for a post processing effect, since we just draw
    // a full screen triangle.
    DrawPostProcessing,
);

#[derive(Resource)]
pub(crate) struct PixelateData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayout,
    pub uniform_bind_group: Option<BindGroup>,
}

impl FromWorld for PixelateData {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let shared_layout = &world
            .resource::<super::PostProcessingSharedLayout>()
            .shared_layout;

        let uniform_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Pixelate Uniform Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(PixelateUniform::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
        });

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("Pixelate Pipeline".into()),
            layout: Some(vec![shared_layout.clone(), uniform_layout.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: PIXELATE_SHADER_HANDLE.typed(),
                shader_defs: vec![ShaderDefVal::Int("MAX_DIRECTIONAL_LIGHTS".to_string(), 1)],
                entry_point: "fragment".into(),
                targets: vec![Some(TextureFormat::bevy_default().into())],
            }),
        });

        PixelateData {
            pipeline_id,
            uniform_bind_group: None,
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
            .add_system_to_stage(RenderStage::Prepare, prepare)
            .add_system_to_stage(RenderStage::Queue, queue)
            .add_render_command::<PostProcessingPhaseItem, DrawPixelate<PixelateUniform>>();
    }
}

fn prepare(
    data: Res<PixelateData>,
    mut views: Query<(Entity, &mut RenderPhase<PostProcessingPhaseItem>)>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase) in views.iter_mut() {
        let draw_function = draw_functions.read().id::<DrawPixelate<PixelateUniform>>();

        phase.add(PostProcessingPhaseItem {
            entity,
            sort_key: FloatOrd(0.0), // TODO ordering
            draw_function,
            pipeline_id: data.pipeline_id,
        });
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    mut data: ResMut<PixelateData>,
    uniforms: Res<ComponentUniforms<PixelateUniform>>,
    views: Query<Entity, With<PixelateUniform>>,
) {
    data.uniform_bind_group = None;

    if let Some(uniforms) = uniforms.binding() {
        if !views.is_empty() {
            data.uniform_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
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

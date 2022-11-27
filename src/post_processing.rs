use std::{marker::PhantomData, sync::Mutex};

use bevy::{
    core_pipeline::{core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state},
    prelude::{
        default, Commands, Component, Entity, FromWorld, Plugin, Query, QueryState, Res, ResMut,
        With, World,
    },
    render::{
        extract_component::{DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{Node, RenderGraph},
        render_phase::TrackedRenderPass,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FilterMode, FragmentState,
            MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
            ShaderStages, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines,
            TextureFormat, TextureSampleType, TextureViewDimension, TextureViewId,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
};

use bevy::{prelude::Resource, render::render_resource::BindGroupLayout};

use self::traits::PostProcessingNode;

/// Pixelation effect.
pub mod pixelate;

/// Chromatic aberration effect.
pub mod chromatic_aberration;

mod traits {
    use bevy::render::render_resource::BindGroupLayout;
    use bevy::render::render_resource::BindingResource;
    use bevy::render::render_resource::CachedRenderPipelineId;
    use bevy::{prelude::Component, render::extract_component::ExtractComponent};
    use bevy::{
        prelude::{Camera, FromWorld, With, World},
        render::render_resource::ShaderRef,
    };
    use std::ops::Deref;

    pub(crate) trait PostProcessingNode {
        const IN_VIEW: &'static str;

        // Key for specialization
        type Key;

        type Uniform: Component;
        // type Pipeline: Resource + SpecializedRenderPipeline;
        type ComponentPipeline: Component
            + Deref<Target = CachedRenderPipelineId>
            + From<CachedRenderPipelineId>;

        fn shader_defs(&self, key: Self::Key) -> Vec<String>;

        fn shader(&self) -> Handle<Shader>;

        fn bind_group_layout(&self) -> &BindGroupLayout;

        fn binding_resource(&self, world: &World) -> BindingResource;

        fn pass_label(&self) -> Option<&'static str>;
    }

    pub(crate) trait PostProcessingPlugin: 'static {
        const NODE_NAME_3D: &'static str;

        type UserSettings: Component;
        type Uniform: Component
            + ExtractComponent<Query = &'static Self, Filter = With<Camera>, Out = Self::UserSettings>;
        type Node: PostProcessingNode + FromWorld;
    }
}

#[derive(Component)]
struct PerCameraPipelineId<T> {
    pipeline_id: CachedRenderPipelineId,
    marker: PhantomData<T>,
}

#[derive(Resource)]
struct PostProcessingPipeline<T> {
    bind_group_layout: BindGroupLayout,
    marker: PhantomData<T>,
}

impl<T: ShaderType> FromWorld for PostProcessingPipeline<T> {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None, // TODO
                    entries: &[
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
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: true,
                                min_binding_size: Some(T::min_size()),
                            },
                            count: None,
                        },
                    ],
                });

        Self {
            bind_group_layout,
            marker: PhantomData,
        }
    }
}

/// TODO
#[derive(Default)]
pub struct PostProcessingPlugin<T: traits::PostProcessingPlugin> {
    marker: PhantomData<T>,
}

fn prepare_post_processing_pipelines<
    T: SpecializedRenderPipeline + Resource + traits::PostProcessingNode,
    U: Component,
>(
    mut commands: Commands,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<T>>,
    pipeline_resource: Res<T>,
    views: Query<(Entity, &ExtractedView), With<U>>,
) where
    <T as SpecializedRenderPipeline>::Key: Sync + Send,
{
    for (entity, view) in &views {
        let pipeline_id = pipelines.specialize(&mut pipeline_cache, &pipeline_resource, ());

        commands
            .entity(entity)
            .insert(T::ComponentPipeline::from(pipeline_id));
    }
}

impl<T: traits::PostProcessingPlugin + Send + Sync + 'static> Plugin for PostProcessingPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        // TODO: load internal asset

        app.add_plugin(ExtractComponentPlugin::<T::UserSettings>::default());
        app.add_plugin(UniformComponentPlugin::<T::Uniform>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .init_resource::<PostProcessingPipeline<T>>()
            .init_resource::<SpecializedRenderPipeline<PostProcessingPipeline<T>>>()
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_post_processing_pipelines::<T, T::Uniform>,
            );

        {
            let node = <T::Node as FromWorld>::from_world(&mut render_app.world);

            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding
                .get_sub_graph_mut(core_3d::graph::NAME)
                .expect("Graph should be available");

            graph.add_node(T::NODE_NAME_3D, node);

            graph.add_slot_edge(
                graph.input_node().id,
                core_3d::graph::input::VIEW_ENTITY,
                T::NODE_NAME_3D,
                "view",
            );

            graph.add_node_edge(core_3d::graph::node::MAIN_PASS, T::NODE_NAME_3D);

            graph.add_node_edge(
                T::NODE_NAME_3D,
                core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
            );
        }
    }
}

struct PostProcessingNode<T: traits::PostProcessingNode + 'static> {
    inner: T,
    query: QueryState<
        (
            &'static ViewTarget,
            &'static T::ComponentPipeline,
            &'static DynamicUniformIndex<T::Uniform>,
        ),
        With<ExtractedView>,
    >,
    cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
}

impl<T: traits::PostProcessingNode + 'static> PostProcessingNode<T> {
    fn new(node: T, world: &mut World) -> Self {
        Self {
            inner: node,
            query: QueryState::new(world),
            cached_texture_bind_group: Mutex::new(None),
        }
    }
}

// impl<T> Deref for PostProcessingNode<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<T> DerefMut for PostProcessingNode<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

impl<T: traits::PostProcessingNode> SpecializedRenderPipeline for PostProcessingPipeline<T> {
    type Key = T::Key;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: self.pass_label(),
            layout: Some(vec![self.bind_group_layout().clone()]),
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: self.shader(),
                shader_defs: self.shader_defs(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}

impl<T: traits::PostProcessingNode + Send + Sync + 'static> Node for PostProcessingNode<T> {
    fn update(&mut self, world: &mut bevy::prelude::World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &bevy::prelude::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let view_entity = graph.get_input_entity(T::IN_VIEW)?;

        let pipeline_cache = world.resource::<PipelineCache>();
        // let pixelate_pipeline = world.resource::<PixelatePipeline>();
        // let pixelate_uniforms = world.resource::<ComponentUniforms<PixelateUniform>>();
        // let effect_pipeline = world.resource::<T::Pipeline>();

        // let (target, pipeline, uniform_index) = match self.query.get_manual(world, view_entity) {
        //     Ok(result) => result,
        //     Err(_) => return Ok(()),
        // };
        let (target, pipeline, uniform_index) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        let pipeline = pipeline_cache
            .get_render_pipeline(**pipeline)
            .expect("Render pipeline should be cached");

        let post_process = target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;

        let mut cached_bind_group = self
            .cached_texture_bind_group
            .lock()
            .expect("Lock not held");

        let bind_group = match &mut *cached_bind_group {
            Some((id, bind_group)) if source.id() == *id => bind_group,
            cached_bind_group => {
                let sampler = render_context
                    .render_device
                    .create_sampler(&SamplerDescriptor {
                        mipmap_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Linear,
                        ..default()
                    });

                let bind_group =
                    render_context
                        .render_device
                        .create_bind_group(&BindGroupDescriptor {
                            label: None,
                            layout: &self.inner.bind_group_layout(),
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(source),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(&sampler),
                                },
                                BindGroupEntry {
                                    binding: 2,
                                    resource: self.inner.binding_resource(world),
                                },
                            ],
                        });

                let (_, bind_group) = cached_bind_group.insert((source.id(), bind_group));
                bind_group
            }
        };

        let pass_descriptor = RenderPassDescriptor {
            label: self.inner.pass_label(),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        };

        let mut render_pass = TrackedRenderPass::new(
            render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor),
        );

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[uniform_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

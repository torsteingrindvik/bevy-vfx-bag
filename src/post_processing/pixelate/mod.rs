use bevy::{
    asset::load_internal_asset,
    core_pipeline::{core_2d, core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state},
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::RenderGraph,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BufferBindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
};

use node::PixelateNode;

mod node;

const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17030123524594138798);

/// The pixelation node name.
pub const PIXELATE_NODE_3D: &str = "pixelate_node_3d";

/// The pixelation node name.
pub const PIXELATE_NODE_2D: &str = "pixelate_node_2d";

/// Pixelation parameters.
#[derive(Component, Clone)]
pub struct Pixelate {
    /// Whether the effect should run or not.
    pub enabled: bool,

    /// How many pixels in the width and height in a block after pixelation.
    /// One block has a constant color within it.
    ///
    /// The shader sets a lower bound to 1.0, since that would not change the outcome.
    pub block_size: f32,
}

impl Default for Pixelate {
    fn default() -> Self {
        Self {
            enabled: true,
            block_size: 4.0,
        }
    }
}

impl ExtractComponent for Pixelate {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = PixelateUniform;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        if item.enabled {
            Some(PixelateUniform {
                block_size: item.block_size,
            })
        } else {
            None
        }
    }
}

/// TODO
#[derive(Debug, ShaderType, Component, Clone)]
pub struct PixelateUniform {
    block_size: f32,
}

// impl From<Pixelate> for PixelateUniform {
//     fn from(pixelate: Pixelate) -> Self {
//         Self {
//             block_size: pixelate.block_size,
//         }
//     }
// }

// #[derive(Component)]
// struct PixelateUniformIndex(u32);

/// This plugin allows pixelating the image.
pub struct PixelatePlugin;

impl Plugin for PixelatePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PIXELATE_SHADER_HANDLE,
            "pixelate.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<Pixelate>::default());
        app.add_plugin(UniformComponentPlugin::<PixelateUniform>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .init_resource::<PixelatePipeline>()
            .init_resource::<SpecializedRenderPipelines<PixelatePipeline>>()
            .add_system_to_stage(RenderStage::Prepare, prepare_pixelate_pipelines);

        {
            let node = PixelateNode::new(&mut render_app.world);
            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding
                .get_sub_graph_mut(core_3d::graph::NAME)
                .expect("Graph should be available");

            graph.add_node(PIXELATE_NODE_3D, node);

            graph
                .add_slot_edge(
                    graph.input_node().expect("Graph should have input node").id,
                    core_3d::graph::input::VIEW_ENTITY,
                    PIXELATE_NODE_3D,
                    PixelateNode::IN_VIEW,
                )
                .expect("Slot edge add should always work");

            graph
                .add_node_edge(core_3d::graph::node::MAIN_PASS, PIXELATE_NODE_3D)
                .expect("Node edge should be succesfully added");

            graph
                .add_node_edge(
                    PIXELATE_NODE_3D,
                    core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                )
                .expect("Node edge should be succesfully added");
        }
        {
            let node = PixelateNode::new(&mut render_app.world);
            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding
                .get_sub_graph_mut(core_2d::graph::NAME)
                .expect("Graph should be available");

            graph.add_node(PIXELATE_NODE_2D, node);

            graph
                .add_slot_edge(
                    graph.input_node().expect("Graph should have input node").id,
                    core_2d::graph::input::VIEW_ENTITY,
                    PIXELATE_NODE_2D,
                    PixelateNode::IN_VIEW,
                )
                .expect("Slot edge add should always work");

            graph
                .add_node_edge(
                    PIXELATE_NODE_2D,
                    core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                )
                .expect("Node edge should be succesfully added");
        }
    }
}

/// The pipeline used by the pixelate effect.
#[derive(Resource, Deref)]
pub struct PixelatePipeline {
    texture_bind_group: BindGroupLayout,
}

impl FromWorld for PixelatePipeline {
    fn from_world(render_world: &mut World) -> Self {
        let texture_bind_group = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("pixelate_texture_bind_group_layout"),
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
                            min_binding_size: Some(PixelateUniform::min_size()),
                        },
                        count: None,
                    },
                ],
            });

        PixelatePipeline { texture_bind_group }
    }
}

/// Contains the pipeline id for the pixelate effect.
#[derive(Component)]
pub struct CameraPixelatePipeline {
    /// The id of the cached render pipeline.
    pub pipeline_id: CachedRenderPipelineId,
}

/// The key for specializing the pixelate pipeline.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PixelatePipelineKey {
    texture_format: TextureFormat,
}

impl SpecializedRenderPipeline for PixelatePipeline {
    type Key = PixelatePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("pixelate".into()),
            layout: Some(vec![self.texture_bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: PIXELATE_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        }
    }
}

/// Prepare the pixelate pipelines.
/// Each camera with the effect is specialized on whether it uses HDR.
/// Then a component with the id of this pipeline is stored in a component on the entity.
fn prepare_pixelate_pipelines(
    mut commands: Commands,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PixelatePipeline>>,
    pixelate_pipeline: Res<PixelatePipeline>,
    views: Query<(Entity, &ExtractedView), With<PixelateUniform>>,
) {
    for (entity, view) in &views {
        let pipeline_id = pipelines.specialize(
            &mut pipeline_cache,
            &pixelate_pipeline,
            PixelatePipelineKey {
                texture_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
            },
        );

        commands
            .entity(entity)
            .insert(CameraPixelatePipeline { pipeline_id });
    }
}

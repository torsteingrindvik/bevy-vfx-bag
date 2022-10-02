use bevy::{
    core::{Pod, Zeroable},
    ecs::system::lifetimeless::SRes,
    prelude::{Plugin, *},
    render::{
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
        },
        globals::{GlobalsBuffer, GlobalsUniform},
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{
            sort_phase_system, AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId,
            DrawFunctions, EntityPhaseItem, EntityRenderCommand, PhaseItem, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
            BufferUsages, BufferVec, CachedRenderPipelineId, LoadOp, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            ShaderStages, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::{ViewTarget, ViewUniform, ViewUniforms},
        Extract, RenderApp, RenderStage,
    },
};

use crate::{nodes, quad};

/// This plugin allows adding a vignette effect to a given camera.
/// Add this plugin to the [`App`] in order to use it.
/// Then, add the [`Vignette`] component to the camera you want the effect to apply to.
pub struct VignettePlugin;

struct VignetteRenderPhase {
    draw_function: DrawFunctionId,
    entity: Entity,
    pipeline: CachedRenderPipelineId,
}

impl CachedRenderPipelinePhaseItem for VignetteRenderPhase {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

impl EntityPhaseItem for VignetteRenderPhase {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl PhaseItem for VignetteRenderPhase {
    type SortKey = u8;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        0
    }

    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

struct VignetteNode {
    query: QueryState<(
        &'static RenderPhase<VignetteRenderPhase>,
        &'static ViewTarget,
    )>,
}

impl VignetteNode {
    fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for VignetteNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(nodes::VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let input_view_entity = graph.get_input_entity(nodes::VIEW)?;

        let (phase, target) = if let Ok(stuff) = self.query.get_manual(world, input_view_entity) {
            stuff
        } else {
            return Ok(());
        };
        if phase.items.is_empty() {
            return Ok(());
        }

        let pass_descriptor = RenderPassDescriptor {
            label: Some("Vignette RenderPass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        let draw_functions = world.resource::<DrawFunctions<VignetteRenderPhase>>();

        let render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        let mut draw_functions = draw_functions.write();
        let mut tracked_pass = TrackedRenderPass::new(render_pass);

        for item in &phase.items {
            let draw_function = draw_functions
                .get_mut(item.draw_function)
                .expect("DrawFunctionId should be valid");
            draw_function.draw(world, &mut tracked_pass, input_view_entity, item);
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct VignetteVertex {
    position: [f32; 2],
}

impl From<Vec2> for VignetteVertex {
    fn from(vec2: Vec2) -> Self {
        Self {
            position: vec2.into(),
        }
    }
}

#[derive(Resource)]
struct VignetteVertices {
    vertices: BufferVec<VignetteVertex>,
}

impl Default for VignetteVertices {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
        }
    }
}

fn prepare_vignette_quad(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut vignette_vertices: ResMut<VignetteVertices>,
) {
    quad::write_quad_buffer(
        &mut vignette_vertices.vertices,
        &render_device,
        &render_queue,
    );
}

#[derive(Resource)]
struct VignetteBindGroup {
    bind_group: BindGroup,
}

/// This component enables a vignette effect on the camera it is insert onto.
/// Assumes the [`VignettePlugin`] has been added to the [`App`].
#[derive(Debug, Component, Clone)]
pub struct Vignette {
    /// The radius of the effect.
    /// A radius of 1.0 will cover the entire screen (in both axes).
    /// A radius of less than 1.0 will start shrinking the effect towards the center of the screen.
    pub radius: f32,

    /// The distance of the smooth transition between the effect and the scene.
    /// Note that this will add to the radius of the effect.
    pub feathering: f32,

    /// The color of the vignette.
    /// Note that the alpha channel is used by the effect.
    pub color: Color,
}

impl Vignette {
    /// Create a vignette effect with the given parameters.
    pub fn new(radius: f32, feathering: f32, color: Color) -> Self {
        Self {
            radius,
            feathering,
            color,
        }
    }
}

impl ExtractComponent for Vignette {
    type Query = &'static Self;
    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

impl Default for Vignette {
    fn default() -> Self {
        let mut color = Color::BLACK;
        color.set_a(0.8);

        Self {
            radius: 1.0,
            feathering: 0.1,
            color,
        }
    }
}

#[derive(Debug, Component, ShaderType, Clone)]
struct VignetteUniform {
    radius: f32,
    feathering: f32,
    color: Color,
}

impl From<Vignette> for VignetteUniform {
    fn from(config: Vignette) -> Self {
        Self {
            radius: config.radius,
            feathering: config.feathering,
            color: config.color,
        }
    }
}

impl Default for VignetteUniform {
    fn default() -> Self {
        Vignette::default().into()
    }
}

fn queue_vignette_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    vignette_pipeline: Res<VignettePipeline>,
    globals_buffer: Res<GlobalsBuffer>,
    view_uniforms: Res<ViewUniforms>,
    vignette_uniforms: Res<ComponentUniforms<VignetteUniform>>,
) {
    if let (Some(globals_binding), Some(view_binding), Some(vignette_binding)) = (
        globals_buffer.buffer.binding(),
        view_uniforms.uniforms.binding(),
        vignette_uniforms.binding(),
    ) {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("Vignette BindGroupDescriptor"),
            layout: &vignette_pipeline.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals_binding.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: view_binding.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: vignette_binding.clone(),
                },
            ],
        });

        commands.insert_resource(VignetteBindGroup { bind_group });
    }
}

fn queue_vignette_phase_items(
    mut commands: Commands,
    mut views: Query<&mut RenderPhase<VignetteRenderPhase>>,
    draw_functions: Res<DrawFunctions<VignetteRenderPhase>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<VignettePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    vignette_pipeline: Res<VignettePipeline>,
) {
    let draw_function = draw_functions
        .read()
        .get_id::<VignetteDrawFunctions>()
        .expect("Should get valid DrawFunctionId");

    for mut render_phase in views.iter_mut() {
        let pipeline = pipelines.specialize(&mut pipeline_cache, &vignette_pipeline, ());

        let id = commands.spawn_empty().id();

        render_phase.add(VignetteRenderPhase {
            draw_function,
            entity: id,
            pipeline,
        })
    }
}

fn extract_vignette_cameras(
    mut commands: Commands,
    query: Extract<Query<(Entity, &Camera, &Vignette)>>,
) {
    for (entity, camera, vignette) in query.iter() {
        if !camera.is_active {
            continue;
        }

        commands
            .get_or_spawn(entity)
            .insert(VignetteUniform::from(vignette.clone()))
            .insert(RenderPhase::<VignetteRenderPhase>::default());
    }
}

type VignetteDrawFunctions = (
    SetItemPipeline,
    SetVignetteBindGroup<0>,
    DrawVignetteVertices,
);

struct SetVignetteBindGroup<const I: usize>();

impl<const I: usize> EntityRenderCommand for SetVignetteBindGroup<I> {
    type Param = SRes<VignetteBindGroup>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        bind_group: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_group = &bind_group.into_inner().bind_group;
        pass.set_bind_group(I, bind_group, &[]);

        RenderCommandResult::Success
    }
}

struct DrawVignetteVertices;

impl EntityRenderCommand for DrawVignetteVertices {
    type Param = SRes<VignetteVertices>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        my_vertices: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        pass.set_vertex_buffer(
            0,
            my_vertices
                .into_inner()
                .vertices
                .buffer()
                .expect("VignetteVertices should have a buffer")
                .slice(..),
        );
        let n = quad::QUAD_VERTEX_POSITIONS.len() as u32;
        pass.draw(0..n, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct VignettePipeline {
    layout: BindGroupLayout,
    shader: Handle<Shader>,
}

impl FromWorld for VignettePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Vignette BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(VignetteUniform::min_size()),
                    },
                    count: None,
                },
            ],
        });

        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/vignette.wgsl");

        Self { shader, layout }
    }
}

impl SpecializedRenderPipeline for VignettePipeline {
    type Key = ();

    fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
        quad::quad_render_pipeline_descriptor(
            "Vignette RenderPipeline",
            self.shader.clone(),
            self.layout.clone(),
        )
    }
}

impl Plugin for VignettePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<Vignette>::default())
            .add_plugin(UniformComponentPlugin::<VignetteUniform>::default());

        let render_app = app
            .get_sub_app_mut(RenderApp)
            .expect("Should be able to get RenderApp SubApp");

        render_app
            .init_resource::<VignettePipeline>()
            .init_resource::<SpecializedRenderPipelines<VignettePipeline>>()
            .init_resource::<VignetteVertices>()
            .init_resource::<DrawFunctions<VignetteRenderPhase>>()
            .add_render_command::<VignetteRenderPhase, VignetteDrawFunctions>()
            .add_system_to_stage(RenderStage::Extract, extract_vignette_cameras)
            .add_system_to_stage(RenderStage::Prepare, prepare_vignette_quad)
            .add_system_to_stage(RenderStage::Queue, queue_vignette_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_vignette_phase_items)
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<VignetteRenderPhase>,
            );

        let node = VignetteNode::new(&mut render_app.world);

        nodes::add_node_before_ui_pass(app, node, "VignetteNode");
    }
}

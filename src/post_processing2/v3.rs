use std::{any::TypeId, marker::PhantomData, sync::Mutex};

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        globals::{GlobalsBuffer, GlobalsUniform},
        mesh::MeshVertexBufferLayout,
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{
            sort_phase_system, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions,
            EntityRenderCommand, PhaseItem, RenderCommandResult, RenderPhase, SetItemPipeline,
            TrackedRenderPass,
        },
        render_resource::{
            AsBindGroup, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites, FilterMode,
            FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            SamplerBindingType, SamplerDescriptor, ShaderDefVal, ShaderRef, ShaderStages,
            ShaderType, SpecializedMeshPipelineError, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureSampleType, TextureViewDimension, TextureViewId,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        Extract, RenderApp, RenderStage,
    },
    sprite::{
        Material2d, Material2dKey, Material2dPipeline, Material2dPlugin, SetMaterial2dBindGroup,
    },
    utils::{FloatOrd, HashMap},
};

use super::util;

type DrawPostProcessingItem<M> = (
    SetItemPipeline,
    SetTextureAndSampler<0>,
    SetMaterial2dBindGroup<M, 1>,
    DrawPostProcessing,
);

/// Bind groups.
#[derive(Resource, Default)]
pub struct PostProcessingBindGroups {
    cached_texture_bind_groups: HashMap<TextureViewId, BindGroup>,
    current_source_texture: Mutex<Option<TextureViewId>>,
}

/// Render command which sets the shared bind group.
pub struct SetTextureAndSampler<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetTextureAndSampler<I> {
    type Param = SRes<PostProcessingBindGroups>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let id = {
            let lock = bind_groups
                .current_source_texture
                .lock()
                .expect("Mutex should be available");
            *lock.as_ref().expect("Source view id should be set")
        };

        let bind_group = bind_groups
            .into_inner()
            .cached_texture_bind_groups
            .get(&id)
            .expect("Source texture view should be available");

        pass.set_bind_group(I, bind_group, &[]);
        RenderCommandResult::Success
    }
}

/// Render command for drawing the full screen triangle.
pub struct DrawPostProcessing;

impl EntityRenderCommand for DrawPostProcessing {
    type Param = ();
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        _query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Draw the full screen triangle.
        pass.draw(0..3, 0..1);
        RenderCommandResult::Success
    }
}

/// The post processing node.
pub struct PostProcessingNode {
    query: QueryState<
        (
            &'static RenderPhase<PostProcessingPhaseItem>,
            &'static ViewTarget,
        ),
        With<ExtractedView>,
    >,
}

impl PostProcessingNode {
    /// The slot input name.
    pub const IN_VIEW: &'static str = "view";

    /// Createa a new post processing node.
    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl FromWorld for PostProcessingNode {
    fn from_world(world: &mut World) -> Self {
        Self::new(world)
    }
}

fn queue_post_processing_shared_bind_group(
    // mut commands: Commands,
    render_device: Res<RenderDevice>,
    globals: Res<GlobalsBuffer>,
    layout: Res<PostProcessingSharedLayout>,
    mut bind_groups: ResMut<PostProcessingBindGroups>,
    // uniforms: Res<ComponentUniforms<BloomUniform>>,

    // TODO: Identify somehow which view targets are being used by post processing.
    views: Query<(Entity, &ViewTarget), With<RaindropsMarker>>,
) {
    for (_, view_target) in &views {
        let id = view_target.main_texture().id();

        if !bind_groups.cached_texture_bind_groups.contains_key(&id) {
            bind_groups.cached_texture_bind_groups.insert(
                id,
                render_device.create_bind_group(&BindGroupDescriptor {
                    label: Some("PostProcessing texture bind group"),
                    layout: &layout.textures_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(view_target.main_texture()),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Sampler(&render_device.create_sampler(
                                &SamplerDescriptor {
                                    label: Some("PostProcessing texture sampler"),
                                    mag_filter: FilterMode::Linear,
                                    min_filter: FilterMode::Linear,
                                    mipmap_filter: FilterMode::Linear,
                                    ..default()
                                },
                            )),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: globals
                                .buffer
                                .binding()
                                .expect("Globals buffer should be available"),
                        },
                    ],
                }),
            );
        }
    }
}

/// A post processing phase item.
/// Contains a draw function which is specialized for a specific material.
/// Points to a matching pipeline- it will for example point to a specific fragment shader as well as
/// having a bind group specialized for the material.
pub struct PostProcessingPhaseItem {
    sort_key: FloatOrd,
    draw_function: DrawFunctionId,
    pipeline_id: CachedRenderPipelineId,
}

impl PhaseItem for PostProcessingPhaseItem {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

impl CachedRenderPipelinePhaseItem for PostProcessingPhaseItem {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline_id
    }
}

/// The bind group layout common to post processing effects.
/// This includes the texture and sampler bind group entries and the globals uniform.
#[derive(Debug, Resource, Clone)]
pub struct PostProcessingSharedLayout {
    textures_layout: BindGroupLayout,
}

impl FromWorld for PostProcessingSharedLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world
            .get_resource::<RenderDevice>()
            .expect("RenderDevice should be available");

        let textures_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("PostProcessing texture bind group layout"),
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
                        has_dynamic_offset: false,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                },
            ],
        });

        Self { textures_layout }
    }
}

/// This resource contains the post processing layout for a specific material.
#[derive(Debug, Resource)]
pub struct PostProcessingLayout<M: Material2d> {
    shared: PostProcessingSharedLayout,
    material_layout: BindGroupLayout,
    marker: PhantomData<M>,
}

impl<M> FromWorld for PostProcessingLayout<M>
where
    M: Material2d,
{
    fn from_world(world: &mut World) -> Self {
        let shared = world
            .get_resource::<PostProcessingSharedLayout>()
            .expect("Shared layout should be available");
        let material_layout = world
            .get_resource::<Material2dPipeline<M>>()
            .expect("Resource should be available")
            .material2d_layout
            .clone();

        Self {
            shared: shared.clone(),
            material_layout,
            marker: PhantomData,
        }
    }
}

impl<M: Material2d> SpecializedRenderPipeline for PostProcessingLayout<M> {
    type Key = ();

    fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
        let shader = match M::fragment_shader() {
            ShaderRef::Handle(handle) => handle,
            _others => unimplemented!("ShaderRef non-handle is not supported"),
        };

        RenderPipelineDescriptor {
            label: Some("PostProcessing pipeline".into()),
            layout: Some(vec![
                self.shared.textures_layout.clone(),
                self.material_layout.clone(),
            ]),
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader,
                // TODO: Get rid of this when Bevy supports it.
                shader_defs: vec![ShaderDefVal::Int("MAX_DIRECTIONAL_LIGHTS".to_string(), 1)],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: bevy::render::render_resource::TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}

impl Node for PostProcessingNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
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
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;

        let bind_groups = world.resource::<PostProcessingBindGroups>();

        let (phase, target) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        // TODO: Handle HDR.

        let draw_functions = world.resource::<DrawFunctions<PostProcessingPhaseItem>>();
        let mut draw_functions = draw_functions.write();
        for item in &phase.items {
            let post_process = target.post_process_write();
            let source = post_process.source;
            let destination = post_process.destination;

            bind_groups
                .current_source_texture
                .lock()
                .expect("Mutex should be unused")
                .replace(source.id());

            let pass_descriptor = RenderPassDescriptor {
                label: Some("PostProcessing pass"),
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

            let draw_function = draw_functions
                .get_mut(item.draw_function)
                .expect("Should get draw function");
            draw_function.draw(world, &mut render_pass, view_entity, item);
        }

        Ok(())
    }
}

/// Decide on ordering for post processing effects.
/// TODO: Describe if higher values or lower values means first.
#[derive(Debug, Component)]
pub struct VfxOrdering(f32);

/// Adds the phase items. Each time one is added it means that a render pass for the effect will be performed.
#[allow(clippy::complexity)]
pub fn queue_post_processing_phase_items<M: Material2d, C: Component>(
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
    pipeline: Res<PostProcessingLayout<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PostProcessingLayout<M>>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut views: Query<
        (
            Entity,
            &mut RenderPhase<PostProcessingPhaseItem>,
            Option<&VfxOrdering>,
        ),
        With<C>,
    >,
) {
    for (_, mut phase, ordering) in views.iter_mut() {
        info!(
            "Adding post processing phase items: {:?}",
            TypeId::of::<C>()
        );

        let pipeline_id = pipelines.specialize(&mut pipeline_cache, &pipeline, ());

        let draw_function = draw_functions.read().id::<DrawPostProcessingItem<M>>();

        phase.add(PostProcessingPhaseItem {
            sort_key: FloatOrd(ordering.map(|ordering| ordering.0).unwrap_or_default()), // todo
            draw_function,
            pipeline_id,
        })
    }
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "4fba30ae-73e6-11ed-8575-9b008b9044f0"]
struct Raindrops {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Option<Handle<Image>>,
}

impl Material2d for Raindrops {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn specialize(
        _descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

#[derive(Debug, Component, Clone, Copy)]
struct RaindropsMarker;

impl ExtractComponent for RaindropsMarker {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

/// TODO
pub struct Pluginz {}

impl Plugin for Pluginz {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<Raindrops>::default())
            .add_plugin(ExtractComponentPlugin::<RaindropsMarker>::default());

        let render_app = app.get_sub_app_mut(RenderApp).expect("Should work");
        util::add_nodes::<PostProcessingNode>(render_app, "PostProcessing2d", "PostProcessing3d");

        render_app
            .init_resource::<DrawFunctions<PostProcessingPhaseItem>>()
            .init_resource::<PostProcessingBindGroups>()
            .init_resource::<PostProcessingSharedLayout>()
            .init_resource::<PostProcessingLayout<Raindrops>>()
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<Raindrops>>>()
            .add_system_to_stage(RenderStage::Extract, extract_post_processing_camera_phases)
            .add_system_to_stage(RenderStage::Queue, queue_post_processing_shared_bind_group)
            .add_system_to_stage(
                RenderStage::Queue,
                queue_post_processing_phase_items::<Raindrops, RaindropsMarker>,
            )
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<PostProcessingPhaseItem>,
            );
    }
}

fn extract_post_processing_camera_phases(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera), With<RaindropsMarker>>>,
) {
    for (entity, camera) in &cameras {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(RenderPhase::<PostProcessingPhaseItem>::default());
        }
    }
}

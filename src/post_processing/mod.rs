use std::{marker::PhantomData, sync::Mutex};

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::DynamicUniformIndex,
        globals::{GlobalsBuffer, GlobalsUniform},
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{
            sort_phase_system, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions,
            PhaseItem, RenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline,
            TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, CachedRenderPipelineId, FilterMode, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
            ShaderDefVal, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
            TextureViewDimension, TextureViewId,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        Extract, RenderApp, RenderStage,
    },
    utils::{FloatOrd, HashMap},
};

// use super::util;

/// Blur
pub mod blur;

/// Chromatic Aberration
pub mod chromatic_aberration;

/// Flip
pub mod flip;

/// LUT
pub mod lut;

/// Masks
pub mod masks;

/// Pixelate
pub mod pixelate;

/// Raindrops
pub mod raindrops;

/// Wave
pub mod wave;

#[derive(Resource)]
pub(crate) struct UniformBindGroup<U: ShaderType> {
    pub inner: Option<BindGroup>,
    marker: PhantomData<U>,
}

impl<U> Default for UniformBindGroup<U>
where
    U: ShaderType,
{
    fn default() -> Self {
        Self {
            inner: None,
            marker: PhantomData,
        }
    }
}

/// Adds a `.order` helper method to a component.
/// When used on a post processing effect, it determines the order in which the effect is applied.
/// See [`VfxOrdering`] for more information.
pub trait PostProcessingOrder: Sized {
    /// Adds an ordering to the component.
    fn with_order(self, order: f32) -> (Self, VfxOrdering<Self>);
}

impl<U> PostProcessingOrder for U
where
    U: Component,
{
    fn with_order(self, order: f32) -> (Self, VfxOrdering<Self>) {
        (self, VfxOrdering::new(order))
    }
}

/// TODO
struct SetEffectBindGroup<U: Component + ShaderType, const I: usize>(PhantomData<U>);
impl<P: PhaseItem, U: Component + ShaderType, const I: usize> RenderCommand<P>
    for SetEffectBindGroup<U, I>
{
    type Param = SRes<UniformBindGroup<U>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<DynamicUniformIndex<U>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        uniform_index: ROQueryItem<'w, Self::ItemWorldQuery>,
        uniform_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = uniform_bind_group.into_inner().inner.as_ref() {
            pass.set_bind_group(I, bind_group, &[uniform_index.index()]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

type DrawPostProcessingEffect<U> = (
    // The pipeline must be set in order to use the correct bind group,
    // access the correct shaders, and so on.
    SetItemPipeline,
    // Common to post processing items is that they all use the same
    // first bind group, which has the input texture (the scene) and
    // the sampler for that.
    SetTextureSamplerGlobals<0>,
    // Here we set the bind group for the effect.
    // This is the second bind group, which for all effects has a uniform (at some offset), and optionally
    // more bind group entries.
    SetEffectBindGroup<U, 1>,
    // Lastly we draw vertices.
    // This is simple for a post processing effect, since we just draw
    // a full screen triangle.
    DrawPostProcessing,
);

pub(crate) fn create_layout(
    world: &mut World,
    label: &str,
    layout_entries: &[BindGroupLayoutEntry],
) -> BindGroupLayout {
    let render_device = world.resource::<RenderDevice>();

    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some(&format!("{label} Uniform Bind Group Layout")),
        entries: layout_entries,
    })
}

pub(crate) fn render_pipeline_descriptor(
    label: &str,
    shared_layout: &BindGroupLayout,
    uniform_layout: &BindGroupLayout,
    shader: Handle<Shader>,
    shader_defs: Vec<ShaderDefVal>,
) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: Some(format!("{label} Render Pipeline").into()),
        layout: Some(vec![shared_layout.clone(), uniform_layout.clone()]),
        vertex: fullscreen_shader_vertex_state(),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            shader,
            shader_defs,
            entry_point: "fragment".into(),
            targets: vec![Some(TextureFormat::bevy_default().into())],
        }),
    }
}

pub(crate) fn create_pipeline(
    world: &mut World,
    label: &str,
    uniform_layout: &BindGroupLayout,
    shader: Handle<Shader>,
    shader_definitions: Vec<ShaderDefVal>,
) -> CachedRenderPipelineId {
    let shared_layout = &world.resource::<PostProcessingSharedLayout>().shared_layout;

    let pipeline_cache = world.resource::<PipelineCache>();

    pipeline_cache.queue_render_pipeline(render_pipeline_descriptor(
        label,
        shared_layout,
        uniform_layout,
        shader,
        shader_definitions,
    ))
}

pub(crate) fn create_layout_and_pipeline(
    world: &mut World,
    label: &str,
    layout_entries: &[BindGroupLayoutEntry],
    shader: Handle<Shader>,
) -> (BindGroupLayout, CachedRenderPipelineId) {
    let uniform_layout = create_layout(world, label, layout_entries);
    let pipeline_id = create_pipeline(world, label, &uniform_layout, shader, vec![]);

    (uniform_layout, pipeline_id)
}

/// Bind groups.
#[derive(Resource, Default, Debug)]
pub struct PostProcessingSharedBindGroups {
    cached_texture_bind_groups: HashMap<TextureViewId, BindGroup>,
    current_source_texture: Mutex<Option<TextureViewId>>,
}

/// Render command which sets the shared bind group containing the source texture and sampler as well as the globals.
pub struct SetTextureSamplerGlobals<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetTextureSamplerGlobals<I> {
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();
    type Param = SRes<PostProcessingSharedBindGroups>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: (),
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let id = {
            let lock = bind_groups
                .current_source_texture
                .try_lock()
                .expect("Mutex should be available");
            *lock.as_ref().expect("Source view id should be set")
        };

        let bind_groups = bind_groups.into_inner();

        if let Some(bind_group) = bind_groups.cached_texture_bind_groups.get(&id) {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            info!("No bind group for texture view id: {id:?} on {bind_groups:?}");
            RenderCommandResult::Failure
        }
    }
}

/// Render command for drawing the full screen triangle.
pub struct DrawPostProcessing;

impl<P: PhaseItem> RenderCommand<P> for DrawPostProcessing {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: (),
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Draw the full screen triangle.
        pass.draw(0..3, 0..1);
        RenderCommandResult::Success
    }
}

#[derive(Debug, Component)]
struct PostProcessingCamera;

#[allow(clippy::type_complexity)]
fn queue_post_processing_shared_bind_groups(
    render_device: Res<RenderDevice>,
    globals: Res<GlobalsBuffer>,
    layout: Res<PostProcessingSharedLayout>,
    mut bind_groups: ResMut<PostProcessingSharedBindGroups>,

    views: Query<(Entity, &ViewTarget), With<PostProcessingCamera>>,
) {
    for (_, view_target) in &views {
        for texture_view in [view_target.main_texture(), view_target.main_texture_other()] {
            let id = &texture_view.id();
            if !bind_groups.cached_texture_bind_groups.contains_key(id) {
                bind_groups.cached_texture_bind_groups.insert(
                    *id,
                    render_device.create_bind_group(&BindGroupDescriptor {
                        label: Some("PostProcessing texture bind group"),
                        layout: &layout.shared_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(texture_view),
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
}

/// A post processing phase item.
/// Contains a draw function which is specialized for a specific material.
/// Points to a matching pipeline- it will for example point to a specific fragment shader as well as
/// having a bind group specialized for the material.
pub struct PostProcessingPhaseItem {
    entity: Entity,
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

    fn entity(&self) -> Entity {
        self.entity
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
    pub(crate) shared_layout: BindGroupLayout,
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

        Self {
            shared_layout: textures_layout,
        }
    }
}

/// This system will add a default post processing phase to all active cameras, given that this camera
/// has the given component `C` in the render world.
///
/// A `VfxOrdering<C>` component can be added to the camera to control the ordering of the effect.
/// Else a default is inserted.
///
/// A `PostProcessingCamera` component is added in order to identify cameras that have any effect applied.
pub(crate) fn extract_post_processing_camera_phases<C: Component>(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera, Option<&VfxOrdering<C>>), With<C>>>,
) {
    for (entity, camera, maybe_ordering) in &cameras {
        if camera.is_active {
            let ordering = if let Some(o) = maybe_ordering {
                o.clone()
            } else {
                VfxOrdering::new(0.0)
            };

            commands.get_or_spawn(entity).insert((
                RenderPhase::<PostProcessingPhaseItem>::default(),
                ordering,
                PostProcessingCamera,
            ));
        }
    }
}

/// The post processing node.
pub struct PostProcessingNode {
    query: QueryState<
        (
            &'static ExtractedCamera,
            &'static ViewTarget,
            &'static RenderPhase<PostProcessingPhaseItem>,
        ),
        With<ExtractedView>,
    >,
}

impl PostProcessingNode {
    /// The slot input name.
    pub const IN_VIEW: &'static str = "view";

    /// Create a a new post processing node.
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
        let shared_bind_groups = world.resource::<PostProcessingSharedBindGroups>();
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;

        let (camera, view_target, phase) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        let draw_functions = world.resource::<DrawFunctions<PostProcessingPhaseItem>>();
        let mut draw_functions = draw_functions.write();
        draw_functions.prepare(world);

        for (_index, item) in phase.items.iter().enumerate() {
            let post_process = view_target.post_process_write();
            let source = post_process.source;
            let destination = post_process.destination;

            shared_bind_groups
                .current_source_texture
                .lock()
                .expect("Mutex should be unused")
                .replace(source.id());

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("PostProcessing pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: destination,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
            });

            if let Some(viewport) = camera.viewport.as_ref() {
                render_pass.set_camera_viewport(viewport);
            }

            draw_functions
                .get_mut(item.draw_function)
                .expect("Draw function should exist")
                .draw(world, &mut render_pass, view_entity, item);
        }

        Ok(())
    }
}

/// Decide on ordering for post processing effects.
/// Lower numbers means run earlier.
#[derive(Debug, Component, Copy)]
pub struct VfxOrdering<C> {
    /// Priority
    pub priority: f32,
    marker: PhantomData<C>,
}

impl<C> From<VfxOrdering<C>> for FloatOrd {
    fn from(ordering: VfxOrdering<C>) -> Self {
        Self(ordering.priority)
    }
}

impl<C> Clone for VfxOrdering<C> {
    fn clone(&self) -> Self {
        Self {
            priority: self.priority,
            marker: self.marker,
        }
    }
}

impl<C> VfxOrdering<C> {
    /// Create a new ordering.
    pub fn new(priority: f32) -> Self {
        Self {
            priority,
            marker: PhantomData,
        }
    }
}

/// TODO
pub struct PostProcessingPlugin {}

pub(crate) fn render_app(app: &mut App) -> &mut App {
    app.get_sub_app_mut(RenderApp)
        .expect("Need a render app for post processing")
}

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app
            .get_sub_app_mut(RenderApp)
            .expect("Need a render app for post processing");

        // All effects share this node.
        crate::util::add_nodes::<PostProcessingNode>(
            render_app,
            "PostProcessing2d",
            "PostProcessing3d",
        );

        render_app
            .init_resource::<DrawFunctions<PostProcessingPhaseItem>>()
            .init_resource::<PostProcessingSharedBindGroups>()
            .init_resource::<PostProcessingSharedLayout>()
            .add_system_to_stage(RenderStage::Queue, queue_post_processing_shared_bind_groups)
            .add_system_to_stage(RenderStage::Extract, extract_camera_phases)
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<PostProcessingPhaseItem>,
            );

        app.add_plugin(blur::Plugin);
        app.add_plugin(chromatic_aberration::Plugin);
        app.add_plugin(flip::Plugin);
        app.add_plugin(lut::Plugin);
        app.add_plugin(masks::Plugin);
        app.add_plugin(raindrops::Plugin);
        app.add_plugin(pixelate::Plugin);
        app.add_plugin(wave::Plugin);
    }
}

fn extract_camera_phases(mut commands: Commands, cameras: Extract<Query<(Entity, &Camera)>>) {
    for (entity, camera) in &cameras {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(RenderPhase::<PostProcessingPhaseItem>::default());
        }
    }
}

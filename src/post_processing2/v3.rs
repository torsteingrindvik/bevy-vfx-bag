use std::hash::Hash;
use std::{marker::PhantomData, sync::Mutex};

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        globals::{GlobalsBuffer, GlobalsUniform},
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{
            sort_phase_system, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions,
            EntityPhaseItem, EntityRenderCommand, PhaseItem, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites, FilterMode,
            FragmentState, LoadOp, MultisampleState, Operations, PrimitiveState, PrimitiveTopology,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            SamplerBindingType, SamplerDescriptor, ShaderDefVal, ShaderStages, ShaderType,
            SpecializedRenderPipeline, TextureSampleType, TextureViewDimension, TextureViewId,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
    sprite::{Material2d, Material2dKey, Material2dPipeline, SetMaterial2dBindGroup},
    utils::{FloatOrd, HashMap},
};

use super::util;

mod post_processing_plugin;

// The draw steps performed on a post processing phase item.
type DrawPostProcessingItem<M> = (
    // The pipeline must be set in order to use the correct bind group,
    // access the correct shaders, and so on.
    SetItemPipeline,
    // Common to post processing items is that they all use the same
    // first bind group, which has the input texture (the scene) and
    // the sampler for that.
    SetTextureAndSampler<0>,
    // The second bind group is specific to the post processing item.
    // It's typically used to pass in parameters to the shader.
    //
    // We don't have to define this bind group ourselves- the material derive
    // does that for us.
    // But we have to set it.
    SetMaterial2dBindGroup<M, 1>,
    // Lastly we draw vertices.
    // This is simple for a post processing effect, since we just draw
    // a full screen triangle.
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

        // Here we fetch the bind group associated with the given texture.
        // This should have been prepared earlier in the render schedule.
        // See [`queue_post_processing_shared_bind_group`].
        let bind_group = {
            match bind_groups.into_inner().cached_texture_bind_groups.get(&id) {
                Some(bg) => bg,
                None => panic!("Bind group for texture {id:?} should be available"),
            }
        };

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

// TODO: Should we just create these directly when the user asks for it in the node run?
// This way we don't have to have access to "main_texture_other"
#[allow(clippy::type_complexity)]
fn queue_post_processing_shared_bind_group(
    render_device: Res<RenderDevice>,
    globals: Res<GlobalsBuffer>,
    layout: Res<PostProcessingSharedLayout>,
    mut bind_groups: ResMut<PostProcessingBindGroups>,

    views: Query<
        (Entity, &ViewTarget),
        AnyOf<(
            &raindrops::RaindropsSettings,
            &pixelate::PixelateSettings,
            &flip::FlipSettings,
            &masks::MaskSettings,
        )>,
    >,
) {
    for (_, view_target) in &views {
        for texture_view in [view_target.main_texture(), view_target.main_texture_other()] {
            let id = &texture_view.id();
            if !bind_groups.cached_texture_bind_groups.contains_key(id) {
                info!("Inserting bind group for id: {id:?}");
                bind_groups.cached_texture_bind_groups.insert(
                    *id,
                    render_device.create_bind_group(&BindGroupDescriptor {
                        label: Some("PostProcessing texture bind group"),
                        layout: &layout.textures_layout,
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

// Needed for this to be a render command.
impl EntityPhaseItem for PostProcessingPhaseItem {
    fn entity(&self) -> Entity {
        self.entity
    }
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
    fragment_shader: Handle<Shader>,
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
        let material_pipeline = world
            .get_resource::<Material2dPipeline<M>>()
            .expect("Resource should be available");

        Self {
            shared: shared.clone(),
            material_layout: material_pipeline.material2d_layout.clone(),
            fragment_shader: material_pipeline
                .fragment_shader
                .as_ref()
                .expect("Should have fragment shader set")
                .clone(),
            marker: PhantomData,
        }
    }
}

impl<M: Material2d> SpecializedRenderPipeline for PostProcessingLayout<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = Material2dKey<M>;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut descriptor = RenderPipelineDescriptor {
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
                shader: self.fragment_shader.clone(),
                // TODO: Get rid of this when Bevy supports it: https://github.com/bevyengine/bevy/issues/6799
                shader_defs: vec![ShaderDefVal::Int("MAX_DIRECTIONAL_LIGHTS".to_string(), 1)],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: bevy::render::render_resource::TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
        };

        let fake_mesh = Mesh::new(PrimitiveTopology::PointList);

        M::specialize(
            &mut descriptor,
            &fake_mesh.get_mesh_vertex_buffer_layout(),
            key,
        )
        .expect("Specialize ok");

        descriptor
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

        let mut has_cleared = false;

        for item in &phase.items {
            // for _ in 0..=1 {
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
                    ops: if has_cleared {
                        Operations {
                            load: LoadOp::Load,
                            store: true,
                        }
                    } else {
                        has_cleared = true;
                        Operations {
                            load: LoadOp::Clear(Default::default()),
                            store: true,
                        }
                    },
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
            // }
        }

        Ok(())
    }
}

/// Decide on ordering for post processing effects.
/// TODO: Describe if higher values or lower values means first.
#[derive(Debug, Component, Copy)]
pub struct VfxOrdering<C> {
    /// Priority
    pub priority: f32,
    marker: PhantomData<C>,
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
    /// Create a new ordering. TODO
    pub fn new(priority: f32) -> Self {
        Self {
            priority,
            marker: PhantomData,
        }
    }
}

/// TODO
pub struct PostProcessingPlugin {}

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        // app.add_system(pixelate::pixelate_add_material);
        // .add_system(flip::flip_add_material);
        // .add_system(masks::masks_add_material);

        let render_app = app.get_sub_app_mut(RenderApp).expect("Should work");

        // All effects share this node.
        util::add_nodes::<PostProcessingNode>(render_app, "PostProcessing2d", "PostProcessing3d");

        render_app
            .init_resource::<DrawFunctions<PostProcessingPhaseItem>>()
            .init_resource::<PostProcessingBindGroups>()
            .init_resource::<PostProcessingSharedLayout>()
            .add_system_to_stage(RenderStage::Queue, queue_post_processing_shared_bind_group)
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<PostProcessingPhaseItem>,
            );

        app.add_plugin(raindrops::Plugin);
        app.add_plugin(lut::Plugin);
        app.add_plugin(masks::Plugin);
        app.add_plugin(flip::Plugin);
        app.add_plugin(pixelate::Plugin);
    }
}

////////////////////////////////////////////////////////////////////////////////
// RAINDROPS
////////////////////////////////////////////////////////////////////////////////

/// Raindrops
pub mod raindrops;

////////////////////////////////////////////////////////////////////////////////
// PIXELATE
////////////////////////////////////////////////////////////////////////////////

/// Pixelate
pub mod pixelate;

////////////////////////////////////////////////////////////////////////////////
// FLIP
////////////////////////////////////////////////////////////////////////////////

/// Flip
pub mod flip;

////////////////////////////////////////////////////////////////////////////////
// MASKS
////////////////////////////////////////////////////////////////////////////////

/// Masks
pub mod masks;

////////////////////////////////////////////////////////////////////////////////
// LUT
////////////////////////////////////////////////////////////////////////////////

/// LUT todo
pub mod lut;

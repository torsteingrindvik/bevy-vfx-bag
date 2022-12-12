use std::hash::Hash;
use std::{marker::PhantomData, sync::Mutex};

use bevy::{
    asset::LoadState,
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
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{
            sort_phase_system, AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId,
            DrawFunctions, EntityPhaseItem, EntityRenderCommand, PhaseItem, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            AddressMode, AsBindGroup, BindGroup, BindGroupDescriptor, BindGroupEntry,
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
            BindingType, BufferBindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            Extent3d, FilterMode, FragmentState, LoadOp, MultisampleState, Operations,
            PipelineCache, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
            ShaderDefVal, ShaderRef, ShaderStages, ShaderType, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureDimension, TextureFormat, TextureSampleType,
            TextureViewDescriptor, TextureViewDimension, TextureViewId,
        },
        renderer::{RenderContext, RenderDevice},
        texture::{BevyDefault, ImageSampler},
        view::{ExtractedView, ViewTarget},
        Extract, RenderApp, RenderStage,
    },
    sprite::{
        Material2d, Material2dKey, Material2dPipeline, Material2dPlugin, Mesh2dPipelineKey,
        RenderMaterials2d, SetMaterial2dBindGroup,
    },
    utils::{FloatOrd, HashMap},
};

use crate::{load_image, load_lut, shader_ref};

use super::util;

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
            &RaindropsSettings,
            &PixelateSettings,
            &FlipSettings,
            &MaskSettings,
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
                // TODO: Get rid of this when Bevy supports it.
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

/// Adds the phase items. Each time one is added it means that a render pass for the effect will be performed.
#[allow(clippy::complexity)]
pub fn queue_post_processing_phase_items<M: Material2d, C: Component>(
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
    render_materials: Res<RenderMaterials2d<M>>,
    // material2d_handles: Query<(Entity, &Handle<M>)>,
    pipeline: Res<PostProcessingLayout<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PostProcessingLayout<M>>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut views: Query<
        (
            Entity,
            &mut RenderPhase<PostProcessingPhaseItem>,
            // Option<&VfxOrdering>,
            &VfxOrdering<C>,
            &Handle<M>,
        ),
        With<C>,
    >,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    // if material2d_handles.is_empty() {
    //     return;
    // }

    for (entity, mut phase, ordering, material_handle) in views.iter_mut() {
        debug!(
            "Adding post processing phase items: {:?}+{:?}",
            std::any::type_name::<M>(),
            std::any::type_name::<C>()
        );
        if let Some(material2d) = render_materials.get(material_handle) {
            let pipeline_id = pipelines.specialize(
                &mut pipeline_cache,
                &pipeline,
                Material2dKey {
                    mesh_key: Mesh2dPipelineKey::NONE,
                    bind_group_data: material2d.key.clone(),
                },
            );

            let draw_function = draw_functions.read().id::<DrawPostProcessingItem<M>>();
            // let draw_function = draw_functions.read().id::<DrawPostProcessingItem>();
            debug!("Draw function found, adding phase item: {entity:?}");

            phase.add(PostProcessingPhaseItem {
                sort_key: FloatOrd(ordering.priority),
                draw_function,
                pipeline_id,
                entity,
            })
        }
    }
}

/// TODO
pub struct PostProcessingPlugin {}

#[derive(Default, Resource)]
struct Handles {
    raindrops_texture: Handle<Image>,
    fallback_lut: Handle<Image>,
    // luts: HashMap<LutVariant, LutImage>,
}

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<Raindrops>::default())
            .add_plugin(Material2dPlugin::<Pixelate>::default())
            .add_plugin(Material2dPlugin::<Flip>::default())
            .add_plugin(Material2dPlugin::<Mask>::default())
            .add_plugin(Material2dPlugin::<Lut>::default())
            .add_plugin(ExtractComponentPlugin::<RaindropsSettings>::default())
            .add_plugin(ExtractComponentPlugin::<PixelateSettings>::default())
            .add_plugin(ExtractComponentPlugin::<FlipSettings>::default())
            .add_plugin(ExtractComponentPlugin::<MaskSettings>::default())
            .add_plugin(ExtractComponentPlugin::<LutSettings>::default());

        let handles = Handles {
            raindrops_texture: load_image!(app, "textures/raindrops.tga", "tga"),
            fallback_lut: load_lut!(app, "luts/sauna.png", "png"),
            // luts: HashMap::from_iter(vec![
            // (LutVariant::Arctic, load_lut!(app, "luts/arctic.png", "png")),
            // (
            //     LutVariant::Burlesque,
            //     load_lut!(app, "luts/burlesque.png", "png"),
            // ),
            // (LutVariant::Denim, load_lut!(app, "luts/denim.png", "png")),
            // (LutVariant::Neo, load_lut!(app, "luts/neo.png", "png")),
            // (
            //     LutVariant::Neutral,
            //     load_lut!(app, "luts/neutral.png", "png"),
            // ),
            // (LutVariant::Rouge, load_lut!(app, "luts/rouge.png", "png")),
            // (LutVariant::Sauna, load_lut!(app, "luts/sauna.png", "png")),
            // (LutVariant::Slate, load_lut!(app, "luts/slate.png", "png")),
            // ]),
        };

        app.insert_resource(handles)
            .add_system(fixup_assets)
            .add_system(raindrops_add_material)
            .add_system(pixelate_add_material)
            .add_system(flip_add_material)
            .add_system(masks_add_material)
            .add_system(lut_add_material);

        let render_app = app.get_sub_app_mut(RenderApp).expect("Should work");
        util::add_nodes::<PostProcessingNode>(render_app, "PostProcessing2d", "PostProcessing3d");

        render_app
            .init_resource::<DrawFunctions<PostProcessingPhaseItem>>()
            .init_resource::<PostProcessingBindGroups>()
            .init_resource::<PostProcessingSharedLayout>()
            .init_resource::<PostProcessingLayout<Raindrops>>()
            .init_resource::<PostProcessingLayout<Pixelate>>()
            .init_resource::<PostProcessingLayout<Flip>>()
            .init_resource::<PostProcessingLayout<Mask>>()
            .init_resource::<PostProcessingLayout<Lut>>()
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<Raindrops>>>()
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<Pixelate>>>()
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<Flip>>>()
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<Mask>>>()
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<Lut>>>()
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingItem<Raindrops>>()
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingItem<Pixelate>>()
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingItem<Flip>>()
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingItem<Mask>>()
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingItem<Lut>>()
            .add_system_to_stage(
                RenderStage::Extract,
                extract_post_processing_camera_phases::<RaindropsSettings>,
            )
            .add_system_to_stage(
                RenderStage::Extract,
                extract_post_processing_camera_phases::<PixelateSettings>,
            )
            .add_system_to_stage(
                RenderStage::Extract,
                extract_post_processing_camera_phases::<FlipSettings>,
            )
            .add_system_to_stage(
                RenderStage::Extract,
                extract_post_processing_camera_phases::<MaskSettings>,
            )
            .add_system_to_stage(
                RenderStage::Extract,
                extract_post_processing_camera_phases::<LutSettings>,
            )
            .add_system_to_stage(RenderStage::Queue, queue_post_processing_shared_bind_group)
            .add_system_to_stage(
                RenderStage::Queue,
                queue_post_processing_phase_items::<Raindrops, RaindropsSettings>,
            )
            .add_system_to_stage(
                RenderStage::Queue,
                queue_post_processing_phase_items::<Pixelate, PixelateSettings>,
            )
            .add_system_to_stage(
                RenderStage::Queue,
                queue_post_processing_phase_items::<Flip, FlipSettings>,
            )
            .add_system_to_stage(
                RenderStage::Queue,
                queue_post_processing_phase_items::<Mask, MaskSettings>,
            )
            .add_system_to_stage(
                RenderStage::Queue,
                queue_post_processing_phase_items::<Lut, LutSettings>,
            )
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<PostProcessingPhaseItem>,
            );
    }
}

// const RAINDROPS_SHADER_HANDLE: HandleUntyped =
//     HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 14785543643812289755);

#[allow(clippy::type_complexity)]
fn extract_post_processing_camera_phases<C: Component>(
    mut commands: Commands,
    cameras: Extract<
        Query<
            (Entity, &Camera, Option<&VfxOrdering<C>>),
            AnyOf<(
                &RaindropsSettings,
                &PixelateSettings,
                &FlipSettings,
                &MaskSettings,
                &LutSettings,
            )>,
        >,
    >,
) {
    for (entity, camera, maybe_ordering) in &cameras {
        if camera.is_active {
            let ordering = if let Some(o) = maybe_ordering {
                o.clone()
            } else {
                VfxOrdering::new(0.0)
            };

            commands
                .get_or_spawn(entity)
                .insert((RenderPhase::<PostProcessingPhaseItem>::default(), ordering));
        }
    }
}

fn fixup_raindrops(
    handle: &Handle<Image>,
    assets: &mut Assets<Image>,
    materials: &mut Assets<Raindrops>,
) {
    info!("Handle was raindrops texture");

    let image = assets
        .get_mut(handle)
        .expect("Handle should point to asset");

    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        label: Some("Repeat Sampler"),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        ..default()
    });

    for (_, _material) in materials.iter_mut() {
        // This mutable "access" is needed to trigger the usage of the new sampler.
        info!("Material is pointing to: {:?}", _material.color_texture);
    }
}

// fn fixup_luts(handle: &Handle<Image>, assets: &mut Assets<Image>, materials: &mut Assets<Lut>) {
//     info!("Handle was LUT texture");

//     let image = assets
//         .get_mut(handle)
//         .expect("Handle should point to asset");

//     // The LUT is a 3d texture. It has 64 layers, each of which is a 64x64 image.
//     image.texture_descriptor.size = Extent3d {
//         width: 64,
//         height: 64,
//         depth_or_array_layers: 64,
//     };
//     image.texture_descriptor.dimension = TextureDimension::D3;
//     image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

//     image.texture_view_descriptor = Some(TextureViewDescriptor {
//         label: Some("LUT TextureViewDescriptor"),
//         format: Some(image.texture_descriptor.format),
//         dimension: Some(TextureViewDimension::D3),
//         ..default()
//     });

//     // The default sampler may change depending on the image plugin setup,
//     // so be explicit here.
//     image.sampler_descriptor = ImageSampler::linear();

//     // dbg!(&image.sampler_descriptor);
//     // dbg!(&image.texture_descriptor);
//     // dbg!(&image.texture_view_descriptor);

//     for (_, _material) in materials.iter_mut() {
//         // This mutable "access" is needed to trigger the usage of the new sampler.
//         info!("Material is pointing to: {:?}", _material.lut);
//     }
// }

fn fixup_assets(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    handles: Res<Handles>,
    mut raindrop_materials: ResMut<Assets<Raindrops>>,
    mut lut_materials: ResMut<Assets<Lut>>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            info!("Handle to asset created: {:?}", handle);

            if *handle == handles.raindrops_texture {
                fixup_raindrops(handle, &mut assets, &mut raindrop_materials);
            }
            // else if handles
            //     .luts
            //     .values()
            //     .any(|lut_handle| lut_handle.0 == handle)
            // {
            //     fixup_luts(handle, &mut assets, &mut lut_materials);
            // }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// RAINDROPS
////////////////////////////////////////////////////////////////////////////////

// #[cfg(not(feature = "dev"))]
const RAINDROPS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10304902298789658536);

// #[cfg(not(feature = "dev"))]
// const RAINDROPS_TEXTURE_HANDLE: HandleUntyped =
//     HandleUntyped::weak_from_u64(Image::TYPE_UUID, 9363411587132811616);

#[allow(clippy::type_complexity)]
fn raindrops_add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Raindrops>>,
    handles: Res<Handles>,
    cameras: Query<(Entity, &RaindropsSettings), (With<Camera>, Without<Handle<Raindrops>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Raindrops {
            color_texture: handles.raindrops_texture.clone(),
            raindrops: RaindropsUniform {
                time_scaling: settings.time_scaling,
                intensity: settings.intensity,
                zoom: settings.zoom,
            },
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
struct RaindropsUniform {
    time_scaling: f32,
    intensity: f32,
    zoom: f32,
}

impl Default for RaindropsUniform {
    fn default() -> Self {
        Self {
            time_scaling: 0.8,
            intensity: 0.03,
            zoom: 1.0,
        }
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "4fba30ae-73e6-11ed-8575-9b008b9044f0"]
pub struct Raindrops {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Handle<Image>,

    #[uniform(2)]
    raindrops: RaindropsUniform,
}

impl Material2d for Raindrops {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(RAINDROPS_SHADER_HANDLE, "shaders/raindrops3.wgsl")

        // "shaders/raindrops3.wgsl".into()
    }
}

/// TODO
#[derive(Debug, Component, Clone, Copy)]
pub struct RaindropsSettings {
    time_scaling: f32,
    intensity: f32,
    zoom: f32,
}

impl Default for RaindropsSettings {
    fn default() -> Self {
        Self {
            time_scaling: 0.8,
            intensity: 0.03,
            zoom: 1.0,
        }
    }
}

impl ExtractComponent for RaindropsSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

////////////////////////////////////////////////////////////////////////////////
// PIXELATE
////////////////////////////////////////////////////////////////////////////////

const PIXELATE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11093977931118718560);

#[allow(clippy::type_complexity)]
fn pixelate_add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Pixelate>>,
    cameras: Query<(Entity, &PixelateSettings), (With<Camera>, Without<Handle<Pixelate>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Pixelate {
            pixelate: PixelateUniform {
                block_size: settings.block_size,
            },
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
struct PixelateUniform {
    block_size: f32,
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "485141dc-7890-11ed-9cf4-ab2aa4ee03b0"]
pub struct Pixelate {
    #[uniform(0)]
    pixelate: PixelateUniform,
}

impl Material2d for Pixelate {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(PIXELATE_SHADER_HANDLE, "shaders/pixelate3.wgsl")
    }
}

/// TODO
#[derive(Debug, Component, Clone, Copy)]
pub struct PixelateSettings {
    block_size: f32,
}

impl Default for PixelateSettings {
    fn default() -> Self {
        Self { block_size: 8.0 }
    }
}

impl ExtractComponent for PixelateSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FLIP
////////////////////////////////////////////////////////////////////////////////

const FLIP_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1649866799156783187);

#[allow(clippy::type_complexity)]
fn flip_add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Flip>>,
    cameras: Query<(Entity, &FlipSettings), (With<Camera>, Without<Handle<Flip>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Flip {
            flip: FlipUniform::from(*settings),
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
struct FlipUniform {
    x: f32,
    y: f32,
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

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "cbb6349e-7a00-11ed-8a99-63067e99f73e"]
pub struct Flip {
    #[uniform(0)]
    flip: FlipUniform,
}

impl Material2d for Flip {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(FLIP_SHADER_HANDLE, "shaders/flip3.wgsl")
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
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

////////////////////////////////////////////////////////////////////////////////
// MASKS
////////////////////////////////////////////////////////////////////////////////

const MASK_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1649866799156783187);

#[allow(clippy::type_complexity)]
fn masks_add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Mask>>,
    cameras: Query<(Entity, &MaskSettings), (With<Camera>, Without<Handle<Mask>>)>,
) {
    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Mask {
            mask: MaskUniform::from(*settings),
            variant: settings.variant,
        });
        commands.entity(entity).insert(material_handle);
    }
}

#[derive(Debug, ShaderType, Clone)]
struct MaskUniform {
    strength: f32,
}

impl From<MaskSettings> for MaskUniform {
    fn from(mask: MaskSettings) -> Self {
        Self {
            strength: mask.strength,
        }
    }
}

/// TODO
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MaskKey(MaskVariant);

impl From<&Mask> for MaskKey {
    fn from(material: &Mask) -> Self {
        Self(material.variant)
    }
}

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "1f0d1510-7a04-11ed-bf61-b3427e523ca2"]
#[bind_group_data(MaskKey)]
pub struct Mask {
    #[uniform(0)]
    mask: MaskUniform,

    variant: MaskVariant,
}

impl Material2d for Mask {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(MASK_SHADER_HANDLE, "shaders/masks3.wgsl")
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let def = match key.bind_group_data.0 {
            MaskVariant::Square => "SQUARE",
            MaskVariant::Crt => "CRT",
            MaskVariant::Vignette => "VIGNETTE",
        };
        descriptor
            .fragment
            .as_mut()
            .expect("Should have fragment state")
            .shader_defs
            .push(def.into());
        info!("Specializing mask shader with {:?}", def);

        Ok(())
    }
}

/// This controls the parameters of the effect.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum MaskVariant {
    /// Rounded square type mask.
    ///
    /// One use of this mask is to post-process _other_ effects which might
    /// have artifacts around the edges.
    /// This mask can then attenuate that effect and thus remove the effects of the
    /// artifacts.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    3.0 almost loses the square shape.
    /// High end:   100.0 has almost sharp, thin edges.
    Square,

    /// Rounded square type mask, but more oval like a CRT television.
    ///
    /// This effect can be used as a part of a retry-style effect.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    3000.0 almost loses the CRT shape.
    /// High end:   500000.0 "inflates" the effect a bit.
    Crt,

    /// Vignette mask.
    ///
    /// This effect can be used to replicate the classic photography
    /// light attenuation seen at the edges of photos.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    0.10 gives a very subtle effect.
    /// High end:   1.50 is almost a spotlight in the middle of the screen.
    Vignette,
}

/// TODO
#[derive(Debug, Copy, Clone, Component)]
pub struct MaskSettings {
    /// The strength parameter of the mask in use.
    ///
    /// See [`MaskVariant`] for guidelines on which range of values make sense
    /// for the variant in use.
    ///
    /// Run the masks example to experiment with these values interactively.
    pub strength: f32,

    /// Which [`MaskVariant`] to produce.
    pub variant: MaskVariant,
}

impl MaskSettings {
    /// Create a new square mask with a reasonable strength value.
    pub fn new_square() -> Self {
        Self {
            strength: 20.,
            variant: MaskVariant::Square,
        }
    }

    /// Create a new CRT mask with a reasonable strength value.
    pub fn new_crt() -> Self {
        Self {
            strength: 80000.,
            variant: MaskVariant::Crt,
        }
    }

    /// Create a new vignette mask with a reasonable strength value.
    pub fn new_vignette() -> Self {
        Self {
            strength: 0.66,
            variant: MaskVariant::Vignette,
        }
    }
}

impl Default for MaskSettings {
    fn default() -> Self {
        Self::new_vignette()
    }
}

impl ExtractComponent for MaskSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

////////////////////////////////////////////////////////////////////////////////
// LUT
////////////////////////////////////////////////////////////////////////////////

const LUT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10304902298789658536);

#[allow(clippy::type_complexity)]
fn lut_add_material(
    mut commands: Commands,
    mut assets: ResMut<Assets<Lut>>,
    // asset_server: Res<AssetServer>,
    handles: Res<Handles>,
    cameras: Query<(Entity, &LutSettings), (With<Camera>, Without<Handle<Lut>>)>,
) {
    // for (entity, settings) in cameras.iter() {
    //     let lut_handle = handles
    //         .luts
    //         .get(&settings.variant)
    //         .cloned()
    //         .expect("LUT handle should be valid");
    //     if matches!(asset_server.get_load_state(&lut_handle), LoadState::Loaded) {
    //         info!("Loaded! Let's go: {lut_handle:?}, {settings:?}");
    //         let material_handle = assets.add(Lut {
    //             lut: LutImage(lut_handle),
    //         });
    //         commands.entity(entity).insert(material_handle);
    //     }
    // }

    for (entity, settings) in cameras.iter() {
        let material_handle = assets.add(Lut {
            // lut: handles
            //     .luts
            //     .get(&settings.variant)
            //     .cloned()
            //     .expect("LUT handle should be valid"),
            lut: handles.fallback_lut.clone(),
            // lut: handles

            //     .luts
            //     .get(&settings.variant)
            //     .cloned()
            //     .expect("LUT handle should be valid"),
        });
        commands.entity(entity).insert(material_handle);
    }
}

// TODO
// #[derive(Debug, Clone)]
// pub struct LutImage(Handle<Image>);

// impl<'a> From<&'a LutImage> for Option<&'a Handle<Image>> {
//     fn from(lut: &'a LutImage) -> Self {
//         // info!("Doing the thing from etc: {lut:?}, hopefully the image is ready now");
//         Some(&lut.0)
//         // None
//     }
// }

/// TODO
#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "de05b53e-7a14-11ed-89c7-e315bbf02c20"]
pub struct Lut {
    #[texture(7, dimension = "3d")]
    #[sampler(8)]
    // lut: LutImage,
    lut: Handle<Image>,
}

impl Material2d for Lut {
    fn fragment_shader() -> ShaderRef {
        shader_ref!(LUT_SHADER_HANDLE, "shaders/lut3.wgsl")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LutVariant {
    Arctic,
    Burlesque,
    Denim,
    Neo,
    Neutral,
    Rouge,
    Sauna,
    Slate,
}

/// TODO
#[derive(Debug, Clone, Component)]
pub struct LutSettings {
    variant: LutVariant,
}

impl Default for LutSettings {
    fn default() -> Self {
        Self::new_arctic()
    }
}

impl LutSettings {
    fn new_variant(variant: LutVariant) -> Self {
        Self { variant }
    }

    /// TODO
    pub fn new_arctic() -> Self {
        Self::new_variant(LutVariant::Arctic)
    }

    /// TODO
    pub fn new_neutral() -> Self {
        Self::new_variant(LutVariant::Neutral)
    }
}

impl ExtractComponent for LutSettings {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
}

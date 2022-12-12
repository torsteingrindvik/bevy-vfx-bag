use std::{marker::PhantomData, num::NonZeroU64};

use bevy::{
    core_pipeline::{core_2d, core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state},
    ecs::component::ComponentDescriptor,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        globals::{GlobalsBuffer, GlobalsUniform},
        render_graph::{self, RenderGraph, SlotInfo, SlotType},
        render_phase::TrackedRenderPass,
        render_resource::{
            encase::private::WriteInto, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites, FilterMode,
            FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            SamplerBindingType, SamplerDescriptor, ShaderDefVal, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
};

// pub(crate) trait BindGroupLayoutBuilder {
//     fn extra_bind_group_layout() -> Option<BindGroupLayoutDescriptor<'static>>;

//     fn extra_bind_group_layout() -> Option<BindGroupLayoutDescriptor<'static>>;
// }

pub fn render_pipeline_descriptor(
    label: &'static str,
    bind_group_layout: BindGroupLayout,
    shader_handle: Handle<Shader>,
    texture_format: TextureFormat,
) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: Some(label.into()),
        layout: Some(vec![bind_group_layout]),
        vertex: fullscreen_shader_vertex_state(),
        fragment: Some(FragmentState {
            shader: shader_handle,
            shader_defs: vec![ShaderDefVal::Int("MAX_DIRECTIONAL_LIGHTS".to_string(), 1)],
            entry_point: "fragment".into(),
            targets: vec![Some(ColorTargetState {
                format: texture_format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
    }
}

pub fn bind_group_layout(
    render_world: &mut World,
    label: &'static str,
    uniform_min_size: NonZeroU64,
) -> BindGroupLayout {
    render_world
        .resource::<RenderDevice>()
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
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
                        // min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(uniform_min_size),
                        // min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
}

pub fn add_nodes<T: FromWorld + render_graph::Node>(
    render_app: &mut App,
    name_2d: &str,
    name_3d: &str,
) {
    {
        let node = <T as FromWorld>::from_world(&mut render_app.world);
        let mut binding = render_app.world.resource_mut::<RenderGraph>();
        let graph = binding
            .get_sub_graph_mut(core_3d::graph::NAME)
            .expect("Graph should be available");

        graph.add_node(name_3d.to_owned(), node);

        graph.add_slot_edge(
            graph.input_node().id,
            core_3d::graph::input::VIEW_ENTITY,
            name_3d.to_owned(),
            "view",
        );

        graph.add_node_edge(core_3d::graph::node::MAIN_PASS, name_3d.to_owned());

        graph.add_node_edge(
            name_3d.to_owned(),
            core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        );
    }
    {
        let node = <T as FromWorld>::from_world(&mut render_app.world);
        let mut binding = render_app.world.resource_mut::<RenderGraph>();
        let graph = binding
            .get_sub_graph_mut(core_2d::graph::NAME)
            .expect("Graph should be available");

        graph.add_node(name_2d.to_owned(), node);

        graph.add_slot_edge(
            graph.input_node().id,
            core_2d::graph::input::VIEW_ENTITY,
            name_2d.to_owned(),
            "view",
        );

        graph.add_node_edge(
            name_2d.to_owned(),
            core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        );
    }
}

#[derive(Component)]
pub(crate) struct SpecializedPipelinesCache<T> {
    pub pipeline_id: CachedRenderPipelineId,
    marker: PhantomData<T>,
}

impl<T> SpecializedPipelinesCache<T> {
    fn new(pipeline_id: CachedRenderPipelineId) -> Self {
        Self {
            pipeline_id,
            marker: PhantomData,
        }
    }
}

pub(crate) struct PrepareSpecializedPipelinesPlugin<SP, U> {
    marker_sp: PhantomData<SP>,
    marker_u: PhantomData<U>,
}

impl<SP, U> Default for PrepareSpecializedPipelinesPlugin<SP, U> {
    fn default() -> Self {
        Self {
            marker_sp: Default::default(),
            marker_u: Default::default(),
        }
    }
}

impl<SP, U> Plugin for PrepareSpecializedPipelinesPlugin<SP, U>
where
    SP: SpecializedRenderPipeline<Key = TextureFormat> + Resource,
    SP::Key: Send + Sync,
    U: Component,
{
    fn build(&self, app: &mut App) {
        let render_app = app
            .get_sub_app_mut(RenderApp)
            .expect("Should get render app");

        render_app.add_system_to_stage(RenderStage::Prepare, prepare_specialize_pipelines::<SP, U>);
    }
}

fn prepare_specialize_pipelines<SP, U>(
    mut commands: Commands,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SP>>,
    pipeline: Res<SP>,
    views: Query<(Entity, &ExtractedView), With<U>>,
) where
    SP: SpecializedRenderPipeline<Key = TextureFormat> + Resource,
    SP::Key: Send + Sync,
    U: Component,
{
    for (entity, view) in &views {
        let pipeline_id = pipelines.specialize(
            &mut pipeline_cache,
            &pipeline,
            if view.hdr {
                ViewTarget::TEXTURE_FORMAT_HDR
            } else {
                TextureFormat::bevy_default()
            },
        );

        commands
            .entity(entity)
            .insert(SpecializedPipelinesCache::<SP>::new(pipeline_id));
    }
}

pub(crate) struct PostProcessingPlugin<Settings> {
    name: &'static str,
    shader_handle: Handle<Shader>,

    marker_settings: PhantomData<Settings>,
}

impl<Settings> PostProcessingPlugin<Settings> {
    pub(crate) fn new(name: &'static str, shader_handle: Handle<Shader>) -> Self {
        Self {
            name,
            shader_handle,
            marker_settings: PhantomData,
        }
    }
}

impl<C> Plugin for PostProcessingPlugin<C>
where
    C: ExtractComponent,
    C::Out: Component + ShaderType + WriteInto + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<C>::default());
        app.add_plugin(UniformComponentPlugin::<C::Out>::default());
        app.add_plugin(PrepareSpecializedPipelinesPlugin::<
            PostProcessingLayout<C::Out>,
            C::Out,
        >::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        let min_size = C::Out::min_size();
        info!(
            "Min size {min_size:?}, component: {:?}",
            ComponentDescriptor::new::<C::Out>()
        );
        let bgl = bind_group_layout(&mut render_app.world, self.name, min_size);

        if let Some(_r) = render_app
            .world
            .get_resource::<PostProcessingLayout<C::Out>>()
        {
            panic!("Should be no such resource")
        };
        // .expect("Should be no such resource");

        render_app
            .insert_resource(PostProcessingLayout::<C::Out>::new(
                self.name,
                bgl,
                self.shader_handle.clone(),
            ))
            .init_resource::<SpecializedRenderPipelines<PostProcessingLayout<C::Out>>>();

        add_nodes::<PostProcessingNode<C::Out>>(
            render_app,
            &format!("{}2D", self.name),
            &format!("{}3D", self.name),
        );
    }
}

#[derive(Resource)]
pub(crate) struct PostProcessingLayout<U> {
    name: &'static str,
    shader_handle: Handle<Shader>,
    pub(crate) bind_group_layout: BindGroupLayout,
    marker: PhantomData<U>,
}

impl<U> PostProcessingLayout<U> {
    fn new(
        name: &'static str,
        bind_group_layout: BindGroupLayout,
        shader_handle: Handle<Shader>,
    ) -> Self {
        Self {
            name,
            bind_group_layout,
            marker: PhantomData,
            shader_handle,
        }
    }
}

impl<U> SpecializedRenderPipeline for PostProcessingLayout<U> {
    type Key = TextureFormat;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        render_pipeline_descriptor(
            self.name,
            self.bind_group_layout.clone(),
            self.shader_handle.clone(),
            key,
        )
    }
}

struct PostProcessingNode<U: Component> {
    #[allow(clippy::type_complexity)]
    query: QueryState<
        (
            &'static ViewTarget,
            &'static SpecializedPipelinesCache<PostProcessingLayout<U>>,
            &'static DynamicUniformIndex<U>,
        ),
        With<ExtractedView>,
    >,
    // cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
    // name: &'static str,
}

impl<U: Component> FromWorld for PostProcessingNode<U> {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            // cached_texture_bind_group: Mutex::new(None),
        }
    }
}

impl<U> render_graph::Node for PostProcessingNode<U>
where
    U: Component + ShaderType + WriteInto,
{
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new("view", SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let view_entity = graph.get_input_entity("view")?;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_layout = world.resource::<PostProcessingLayout<U>>();
        let component_uniforms = world.resource::<ComponentUniforms<U>>();
        let globals_buffer = world.resource::<GlobalsBuffer>();

        let (target, pipeline, uniform_index) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        let pipeline = pipeline_cache
            .get_render_pipeline(pipeline.pipeline_id)
            .expect("Render pipeline should be cached");

        let post_process = target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;

        // let mut cached_bind_group = self
        //     .cached_texture_bind_group
        //     .lock()
        //     .expect("Lock not held");
        let sampler = render_context
            .render_device
            .create_sampler(&SamplerDescriptor {
                mipmap_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..default()
            });

        let bind_group = render_context
            .render_device
            .create_bind_group(&BindGroupDescriptor {
                // label: Some(self.name),
                label: None,
                layout: &pipeline_layout.bind_group_layout,
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
                        resource: globals_buffer
                            .buffer
                            .binding()
                            .expect("Globals buffer should be available"), // .clone(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: component_uniforms.binding().expect("This should work?"),
                    },
                ],
            });

        // let bind_group = match &mut *cached_bind_group {
        //     Some((id, bind_group)) if source.id() == *id => bind_group,
        //     cached_bind_group => {
        //         let sampler = render_context
        //             .render_device
        //             .create_sampler(&SamplerDescriptor {
        //                 mipmap_filter: FilterMode::Linear,
        //                 mag_filter: FilterMode::Linear,
        //                 min_filter: FilterMode::Linear,
        //                 ..default()
        //             });

        //         let bind_group =
        //             render_context
        //                 .render_device
        //                 .create_bind_group(&BindGroupDescriptor {
        //                     // label: Some(self.name),
        //                     label: None,
        //                     layout: &pipeline_layout.bind_group_layout,
        //                     entries: &[
        //                         BindGroupEntry {
        //                             binding: 0,
        //                             resource: BindingResource::TextureView(source),
        //                         },
        //                         BindGroupEntry {
        //                             binding: 1,
        //                             resource: BindingResource::Sampler(&sampler),
        //                         },
        //                         BindGroupEntry {
        //                             binding: 2,
        //                             resource: globals_buffer
        //                                 .buffer
        //                                 .binding()
        //                                 .expect("Globals buffer should be available"), // .clone(),
        //                         },
        //                         BindGroupEntry {
        //                             binding: 3,
        //                             resource: component_uniforms
        //                                 .binding()
        //                                 .expect("This should work?"),
        //                         },
        //                     ],
        //                 });

        //         let (_, bind_group) = cached_bind_group.insert((source.id(), bind_group));
        //         bind_group
        //     }
        // };

        let pass_descriptor = RenderPassDescriptor {
            // label: Some(self.name),
            label: None,
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
        // TODO: Make a default bind group here with source and destination.
        // Always set that as index 0.

        // TODO: Change to index 1 here and in shaders.
        render_pass.set_bind_group(0, &bind_group, &[uniform_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

/// TODO
#[doc(hidden)]
#[macro_export]
macro_rules! load_shader {
    ($app: ident, $handle: ident, $path_str: expr) => {{
        if cfg!(feature = "dev") {
            let asset_server = $app.world.resource::<AssetServer>();
            asset_server.load($path_str)
        } else {
            use bevy::asset::load_internal_asset;
            load_internal_asset!(
                $app,
                $handle,
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str),
                Shader::from_wgsl
            );
            $handle.typed()
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! load_image {
    ($app: ident, $path_str: expr, $ext:literal) => {{
        if cfg!(feature = "dev") {
            let asset_server = $app.world.resource::<AssetServer>();
            asset_server.load($path_str)
        } else {
            // use bevy::render::texture::ImageTextureLoader;
            use bevy::render::texture::{CompressedImageFormats, ImageType};
            // load_internal_asset!($app, $handle, $path_str, Image::from_bytes);
            let mut assets = $app.world.resource_mut::<Assets<_>>();
            assets.add(
                // $handle,
                (Image::from_buffer)(
                    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str)),
                    ImageType::Extension($ext),
                    CompressedImageFormats::NONE,
                    true,
                )
                .expect("image should load"),
            )
            // $handle.typed()

            // use bevy::render::texture::{CompressedImageFormats, ImageType};
            // let mut image_assets = $app
            //     .world
            //     .get_resource_mut::<Assets<Image>>()
            //     .expect("Should have Assets<Image>");

            // let image_bytes =
            //     include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str));
            // let image = Image::from_buffer(
            //     image_bytes,
            //     ImageType::Extension("tga"),
            //     CompressedImageFormats::NONE,
            //     true,
            // )
            // .expect("TGA should load properly");

            // image_assets.add(image)
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! load_lut {
    ($app: ident, $path_str: expr, $ext:literal) => {{
        use bevy::render::texture::{CompressedImageFormats, ImageType};

        let mut assets = $app.world.resource_mut::<Assets<_>>();

        let mut image = Image::from_buffer(
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str)),
            ImageType::Extension("png"), // todo
            CompressedImageFormats::NONE,
            // If `true` the output the mapping is very dark.
            // If not, it's much closer to the original.
            false,
        )
        .expect("Should be able to load image from buffer");

        image.texture_descriptor.dimension = TextureDimension::D3;
        image.texture_descriptor.size = Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 64,
        };
        image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

        image.texture_view_descriptor = Some(TextureViewDescriptor {
            label: Some("LUT TextureViewDescriptor"),
            format: Some(image.texture_descriptor.format),
            dimension: Some(TextureViewDimension::D3),
            ..default()
        });

        image.sampler_descriptor = ImageSampler::linear();

        let handle = assets.add(image);

        handle
        // LutImage(handle)
    }};
}

use std::sync::Mutex;

use bevy::{
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::TrackedRenderPass,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, FilterMode,
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
            SamplerDescriptor, TextureViewId,
        },
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

use super::{CameraPixelatePipeline, PixelatePipeline, PixelateUniform};

pub struct PixelateNode {
    query: QueryState<
        (
            &'static ViewTarget,
            &'static CameraPixelatePipeline,
            &'static DynamicUniformIndex<PixelateUniform>,
        ),
        With<ExtractedView>,
    >,
    cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
}

impl PixelateNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            cached_texture_bind_group: Mutex::new(None),
        }
    }
}

impl Node for PixelateNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(PixelateNode::IN_VIEW, SlotType::Entity)]
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
        let pipeline_cache = world.resource::<PipelineCache>();
        let pixelate_pipeline = world.resource::<PixelatePipeline>();
        let pixelate_uniforms = world.resource::<ComponentUniforms<PixelateUniform>>();

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
                            layout: &pixelate_pipeline.texture_bind_group,
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
                                    resource: pixelate_uniforms
                                        .binding()
                                        .expect("This should work?"),
                                },
                            ],
                        });

                let (_, bind_group) = cached_bind_group.insert((source.id(), bind_group));
                bind_group
            }
        };

        let pass_descriptor = RenderPassDescriptor {
            label: Some("pixelate_pass"),
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

use bevy::{
    prelude::{Handle, Shader, Vec2},
    render::{
        render_resource::{
            BindGroupLayout, BlendState, BufferVec, ColorTargetState, ColorWrites, FragmentState,
            FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
    },
};
use bytemuck::Pod;

pub(crate) const QUAD_VERTEX_POSITIONS: [Vec2; 6] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, -1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(-1.0, 1.0),
];

/// * Clear buffer
/// * Fill it with quad verts
/// * Write buffer
pub(crate) fn write_quad_buffer<T>(
    buffer: &mut BufferVec<T>,
    device: &RenderDevice,
    queue: &RenderQueue,
) where
    T: Pod,
    T: From<Vec2>,
{
    buffer.clear();

    for vertex in QUAD_VERTEX_POSITIONS {
        let t: T = vertex.into();
        buffer.push(t);
    }

    buffer.write_buffer(device, queue);
}

/// Describes a render pipeline which renders a full screen quad.
/// It will likely be reusable for several types of effects.
///
/// The shader is likely to be different as well as the bind group layout,
/// so those must be supplied at the callsite.
pub(crate) fn quad_render_pipeline_descriptor(
    label: &'static str,
    shader: Handle<Shader>,
    bind_group_layout: BindGroupLayout,
) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: Some(label.into()),
        layout: Some(vec![bind_group_layout]),
        vertex: VertexState {
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "vertex".into(),
            buffers: vec![VertexBufferLayout::from_vertex_formats(
                VertexStepMode::Vertex,
                vec![VertexFormat::Float32x2],
            )],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(FragmentState {
            shader,
            shader_defs: vec![],
            entry_point: "fragment".into(),
            targets: vec![Some(ColorTargetState {
                format: bevy::render::render_resource::TextureFormat::bevy_default(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
        }),
    }
}

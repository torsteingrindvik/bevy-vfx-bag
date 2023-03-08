use std::{f32::consts::PI, fmt::Display};

use bevy::render::{render_phase::AddRenderCommand, RenderSet};
pub(crate) use bevy::{
    asset::load_internal_asset,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
            BindingType, BufferBindingType, CachedRenderPipelineId, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
};

use crate::post_processing::{DrawPostProcessingEffect, UniformBindGroup};

use super::{Order, PostProcessingPhaseItem};

pub(crate) const CHROMATIC_ABERRATION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4357337502039082134);

#[derive(Resource)]
pub(crate) struct ChromaticAberrationData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayout,
}

impl FromWorld for ChromaticAberrationData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "ChromaticAberration",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ChromaticAberration::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            CHROMATIC_ABERRATION_SHADER_HANDLE.typed(),
        );

        ChromaticAberrationData {
            pipeline_id,
            uniform_layout,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            CHROMATIC_ABERRATION_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/",
                "chromatic-aberration.wgsl"
            ),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<ChromaticAberration>::default())
            .add_plugin(UniformComponentPlugin::<ChromaticAberration>::default());

        super::render_app(app)
            .add_system(
                super::extract_post_processing_camera_phases::<ChromaticAberration>.in_schedule(ExtractSchedule),
            )
            .init_resource::<ChromaticAberrationData>()
            .init_resource::<UniformBindGroup<ChromaticAberration>>()
            .add_system(prepare.in_set(RenderSet::Prepare))
            .add_system(queue.in_set(RenderSet::Queue))
            .add_render_command::<PostProcessingPhaseItem, DrawPostProcessingEffect<ChromaticAberration>>(
            );
    }
}

fn prepare(
    data: Res<ChromaticAberrationData>,
    mut views: Query<(
        Entity,
        &mut RenderPhase<PostProcessingPhaseItem>,
        &Order<ChromaticAberration>,
    )>,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions
            .read()
            .id::<DrawPostProcessingEffect<ChromaticAberration>>();

        phase.add(PostProcessingPhaseItem {
            entity,
            sort_key: (*order).into(),
            draw_function,
            pipeline_id: data.pipeline_id,
        });
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    data: Res<ChromaticAberrationData>,
    mut bind_group: ResMut<UniformBindGroup<ChromaticAberration>>,
    uniforms: Res<ComponentUniforms<ChromaticAberration>>,
    views: Query<Entity, With<ChromaticAberration>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("ChromaticAberration Uniform Bind Group"),
                layout: &data.uniform_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            }));
        }
    }
}

/// Chromatic Aberration settings.
#[derive(Debug, Copy, Clone, Component, ShaderType)]
pub struct ChromaticAberration {
    /// The direction (in UV space) the red channel is offset in.
    /// Will be normalized.
    pub dir_r: Vec2,

    /// How far (in UV space) the red channel should be displaced.
    pub magnitude_r: f32,

    /// The direction (in UV space) the green channel is offset in.
    /// Will be normalized.
    pub dir_g: Vec2,

    /// How far (in UV space) the green channel should be displaced.
    pub magnitude_g: f32,

    /// The direction (in UV space) the blue channel is offset in.
    /// Will be normalized.
    pub dir_b: Vec2,

    /// How far (in UV space) the blue channel should be displaced.
    pub magnitude_b: f32,
}

impl ChromaticAberration {
    /// Adds the given diff to the magnitude of all color channels.
    pub fn add_magnitude(&mut self, diff: f32) {
        self.magnitude_r += diff;
        self.magnitude_g += diff;
        self.magnitude_b += diff;
    }
}

impl Default for ChromaticAberration {
    fn default() -> Self {
        let one_third = (2. / 3.) * PI;

        Self {
            dir_r: Vec2::from_angle(0. * one_third),
            magnitude_r: 0.01,
            dir_g: Vec2::from_angle(1. * one_third),
            magnitude_g: 0.01,
            dir_b: Vec2::from_angle(2. * one_third),
            magnitude_b: 0.01,
        }
    }
}

impl Display for ChromaticAberration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base_angle = Vec2::new(1., 0.);
        let angle = |color_dir| base_angle.angle_between(color_dir) * 180. / PI + 180.;

        write!(
            f,
            "Chromatic Aberration [magnitude, angle]:  R: [{:.3}, {:4.0}°] G: [{:.3}, {:4.0}°] B: [{:.3}, {:4.0}°]",
            self.magnitude_r,
            angle(self.dir_r),
            self.magnitude_g,
            angle(self.dir_g),
            self.magnitude_b,
            angle(self.dir_b)
        )
    }
}

impl ExtractComponent for ChromaticAberration {
    type Query = (&'static Self, &'static Camera);
    type Filter = ();
    type Out = Self;

    fn extract_component((settings, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        if !camera.is_active {
            return None;
        }

        Some(*settings)
    }
}

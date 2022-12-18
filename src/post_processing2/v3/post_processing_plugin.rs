// TODO
// 1. Material2dPlugin::<M>::default()
// 2. Add extract component plugin for C
// 3. Move the handles resource which interacts with fixups etc. to the respective plugins
// 4. Add a general add_material system
// 5. Add a post processing layout for M
// 6. Add a specialized render pipeline for M
// 7. Add a phase item render command for M
// 8. Add an extract camera phases for C
// 9. Add a queue phase items for M, C

use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase},
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        Extract, RenderApp, RenderStage,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, Mesh2dPipelineKey, RenderMaterials2d},
    utils::FloatOrd,
};
use std::{hash::Hash, marker::PhantomData};

use super::{DrawPostProcessingItem, PostProcessingLayout, PostProcessingPhaseItem, VfxOrdering};

#[allow(clippy::type_complexity)]
fn extract_post_processing_camera_phases<C: Component>(
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

            commands
                .get_or_spawn(entity)
                .insert((RenderPhase::<PostProcessingPhaseItem>::default(), ordering));
        }
    }
}

/// Adds the phase items. Each time one is added it means that a render pass for the effect will be performed.
#[allow(clippy::complexity)]
pub fn queue_post_processing_phase_items<M: Material2d, C: Component>(
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
    render_materials: Res<RenderMaterials2d<M>>,
    pipeline: Res<PostProcessingLayout<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PostProcessingLayout<M>>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut views: Query<
        (
            Entity,
            &mut RenderPhase<PostProcessingPhaseItem>,
            &VfxOrdering<C>,
            &Handle<M>,
        ),
        With<C>,
    >,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
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

pub(crate) struct Plugin<M, C> {
    marker: PhantomData<(M, C)>,
}
impl<M, C> Default for Plugin<M, C> {
    fn default() -> Self {
        Self {
            marker: Default::default(),
        }
    }
}

impl<M, C> bevy::prelude::Plugin for Plugin<M, C>
where
    M: Material2d,
    M::Data: PartialEq + Eq + Hash + Clone,
    C: Component + ExtractComponent,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        // This handles the asset parts of the materials,
        // as well as making bind groups (TODO: Go through)
        app.add_plugin(Material2dPlugin::<M>::default());

        // This allows access to the user's settings for some effect
        // to be available in the render world.
        app.add_plugin(ExtractComponentPlugin::<C>::default());

        let render_app = app.get_sub_app_mut(RenderApp).expect("Should work");

        // This has:
        // - The shared bing group layout which has the source texture and sampler
        // - The layout specific for this material
        // - The handle to this material's shader
        render_app.init_resource::<PostProcessingLayout<M>>();

        // This has: TODO
        render_app.init_resource::<SpecializedRenderPipelines<PostProcessingLayout<M>>>();

        // This adds the steps we need to do to render this material with its specific bind group.
        render_app.add_render_command::<PostProcessingPhaseItem, DrawPostProcessingItem<M>>();

        // TODO: Change system to have with <C>
        render_app.add_system_to_stage(
            RenderStage::Extract,
            extract_post_processing_camera_phases::<C>,
        );

        // Creates (or pulls from cache) a pipeline id for each phase item, i.e. effect on a camera.
        render_app.add_system_to_stage(
            RenderStage::Queue,
            queue_post_processing_phase_items::<M, C>,
        );
    }
}

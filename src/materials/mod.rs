use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid, render::RenderApp};

#[derive(Debug, Default)]
pub(crate) struct MaterialsPlugin;

pub(crate) const MATH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3078479269656997866);

pub(crate) const VALUE_NOISE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2904626210061300097);

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        // let render_app = app
        //     .get_sub_app_mut(RenderApp)
        //     .expect("Need a render app for post processing");

        load_internal_asset!(
            app,
            MATH_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "math.wgsl"),
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            VALUE_NOISE_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/",
                "value_noise.wgsl"
            ),
            Shader::from_wgsl
        );

        // render_app
        //     .init_resource::<DrawFunctions<PostProcessingPhaseItem>>()
        //     .init_resource::<PostProcessingSharedBindGroups>()
        //     .init_resource::<PostProcessingSharedLayout>()
        //     .add_system_to_schedule(ExtractSchedule, extract_camera_phases)
        //     .add_system(queue_post_processing_shared_bind_groups.in_set(RenderSet::Queue))
        //     .add_system(sort_phase_system::<PostProcessingPhaseItem>.in_set(RenderSet::PhaseSort));

        // app.add_plugin(blur::Plugin);
        // app.add_plugin(chromatic_aberration::Plugin);
        // app.add_plugin(flip::Plugin);
        // app.add_plugin(lut::Plugin);
        // app.add_plugin(masks::Plugin);
        // app.add_plugin(raindrops::Plugin);
        // app.add_plugin(pixelate::Plugin);
        // app.add_plugin(wave::Plugin);
    }
}

use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid};

#[derive(Debug, Default)]
pub(crate) struct MaterialsPlugin;

pub(crate) const MATH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3078479269656997866);

pub(crate) const VALUE_NOISE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2904626210061300097);

pub(crate) const FBM_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6883246438893588577);

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
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

        load_internal_asset!(
            app,
            FBM_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "fbm.wgsl"),
            Shader::from_wgsl
        );
    }
}

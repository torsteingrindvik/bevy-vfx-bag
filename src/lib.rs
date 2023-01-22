#![allow(clippy::too_many_arguments)] // Bevy fns tend to have many args.
#![deny(clippy::unwrap_used)] // Let's try to explain invariants when we unwrap (so use expect).
#![deny(missing_docs)] // Let's try to have good habits.
#![doc = include_str!("../README.md")]

use bevy::prelude::*;

/// Post processing effects.
pub mod post_processing2;

/// For post processing effects to work, this marker should be added to a camera.
/// This camera will be changed to render to an image buffer which will then be applied
/// post processing to.
/// Note that UI will be disabled for the marked camera, and applied _after_ effects are added.
#[derive(Debug, Clone, Copy, Component)]
pub struct PostProcessingInput;

/// Macro for selecting a way to load shaders
/// based on the "dev" feature.
/// If the feature is on, we load from an assets-relative
/// path. Suitable for hot-reloading.
/// Else, the shader is loaded via the handle.
/// Suitable when this crate is used as a dependency.
#[doc(hidden)]
#[macro_export]
macro_rules! shader_ref {
    ($handle: ident, $path_str: expr) => {{
        use bevy::render::render_resource::ShaderRef;

        let s: ShaderRef = if cfg!(feature = "dev") {
            $path_str.into()
        } else {
            $handle.typed().into()
        };

        s
    }};
}

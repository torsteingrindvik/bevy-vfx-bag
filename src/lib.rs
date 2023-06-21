#![allow(clippy::too_many_arguments)] // Bevy fns tend to have many args.
#![deny(clippy::unwrap_used)] // Let's try to explain invariants when we unwrap (so use expect).
#![deny(missing_docs)] // Let's try to have good habits.
#![doc = include_str!("../README.md")]

/// Post processing effects.
pub mod post_processing;

/// Materials.
pub mod materials;

mod plugin;

pub use plugin::BevyVfxBagPlugin;

/// Utilities.
pub(crate) mod util;

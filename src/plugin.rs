use bevy::prelude::*;

use crate::post_processing;

/// The main plugin needed to use any effects.
#[derive(Debug, Default)]
pub struct BevyVfxBagPlugin;

impl Plugin for BevyVfxBagPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(post_processing::PostProcessingPlugin);
    }
}

pub(crate) use crate::{load_shader, post_processing2::util::PostProcessingPlugin};
pub(crate) use bevy::{
    asset::load_internal_asset,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

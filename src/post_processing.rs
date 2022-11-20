// use std::marker::PhantomData;

// use bevy::{
//     prelude::{FromWorld, Plugin, Resource},
//     render::render_resource::SpecializedRenderPipeline,
// };

/// Pixelation effect.
pub mod pixelate;

// trait Effect {
//     type Pipeline<P: Resource + FromWorld + SpecializedRenderPipeline>;
// }

// impl<T> Plugin for T
// where
//     T: Effect,
// {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         app.init_resource::<Self::Pipeline>();
//     }
// }

// trait Effect {
//     type Pipeline: SpecializedRenderPipeline;

//     fn thing(&self) -> Self::Pipeline;
// }

// struct PostProcessingPlugin<P> {
//     marker: PhantomData<fn() -> P>,
// }

// impl<P> Plugin for PostProcessingPlugin<P: Effect<Pipeline = P>> {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         todo!()
//     }
// }

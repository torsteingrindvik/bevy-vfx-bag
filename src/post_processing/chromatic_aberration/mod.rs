use bevy::{
    prelude::{Component, Deref, Resource},
    render::render_resource::{BindGroupLayout, BindingResource, ShaderType},
};

use super::{traits, PostProcessingNode};

mod node;

const CHROMATIC_ABERRATION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1793333381364049744);

struct ChromaticAberrationNode {
    bind_group_layout: BindGroupLayout,
}

/// TODO
#[derive(Debug, Component)]
pub struct ChromaticAberration {
    enabled: bool,
    hello: f32,
}

/// TODO
#[derive(Debug, ShaderType, Component, Clone)]
pub struct ChromaticAberrationUniform {
    hello: f32,
}
// fn bind_group_layout()

/// TODO
#[derive(Resource, Deref)]
pub struct ChromaticAberrationPipeline {
    texture_bind_group: BindGroupLayout,
}

impl traits::PostProcessingNode for ChromaticAberrationNode {
    const IN_VIEW: &'static str = "view";

    type Key = ();

    type Uniform = ChromaticAberrationUniform;

    type ComponentPipeline;

    fn shader_defs(&self, key: Self::Key) -> Vec<String> {
        vec![]
    }

    fn shader(&self) -> Handle<Shader> {
        CHROMATIC_ABERRATION_SHADER_HANDLE.typed()
    }

    fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn binding_resource(&self, world: &bevy::prelude::World) -> BindingResource {
        todo!()
    }

    fn pass_label(&self) -> Option<&'static str> {
        Some("CA2")
    }
    // const IN_VIEW: &'static str = "VIEW";

    // type Uniform = ChromaticAberrationUniform;
    // type ComponentPipeline = ();

    // fn bind_group_layout(
    //     &self,
    //     world: &bevy::prelude::World,
    // ) -> &bevy::render::render_resource::BindGroupLayout {
    //     world.get_resource()
    // }

    // fn binding_resource(
    //     &self,
    //     world: &bevy::prelude::World,
    // ) -> bevy::render::render_resource::BindingResource {
    //     todo!()
    // }

    // fn pass_label(&self) -> Option<&'static str> {
    //     Some("ChromaticAberration")
    // }
}

pub struct ChromaticAberrationPlugin;

impl traits::PostProcessingPlugin for ChromaticAberrationPlugin {
    const NODE_NAME_3D: &'static str = "ChromaticAberration";

    type UserSettings = ChromaticAberration;
    type Uniform = ChromaticAberrationUniform;
    type Node = ChromaticAberrationNode;
}

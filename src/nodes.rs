use bevy::render::render_graph::{Node, RenderGraph};
use bevy::{prelude::*, render::RenderApp};

pub(crate) const VIEW: &str = "view";

pub(crate) fn add_node_before_ui_pass(app: &mut App, node: impl Node, name: &'static str) {
    let render_app = app
        .get_sub_app_mut(RenderApp)
        .expect("Should be able to get RenderApp SubApp");

    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    let graph = graph
        .get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME)
        .expect("Core pipeline 3D subgraph should exist");

    graph.add_node(name, node);

    let root = graph
        .input_node()
        .expect("Should be able to retrieve 3D subgraph input node")
        .id;

    let root_output = bevy::core_pipeline::core_3d::graph::input::VIEW_ENTITY;
    let main_pass = bevy::core_pipeline::core_3d::graph::node::MAIN_PASS;
    let ui_pass = bevy::ui::draw_ui_graph::node::UI_PASS;

    // Our pass needs access to the `world` etc, so grab the output of this sub graph
    graph
        .add_slot_edge(root, root_output, name, VIEW)
        .expect("Should be able to add slot edge from root to new node");

    // Place ourselves in-between the main pass and the ui pass
    graph
        .add_node_edge(main_pass, name)
        .expect("Should be able to add node edge from main pass to new node");
    graph
        .add_node_edge(name, ui_pass)
        .expect("Should be able to add node edge from new node to UI pass");
}

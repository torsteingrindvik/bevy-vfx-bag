use bevy::{
    core_pipeline::{core_2d, core_3d},
    prelude::*,
    render::render_graph::{self, RenderGraph},
};

pub fn add_nodes<T: FromWorld + render_graph::Node>(
    render_app: &mut App,
    name_2d: &str,
    name_3d: &str,
) {
    {
        let node = <T as FromWorld>::from_world(&mut render_app.world);
        let mut binding = render_app.world.resource_mut::<RenderGraph>();
        let graph = binding
            .get_sub_graph_mut(core_3d::graph::NAME)
            .expect("Graph should be available");

        graph.add_node(name_3d.to_owned(), node);

        graph.add_slot_edge(
            graph.input_node().id,
            core_3d::graph::input::VIEW_ENTITY,
            name_3d.to_owned(),
            "view",
        );

        graph.add_node_edge(core_3d::graph::node::MAIN_PASS, name_3d.to_owned());

        graph.add_node_edge(
            name_3d.to_owned(),
            core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        );
    }
    {
        let node = <T as FromWorld>::from_world(&mut render_app.world);
        let mut binding = render_app.world.resource_mut::<RenderGraph>();
        let graph = binding
            .get_sub_graph_mut(core_2d::graph::NAME)
            .expect("Graph should be available");

        graph.add_node(name_2d.to_owned(), node);

        graph.add_slot_edge(
            graph.input_node().id,
            core_2d::graph::input::VIEW_ENTITY,
            name_2d.to_owned(),
            "view",
        );

        graph.add_node_edge(
            name_2d.to_owned(),
            core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        );
    }
}

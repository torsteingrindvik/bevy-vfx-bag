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

/// TODO
#[doc(hidden)]
#[macro_export]
macro_rules! load_shader {
    ($app: ident, $handle: ident, $path_str: expr) => {{
        if cfg!(feature = "dev") {
            let asset_server = $app.world.resource::<AssetServer>();
            asset_server.load($path_str)
        } else {
            use bevy::asset::load_internal_asset;
            load_internal_asset!(
                $app,
                $handle,
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str),
                Shader::from_wgsl
            );
            $handle.typed()
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! load_image {
    ($app: ident, $path_str: expr, $ext:literal, $srgb: expr) => {{
        if cfg!(feature = "dev") {
            let asset_server = $app.world.resource::<AssetServer>();
            let handle = asset_server.load($path_str);
            info!("Loading image: {}, handle: {:?}", $path_str, &handle);
            handle
        } else {
            // use bevy::render::texture::ImageTextureLoader;
            use bevy::render::texture::{CompressedImageFormats, ImageType};
            // load_internal_asset!($app, $handle, $path_str, Image::from_bytes);
            let mut assets = $app.world.resource_mut::<Assets<_>>();
            assets.add(
                // $handle,
                (Image::from_buffer)(
                    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str)),
                    ImageType::Extension($ext),
                    CompressedImageFormats::NONE,
                    $srgb,
                )
                .expect("image should load"),
            )
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! load_lut {
    // ($app: ident, $path_str: expr, $ext:literal) => {{
    //     use bevy::render::texture::{CompressedImageFormats, ImageType};

    //     let handle = if cfg!(feature = "dev") {
    //         let asset_server = $app.world.resource::<AssetServer>();
    //         let handle = asset_server.load($path_str);
    //         info!("Loading image: {}, handle: {:?}", $path_str, &handle);
    //         handle
    //     } else {
    //         let mut assets = $app.world.resource_mut::<Assets<_>>();

    //         let mut image = Image::from_buffer(
    //             include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path_str)),
    //             ImageType::Extension("png"), // todo
    //             CompressedImageFormats::NONE,
    //             // If `true` the output the mapping is very dark.
    //             // If not, it's much closer to the original.
    //             false,
    //         )
    //         .expect("Should be able to load image from buffer");

    //         image.texture_descriptor.dimension = TextureDimension::D3;
    //         image.texture_descriptor.size = Extent3d {
    //             width: 64,
    //             height: 64,
    //             depth_or_array_layers: 64,
    //         };
    //         image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

    //         image.texture_view_descriptor = Some(TextureViewDescriptor {
    //             label: Some("LUT TextureViewDescriptor"),
    //             format: Some(image.texture_descriptor.format),
    //             dimension: Some(TextureViewDimension::D3),
    //             ..default()
    //         });

    //         image.sampler_descriptor = ImageSampler::linear();

    //         let handle = assets.add(image);

    //         handle
    //     };

    //     (handle, IsFixed(false))
    // }};
    ($app: ident, $path_str: expr, $ext:literal) => {{
        let handle = load_image!($app, $path_str, $ext, false);
        (handle, IsFixed(false))
    }};
}

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

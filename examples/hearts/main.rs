#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

fn main() {
    App::new()
        .add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::default())
        .add_plugin(Material2dPlugin::<HeartMaterial>::default())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HeartMaterial>>,
    windows: Query<&Window>,
) {
    let window = windows.single();

    let hearts_quad_1 = shape::Quad::new(Vec2::new(
        (window.physical_width() - 300) as f32,
        (window.physical_height() - 150) as f32,
    ));
    let hearts_color_1 = Color::Rgba {
        red: 0.9,
        green: 0.,
        blue: 0.,
        alpha: 0.9,
    };
    let hearts_transform_1 = Transform::from_xyz(0., 0., 0.);

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(hearts_quad_1.into())),
        material: materials.add(HeartMaterial {
            color: hearts_color_1,
            num_hearts: Vec2::new(5., 2.),
        }),
        transform: hearts_transform_1,
        ..default()
    });

    let hearts_quad_2 = shape::Quad::new(Vec2::new(
        (window.physical_width() / 10) as f32,
        (window.physical_height() / 4) as f32,
    ));
    let hearts_color_2 = Color::Rgba {
        red: 0.1,
        green: 0.,
        blue: 0.9,
        alpha: 0.9,
    };
    let hearts_transform_2 = Transform::from_xyz(0., 0., 1.);

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(hearts_quad_2.into())),
        material: materials.add(HeartMaterial {
            color: hearts_color_2,
            num_hearts: Vec2::new(4., 4.),
        }),
        transform: hearts_transform_2,
        ..default()
    });

    commands.spawn((Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    },));
}

impl Material2d for HeartMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/heart.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "ff664fca-c02f-11ed-bf9f-325096b39f47"]
pub struct HeartMaterial {
    #[uniform(0)]
    color: Color,

    #[uniform(1)]
    num_hearts: Vec2,
}

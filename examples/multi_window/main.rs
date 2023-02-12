#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{Window, WindowRef},
};

use bevy_vfx_bag::post_processing::{
    blur::Blur, chromatic_aberration::ChromaticAberration, flip::Flip, lut::Lut, masks::Mask,
    pixelate::Pixelate, raindrops::Raindrops, wave::Wave, PostProcessingOrder,
    PostProcessingPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PostProcessingPlugin::default())
        .add_startup_system(setup);

    app.run();
}

fn setup(mut commands: Commands) {
    let transform = Transform::from_xyz(-5.0, 12., 10.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y);

    // First window: Clean, no effects
    commands.spawn((Camera3dBundle {
        transform,
        ..default()
    },));

    // Second window: Camera has effects
    let window_2 = commands.spawn(Window::default()).id();
    commands.spawn((
        Camera3dBundle {
            transform,
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(window_2)),
                ..default()
            },
            ..default()
        },
        Wave::default().with_order(0.),
        Pixelate::default().with_order(1.),
        Mask::default().with_order(2.),
        Lut::default().with_order(3.),
        Blur::default().with_order(4.),
        Flip::default().with_order(5.),
    ));

    // Third window: Camera has other effects
    let window_3 = commands.spawn(Window::default()).id();
    commands.spawn((
        Camera3dBundle {
            transform,
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(window_3)),
                ..default()
            },
            ..default()
        },
        Mask::crt().with_order(0.),
        Lut::arctic().with_order(1.),
        ChromaticAberration::default().with_order(2.),
        Raindrops::default().with_order(3.),
    ));
}

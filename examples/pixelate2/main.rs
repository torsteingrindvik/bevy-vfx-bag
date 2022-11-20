#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{CreateWindow, PresentMode, WindowId},
};

use bevy_vfx_bag::post_processing::pixelate::{Pixelate, PixelatePlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PixelatePlugin)
        .add_startup_system(startup);
    // // .add_system(update)
    // .run();

    // let s = bevy_mod_debugdump::get_render_schedule(&mut app);
    // let mut f = std::fs::File::create("pixelate-render-schedule.dot").unwrap();
    // f.write_all(s.as_bytes()).unwrap();

    // let s = bevy_mod_debugdump::get_render_graph(&mut app);
    // let mut f = std::fs::File::create("pixelate-renderjV-graph.dot").unwrap();
    // f.write_all(s.as_bytes()).unwrap();

    app.run();
}

fn startup(mut commands: Commands, mut create_window_events: EventWriter<CreateWindow>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.0, 12., 10.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        Pixelate {
            enabled: true,
            block_size: 10.0,
        },
        // examples_common::LeftCamera,
    ));

    let window_id = WindowId::new();

    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            present_mode: PresentMode::AutoNoVsync,
            title: "Second window".to_string(),
            ..default()
        },
    });

    // second window camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.0, 12., 10.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            camera: Camera {
                target: RenderTarget::Window(window_id),
                ..default()
            },
            ..default()
        },
        Pixelate {
            enabled: true,
            block_size: 5.0,
        },
    ));

    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_xyz(0.0, 6., 12.0)
    //             .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    //         camera: Camera {
    //             priority: 1, // To clear up ambiguities
    //             ..default()
    //         },
    //         camera_3d: Camera3d {
    //             clear_color: ClearColorConfig::None, // To not overwrite previous camera's work
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Pixelate {
    //         enabled: true,
    //         block_size: 50.0,
    //     },
    //     examples_common::RightCamera,
    // ));
}

// fn update(keyboard_input: Res<Input<KeyCode>>, mut text: ResMut<examples_common::ExampleText>) {
//     let mut pixelate_diff = 0.0;

//     if keyboard_input.just_pressed(KeyCode::P) {
//         // passthrough.0 = !passthrough.0;
//     }

//     if keyboard_input.just_pressed(KeyCode::Up) {
//         pixelate_diff = 1.0;
//     } else if keyboard_input.just_pressed(KeyCode::Down) {
//         pixelate_diff = -1.0;
//     }

//     // pixelate.block_size += pixelate_diff;
//     // pixelate.block_size = 1.0_f32.max(pixelate.block_size);

//     // text.0 = format!(
//     //     "Pixelate block size (↑↓): {:.2?}, [P]assthrough: {:?}",
//     //     pixelate.block_size, passthrough.0
//     // );
// }

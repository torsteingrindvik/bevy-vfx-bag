#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::post_processing::{flip::Flip, PostProcessingPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PostProcessingPlugin::default())
        .add_startup_system(startup)
        .add_system(examples_common::print_on_change::<Flip>)
        .add_system_to_schedule(CoreSchedule::FixedUpdate, update)
        .insert_resource(FixedTime::new_from_secs(1.5))
        .run();
}

fn startup(mut commands: Commands) {
    info!("Flips the screen orientation every interval.");

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        Flip::default(),
    ));
}

// Switch flip modes every second.
fn update(mut query: Query<&mut Flip, With<Camera>>) {
    let mut flip = query.single_mut();

    *flip = match *flip {
        Flip::None => Flip::Horizontal,
        Flip::Horizontal => Flip::Vertical,
        Flip::Vertical => Flip::HorizontalVertical,
        Flip::HorizontalVertical => Flip::None,
    };
}

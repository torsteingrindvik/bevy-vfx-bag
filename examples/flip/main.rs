#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy::render::camera::RenderTarget;
use bevy::time::FixedTimestep;
use bevy_vfx_bag::camera::flip::{Flip, FlipPlugin};
use bevy_vfx_bag::{BevyVfxBagImage, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        // The [`FlipPlugin`] will insert a default [`Flip::None`] unless
        // you insert one before the plugin, like this.
        .insert_resource(Flip::Horizontal)
        .add_plugin(FlipPlugin)
        .add_startup_system(startup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(update),
        )
        .run();
}

fn startup(mut commands: Commands, image_handle: Res<BevyVfxBagImage>) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            camera: Camera {
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .insert(UiCameraConfig { show_ui: false });
}

// Switch flip modes every second.
fn update(mut flip: ResMut<Flip>, mut text: ResMut<examples_common::ExampleText>) {
    *flip = match *flip {
        Flip::None => Flip::Horizontal,
        Flip::Horizontal => Flip::Vertical,
        Flip::Vertical => Flip::HorizontalVertical,
        Flip::HorizontalVertical => Flip::None,
    };

    // Display on screen which mode we are in
    text.0 = format!("{:?}", *flip);
}

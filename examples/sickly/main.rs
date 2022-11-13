#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::tint::{Tint, TintPlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(TintPlugin)
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(PostProcessingInput);
}

fn update(time: Res<Time>, mut tint: ResMut<Tint>, mut text: ResMut<examples_common::ExampleText>) {
    // Display blur amount on screen
    // text.0 = format!(
    //     "Blur (↑↓): {:.2?}, radius (←→): {:.3?}",
    //     blur.amount, blur.kernel_radius
    // );
    let g = (time.elapsed_seconds().sin() + 1.0) / 2.0;

    tint.color = Color::rgb(1.0, g, 1.0);
}

use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use core::f32::consts::PI;

/// Adds some "sane defaults" for showing examples/development:
///
/// * The default Bevy plugins
/// * Hot reloading
/// * Close on ESC button press
pub struct SaneDefaultsPlugin;

#[derive(Debug, Resource)]
pub struct ExampleText(pub String);

impl Default for ExampleText {
    fn default() -> Self {
        Self("Loading...".into())
    }
}

impl Plugin for SaneDefaultsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExampleText>()
            .add_plugins(
                DefaultPlugins
                    .set(AssetPlugin {
                        watch_for_changes: true,
                        ..default()
                    })
                    .set(ImagePlugin::default_nearest()),
            )
            .add_system(bevy::window::close_on_esc);
    }
}

/// This plugin combines two Bevy examples:
///
/// https://github.com/bevyengine/bevy/blob/v0.8.1/examples/3d/shapes.rs
/// https://github.com/bevyengine/bevy/blob/v0.8.1/examples/ui/text.rs
///
/// This example can be started by just loading this plugin.
/// This is useful to separate this crate's code and role from regular upstream Bevy code.
pub struct ShapesExamplePlugin {
    add_3d_camera_bundle: bool,
}

impl ShapesExamplePlugin {
    // pub fn with_3d_camera() -> Self {
    //     Self {
    //         add_3d_camera_bundle: true,
    //     }
    // }

    pub fn without_3d_camera() -> Self {
        Self {
            add_3d_camera_bundle: false,
        }
    }
}

#[derive(Resource)]
pub(crate) struct ShouldAdd3dCameraBundle(bool);

impl Plugin for ShapesExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShouldAdd3dCameraBundle(self.add_3d_camera_bundle))
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(shapes::setup)
            .add_startup_system(ui::setup)
            .add_system(shapes::rotate)
            .add_system(ui::fps_text_update)
            .add_system(ui::ui_text_update);
    }
}

#[derive(Component)]
pub(crate) struct Shape;

const X_EXTENT: f32 = 14.;

mod shapes {
    use super::*;

    pub(crate) fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut images: ResMut<Assets<Image>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        add_3d_camera_bundle: Res<ShouldAdd3dCameraBundle>,
    ) {
        let debug_material = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        });

        let shapes = [
            meshes.add(shape::Cube::default().into()),
            meshes.add(shape::Box::default().into()),
            meshes.add(shape::Capsule::default().into()),
            meshes.add(shape::Torus::default().into()),
            meshes.add(shape::Icosphere::default().into()),
            meshes.add(shape::UVSphere::default().into()),
        ];

        let num_shapes = shapes.len();

        for (i, shape) in shapes.into_iter().enumerate() {
            commands
                .spawn(PbrBundle {
                    mesh: shape.clone(),
                    material: debug_material.clone(),
                    transform: Transform::from_xyz(
                        -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                        2.0,
                        0.0,
                    )
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                    ..default()
                })
                .insert(Shape);

            commands
                .spawn(PbrBundle {
                    mesh: shape,
                    material: debug_material.clone(),
                    transform: Transform::from_xyz(
                        -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                        2.0,
                        0.0,
                    )
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                    ..default()
                })
                .insert(Shape);
        }

        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 9000.0,
                range: 100.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(8.0, 16.0, 8.0),
            ..default()
        });

        // ground plane
        commands.spawn(PbrBundle {
            mesh: meshes.add(shape::Plane { size: 50. }.into()),
            material: materials.add(Color::SILVER.into()),
            ..default()
        });

        if add_3d_camera_bundle.0 {
            commands.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 6., 12.0)
                    .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
                ..default()
            });
        }
    }

    pub(crate) fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 2.);
        }
    }

    /// Creates a colorful test pattern
    fn uv_debug_texture() -> Image {
        const TEXTURE_SIZE: usize = 8;

        let mut palette: [u8; 32] = [
            255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102,
            255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
        ];

        let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
        for y in 0..TEXTURE_SIZE {
            let offset = TEXTURE_SIZE * y * 4;
            texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
            palette.rotate_right(4);
        }

        Image::new_fill(
            Extent3d {
                width: TEXTURE_SIZE as u32,
                height: TEXTURE_SIZE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &texture_data,
            TextureFormat::Rgba8UnormSrgb,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// UI
////////////////////////////////////////////////////////////////////////////////

mod ui {
    use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};

    use super::*;

    // A unit struct to help identify the FPS UI component, since there may be many Text components
    #[derive(Component)]
    pub(crate) struct FpsText;

    // A unit struct to help identify the example UI text component
    #[derive(Component)]
    pub(crate) struct UiText;

    pub(crate) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        // UI camera
        // commands.spawn_bundle(Camera2dBundle::default());
        // Text with one section
        commands
            .spawn(
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    "hello\nbevy!",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::TOP_CENTER)
                // Set the style of the TextBundle itself.
                .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(5.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(UiText);
        // Text with multiple sections
        commands
            .spawn(
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        "FPS: ",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 40.0,
                        color: Color::GOLD,
                    }),
                ])
                .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(FpsText);
    }

    pub(crate) fn ui_text_update(
        message: Res<ExampleText>,
        mut query: Query<&mut Text, With<UiText>>,
    ) {
        for mut text in &mut query {
            text.sections[0].value = message.0.clone();
        }
    }

    pub(crate) fn fps_text_update(
        diagnostics: Res<Diagnostics>,
        mut query: Query<&mut Text, With<FpsText>>,
    ) {
        for mut text in &mut query {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(average) = fps.average() {
                    // Update the value of the second section
                    text.sections[1].value = format!("{average:.2}");
                }
            }
        }
    }
}

#[allow(dead_code)]
fn main() {
    println!("Not an example, just shared code between examples")
}

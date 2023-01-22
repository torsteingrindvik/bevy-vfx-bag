use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{
        camera::Viewport,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::WindowResized,
};
use core::f32::consts::PI;

/// Adds some "sane defaults" for showing examples/development:
///
/// * The default Bevy plugins
/// * Hot reloading
/// * Close on ESC button press
pub struct SaneDefaultsPlugin;

#[derive(Debug, Clone)]
pub struct ListItem {
    name: String,
    enabled: bool,
}

#[derive(Debug, Component, Default, Clone)]
pub struct List {
    items: Vec<ListItem>,
    is_selected: bool,
    line_pointed_to: usize,
    font: Handle<Font>,
}

impl List {
    pub fn new<S: AsRef<str>>(font: Handle<Font>, items: impl IntoIterator<Item = S>) -> Self {
        Self {
            font,
            items: items
                .into_iter()
                .map(|name| ListItem {
                    name: name.as_ref().to_string(),
                    enabled: true,
                })
                .collect(),
            ..default()
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.items
            .iter()
            .find(|item| item.name == name)
            .expect("The item with the given name should be part of the list")
            .enabled
    }

    pub fn enabled_items(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|item| item.enabled)
            .map(|item| item.name.as_str())
            .collect()
    }

    pub fn disabled_items(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|item| !item.enabled)
            .map(|item| item.name.as_str())
            .collect()
    }

    pub fn as_text(&self) -> Text {
        Text::from_sections(
            self.items
                .iter()
                .enumerate()
                .map(move |(i, item)| {
                    let mut value = "".to_owned();
                    if i == self.line_pointed_to {
                        value += "→ ";
                    }
                    value += item.name.as_str();
                    value += "\n";

                    let mut text_section = TextSection {
                        value,
                        style: TextStyle {
                            font: self.font.clone(),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    };
                    if i == self.line_pointed_to && self.is_selected {
                        text_section.style.color = Color::GOLD;
                    } else if item.enabled {
                        text_section.style.color = Color::WHITE;
                    } else {
                        text_section.style.color = Color::DARK_GRAY;
                    }
                    text_section
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn as_text_bundle(&self) -> TextBundle {
        TextBundle {
            text: self.as_text(),
            ..default()
        }
        .with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        })
    }

    pub fn items(&self) -> impl Iterator<Item = &ListItem> {
        self.items.iter()
    }

    pub fn toggle_selected(&mut self) {
        self.is_selected = !self.is_selected;
    }

    pub fn toggle_enabled(&mut self) {
        self.items[self.line_pointed_to].enabled = !self.items[self.line_pointed_to].enabled;
    }

    fn change_line_pointed_to(&mut self, diff: isize) {
        let pointed_to_before = self.line_pointed_to;
        if diff == 1 {
            self.line_pointed_to = (self.line_pointed_to + 1).min(self.items.len() - 1);
        } else {
            self.line_pointed_to = pointed_to_before.saturating_sub(1);
        }

        if self.is_selected && (self.line_pointed_to != pointed_to_before) {
            self.items.swap(self.line_pointed_to, pointed_to_before);
        }
    }

    pub fn up(&mut self) {
        self.change_line_pointed_to(-1);
    }
    pub fn down(&mut self) {
        self.change_line_pointed_to(1);
    }
}

impl Plugin for SaneDefaultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
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
            .add_system(set_camera_viewports)
            .add_system(shapes::rotate)
            .add_system(ui::fps_text_update);
        // .add_system(ui::ui_change_text)
        // .add_system(ui::ui_change_selection);
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
            meshes.add(shape::Icosphere::default().try_into().unwrap()),
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

    // #[derive(Resource, Default)]
    // pub(crate) struct TextSelection {
    //     index: usize,
    //     is_selected: bool,
    // }

    // impl TextSelection {
    //     pub(crate) fn toggle(&mut self) {
    //         self.is_selected = !self.is_selected;
    //     }

    //     pub(crate) fn next(&mut self) {
    //         // Out of bounds check is handled by the UI system
    //         self.index = (self.index + 1);
    //     }

    //     pub(crate) fn previous(&mut self) {
    //         self.index = self.index.saturating_sub(1);
    //     }
    // }

    pub(crate) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        // UI camera
        // commands.spawn_bundle(Camera2dBundle::default());
        // Text with one section
        commands
            .spawn(
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::Center)
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

    // pub(crate) fn ui_change_text(
    //     example_texts: Res<ExampleTexts>,
    //     mut query: Query<&mut Text, With<UiText>>,
    // ) {
    //     if !example_texts.is_changed() {
    //         return;
    //     }

    //     let mut text = query.single_mut();

    //     let is_selected = example_text.is_selected;
    //     let list_index = example_text.line_pointed_to;

    //     *text = Text::from_sections(example_text.lines.iter().enumerate().map(|(index, line)| {
    //         let color = if is_selected && index == list_index {
    //             Color::GOLD
    //         } else {
    //             Color::WHITE
    //         };
    //         let prefix = if index == list_index { "> " } else { "  " };

    //         TextSection::new(
    //             format!("{prefix}{line}\n"),
    //             TextStyle {
    //                 font: text.sections[0].style.font.clone(),
    //                 font_size: text.sections[0].style.font_size,
    //                 color,
    //             },
    //         )
    //     }));
    //     // for mut text in &mut query {
    //     // text.sections[0].value = message.0.clone();
    //     // }
    // }

    // pub(crate) fn ui_change_selection(
    //     // selection: Res<TextSelection>,
    //     mut query: Query<&mut Text, With<UiText>>,
    // ) {
    //     if !selection.is_changed() {
    //         return;
    //     }

    //     let mut text = query.single_mut();

    //     for (index, section) in text.sections.iter_mut().enumerate() {
    //         if selection.is_selected && selection.index == index {
    //             section.style.color = Color::GOLD;
    //         } else {
    //             section.style.color = Color::WHITE;
    //         }
    //     }

    //     // *text = Text::from_sections(message.0.lines().map(|line| {
    //     //     TextSection::new(
    //     //         format!("{line}\n"),
    //     //         TextStyle {
    //     //             font: text.sections[0].style.font.clone(),
    //     //             font_size: text.sections[0].style.font_size,
    //     //             color: text.sections[0].style.color,
    //     //         },
    //     //     )
    //     // }));
    //     // for mut text in &mut query {
    //     // text.sections[0].value = message.0.clone();
    //     // }
    // }

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

#[derive(Component)]
pub struct LeftCamera;

#[derive(Component)]
pub struct RightCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        let window = windows.get(resize_event.window).unwrap();

        let mut left_camera = match left_camera.get_single_mut() {
            Ok(lc) => lc,
            Err(_) => return,
        };
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });

        let mut right_camera = match right_camera.get_single_mut() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        right_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(window.physical_width() / 2, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
// MENU STUFF
// See https://github.com/bevyengine/bevy/blob/main/examples/games/game_menu.rs
////////////////////////////////////////////////////////////////////////////////
// const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
// const BUTTON_LIST_BACKGROUND: Color = Color::rgba(0.3, 0.0, 0.3, 0.15);

// const NORMAL_BUTTON: Color = Color::rgba(0.15, 0.15, 0.15, 0.7);
// const HOVERED_BUTTON: Color = Color::rgba(0.25, 0.25, 0.25, 0.7);
// const HOVERED_PRESSED_BUTTON: Color = Color::rgba(0.25, 0.65, 0.25, 0.7);
// const PRESSED_BUTTON: Color = Color::rgba(0.35, 0.75, 0.35, 0.7);

// // Tag component used to mark wich setting is currently selected
// #[derive(Component)]
// pub struct SelectedOption;

// fn button_color_changer(
//     mut interaction_query: Query<
//         (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
//         (Changed<Interaction>, With<Button>),
//     >,
// ) {
//     for (interaction, mut color, selected) in &mut interaction_query {
//         *color = match (*interaction, selected) {
//             (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
//             (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
//             (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
//             (Interaction::None, None) => NORMAL_BUTTON.into(),
//         }
//     }
// }

// This system updates the settings when a new value for a setting is selected, and marks
// the button as the one currently selected
// pub fn button_selector(
//     interaction_query: Query<
//         (&Interaction, Entity),
//         (Changed<Interaction>, With<Button>, Without<ButtonArrow>),
//     >,
//     mut selected_query: Query<(Entity, &mut BackgroundColor), With<SelectedOption>>,
//     mut commands: Commands,
//     // mut setting: ResMut<T>,
// ) {
//     for (interaction, entity) in &interaction_query {
//         if *interaction == Interaction::Clicked {
//             let (previous_button, mut previous_color) = selected_query.single_mut();
//             *previous_color = NORMAL_BUTTON.into();
//             commands.entity(previous_button).remove::<SelectedOption>();
//             commands.entity(entity).insert(SelectedOption);
//             // *setting = *button_setting;
//         }
//     }
// }

// trait RelatesToButton {
//     fn related_button(&self) -> Option<Entity>;
// }

// pub fn button_arrows_handler<
//     C: Component + Display + Ord + Clone + RelatesToButton,
//     T: Resource + Clone + IntoIterator<Item = C>,
// >(
//     interaction_query: Query<(&Interaction, &ButtonArrow), Changed<Interaction>>,
//     // mut selected_query: Query<(Entity, &mut BackgroundColor), With<SelectedOption>>,
//     mut commands: Commands,
//     mut buttons: ResMut<T>,
// ) {
//     for (interaction, button_arrow) in &interaction_query {
//         if *interaction == Interaction::Clicked {
//             let ButtonArrow {
//                 direction,
//                 related_setting,
//             } = button_arrow;

//             let sorted_buttons = buttons.clone().into_iter().find(|c| c == related_setting);

//             // let component_to_insert = if direction == &ButtonArrowDirection::Up {
//             //     Bu
//             // } else {
//             //     SelectedOption
//             // };

//             // commands.entity(*related_setting).insert(SelectedOption);

//             // let (previous_button, mut previous_color) = selected_query.single_mut();
//             // *previous_color = NORMAL_BUTTON.into();
//             // commands.entity(previous_button).remove::<SelectedOption>();
//             // commands.entity(entity).insert(SelectedOption);
//             // *setting = *button_setting;
//         }
//     }
// }

// #[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// enum ButtonArrowDirection {
//     Up,
//     Down,
// }

// #[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// enum ButtonArrowDirection {
//     Up,
//     Down,
// }

// #[derive(Component, Clone, Copy)]
// pub struct ButtonArrow {
//     direction: ButtonArrowDirection,
//     related_setting: Entity,
// }

// Make a list of buttons for each thing in the list.
// The items in the list must implement display such that they can be displayed as text on the button.
// Also the items must be ordered so that the list can be sorted.
// This allows the list to change order at runtime, and the buttons will be updated to match.
// pub fn buttons_list<
//     C: Component + Display + Ord + Clone,
//     T: Resource + Clone + IntoIterator<Item = C>,
// >(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     listable: Res<T>,
// ) {
//     if !listable.is_changed() {
//         return;
//     }

//     let button_style = Style {
//         size: Size::new(Val::Px(200.0), Val::Px(40.0)),
//         margin: UiRect::all(Val::Px(10.0)),
//         justify_content: JustifyContent::Center,
//         align_items: AlignItems::Center,
//         ..default()
//     };
//     let button_text_style = TextStyle {
//         font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//         font_size: 20.0,
//         color: TEXT_COLOR,
//     };

//     commands
//         // This is a container, spans the whole screen?
//         .spawn((NodeBundle {
//             style: Style {
//                 size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
//                 align_items: AlignItems::Center,
//                 justify_content: JustifyContent::FlexStart,
//                 padding: UiRect::all(Val::Px(10.0)),
//                 ..default()
//             },
//             ..default()
//         },))
//         .with_children(|parent| {
//             parent
//                 .spawn(NodeBundle {
//                     style: Style {
//                         flex_direction: FlexDirection::Column,
//                         align_items: AlignItems::FlexStart,
//                         ..default()
//                     },
//                     ..default()
//                 })
//                 .with_children(|parent| {
//                     parent
//                         // This is the area where the buttons are displayed
//                         .spawn(NodeBundle {
//                             style: Style {
//                                 flex_direction: FlexDirection::Column,
//                                 align_items: AlignItems::Center,
//                                 ..default()
//                             },
//                             background_color: Color::SEA_GREEN.into(),
//                             ..default()
//                         })
//                         .with_children(|parent| {
//                             // Each child here is an individual "button group"

//                             parent.spawn(TextBundle::from_section(
//                                 "Effects",
//                                 button_text_style.clone(),
//                             ));

//                             let mut list = listable.clone().into_iter().collect::<Vec<_>>();
//                             list.sort_unstable();

//                             // Display a button for each possible value
//                             for (index, component) in list.into_iter().enumerate() {
//                                 parent
//                                     .spawn(NodeBundle {
//                                         style: Style {
//                                             flex_direction: FlexDirection::Row,
//                                             align_items: AlignItems::Center,
//                                             ..default()
//                                         },
//                                         ..default()
//                                     })
//                                     .with_children(|parent| {
//                                         // The button
//                                         let mut entity = parent.spawn(ButtonBundle {
//                                             style: Style {
//                                                 size: Size::new(Val::Px(150.0), Val::Px(65.0)),
//                                                 ..button_style.clone()
//                                             },
//                                             background_color: NORMAL_BUTTON.into(),
//                                             ..default()
//                                         });
//                                         entity.insert(component.clone()).with_children(|parent| {
//                                             parent.spawn(TextBundle::from_section(
//                                                 format!("{component}"),
//                                                 button_text_style.clone(),
//                                             ));
//                                         });
//                                         if index == 0 {
//                                             entity.insert(SelectedOption);
//                                         }
//                                         // let button_id = entity.id();

//                                         parent
//                                             .spawn(NodeBundle {
//                                                 style: Style {
//                                                     flex_direction: FlexDirection::Column,
//                                                     align_items: AlignItems::Center,
//                                                     ..default()
//                                                 },
//                                                 ..default()
//                                             })
//                                             .with_children(|parent| {
//                                                 // The up arrow
//                                                 let mut child_entity = parent.spawn((
//                                                     ButtonBundle {
//                                                         style: Style {
//                                                             size: Size::new(
//                                                                 Val::Px(20.0),
//                                                                 Val::Px(20.0),
//                                                             ),
//                                                             ..button_style.clone()
//                                                         },
//                                                         background_color: NORMAL_BUTTON.into(),
//                                                         ..default()
//                                                     },
//                                                     ButtonArrow {
//                                                         direction: ButtonArrowDirection::Up,
//                                                         related_setting: button_id,
//                                                     },
//                                                 ));
//                                                 child_entity
//                                                     // .insert(component.clone())
//                                                     .with_children(|parent| {
//                                                         parent.spawn(TextBundle::from_section(
//                                                             "↑",
//                                                             button_text_style.clone(),
//                                                         ));
//                                                     });

//                                                 // The down arrow
//                                                 let mut child_entity = parent.spawn((
//                                                     ButtonBundle {
//                                                         style: Style {
//                                                             size: Size::new(
//                                                                 Val::Px(20.0),
//                                                                 Val::Px(20.0),
//                                                             ),
//                                                             ..button_style.clone()
//                                                         },
//                                                         background_color: NORMAL_BUTTON.into(),
//                                                         ..default()
//                                                     },
//                                                     ButtonArrow {
//                                                         direction: ButtonArrowDirection::Down,
//                                                         related_setting: button_id,
//                                                     },
//                                                 ));
//                                                 child_entity
//                                                     // .insert(component.clone())
//                                                     .with_children(|parent| {
//                                                         parent.spawn(TextBundle::from_section(
//                                                             "↓",
//                                                             button_text_style.clone(),
//                                                         ));
//                                                     });
//                                             });
//                                     });
//                             }
//                         });
//                 });
//         });
// }

////////////////////////////////////////////////////////////////////////////////
// MAIN
////////////////////////////////////////////////////////////////////////////////
#[allow(dead_code)]
fn main() {
    println!("Not an example, just shared code between examples")
}

#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{CreateWindow, PresentMode, WindowId},
};

use bevy_vfx_bag::post_processing2::v3::{
    PixelateSettings, PostProcessingPlugin, RaindropsSettings, VfxOrdering,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PostProcessingPlugin {})
        .insert_resource(Effects(["Pixelate", "Raindrops"]))
        .add_startup_system(startup)
        .add_system(change_selection)
        .add_system(update_order::<Window1>)
        .add_system(update_order::<Window2>);

    // let s = bevy_mod_debugdump::get_render_schedule(&mut app);
    // let mut f = std::fs::File::create("pixelate-render-schedule.dot").unwrap();
    // f.write_all(s.as_bytes()).unwrap();

    // let s = bevy_mod_debugdump::get_render_graph(&mut app);
    // let mut f = std::fs::File::create("pixelate-render-graph.dot").unwrap();
    // f.write_all(s.as_bytes()).unwrap();

    app.run();
}

#[derive(Component, Clone, Copy)]
struct Window1;

#[derive(Component, Clone, Copy)]
struct Window2;

fn startup(
    mut commands: Commands,
    mut create_window_events: EventWriter<CreateWindow>,
    asset_server: Res<AssetServer>,
    effects: Res<Effects>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.0, 12., 10.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        PixelateSettings::default(),
        RaindropsSettings::default(),
        VfxOrdering::<RaindropsSettings>::new(0.0),
        VfxOrdering::<PixelateSettings>::new(0.0),
        Window1,
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
        PixelateSettings::default(),
        RaindropsSettings::default(),
        // TODO: Insert defaults of these
        VfxOrdering::<RaindropsSettings>::new(0.0),
        VfxOrdering::<PixelateSettings>::new(0.0),
        Window2,
    ));

    let make_list = || {
        TextBundle::from_sections(effects.0.iter().map(|&s| TextSection {
            value: format!(" {s} (on)\n"),
            style: TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
            },
        }))
        .with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        })
    };

    commands
        .spawn((NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn(make_list()).insert((
                Selection::default(),
                Window1,
                WindowRelation(WindowId::primary()),
            ));
            parent.spawn(make_list()).insert((
                Selection::default(),
                Window2,
                WindowRelation(window_id),
            ));
        });
}

#[derive(Component)]
struct WindowRelation(WindowId);

#[derive(Debug, Component)]
enum Change {
    Up,
    Down,
    Select,
    Toggle,
}

fn change_selection(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &WindowRelation), With<Text>>,
    windows: Res<Windows>,
) {
    for window in windows.iter() {
        if window.is_focused() {
            for (entity, window_id) in query.iter_mut() {
                if window_id.0 == window.id() {
                    if keyboard_input.just_pressed(KeyCode::Space) {
                        commands.entity(entity).insert(Change::Select);
                    }
                    if keyboard_input.just_pressed(KeyCode::Up) {
                        commands.entity(entity).insert(Change::Up);
                    } else if keyboard_input.just_pressed(KeyCode::Down) {
                        commands.entity(entity).insert(Change::Down);
                    } else if keyboard_input.just_pressed(KeyCode::T) {
                        commands.entity(entity).insert(Change::Toggle);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Selection {
    is_selected: bool,
    line_pointed_to: usize,
}

#[derive(Resource)]
struct Effects([&'static str; 2]);

#[allow(clippy::type_complexity)]
fn update_order<W: Component>(
    mut commands: Commands,
    mut selection: Local<Selection>,
    mut text: Query<(Entity, &mut Text, &Change), (With<W>, Added<Change>)>,
    mut pixelate: Query<&mut VfxOrdering<PixelateSettings>, With<W>>,
    mut raindrops: Query<&mut VfxOrdering<RaindropsSettings>, With<W>>,

    pixelate_settings: Query<(Entity, Option<&PixelateSettings>), (With<W>, With<Camera>)>,
    raindrops_settings: Query<(Entity, Option<&RaindropsSettings>), (With<W>, With<Camera>)>,
) {
    let (entity, mut text, change) = match text.get_single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    commands.entity(entity).remove::<Change>();

    let previous_index = selection.line_pointed_to;
    let mut should_toggle = false;

    match change {
        Change::Up => selection.line_pointed_to = selection.line_pointed_to.saturating_sub(1),
        Change::Down => {
            selection.line_pointed_to = (selection.line_pointed_to + 1).min(text.sections.len() - 1)
        }
        Change::Select => selection.is_selected = !selection.is_selected,
        Change::Toggle => should_toggle = true,
    }

    if previous_index != selection.line_pointed_to && selection.is_selected {
        let sections = &mut text.sections;
        sections.swap(selection.line_pointed_to, previous_index);
    }

    let mut priority = 0.0;

    for (index, section) in text.sections.iter_mut().enumerate() {
        section.value = if selection.line_pointed_to == index {
            ">".to_string() + &section.value[1..]
        } else {
            " ".to_string() + &section.value[1..]
        };

        let name = &section.value.as_str()[1..];
        info!("Splitting {name}");
        let name = name.rsplit_once(" (").unwrap().0;

        match name {
            "Pixelate" => {
                pixelate.single_mut().priority = priority;

                if index == selection.line_pointed_to {
                    for (entity, maybe_settings) in pixelate_settings.iter() {
                        if should_toggle {
                            if maybe_settings.is_some() {
                                commands.entity(entity).remove::<PixelateSettings>();
                                info!("Doing a replace of {} (after remove)", section.value);
                                section.value = section.value.replace("(on)", "(off)");
                            } else {
                                commands.entity(entity).insert(PixelateSettings::default());
                                info!("Doing a replace of {} (after insert)", section.value);
                                section.value = section.value.replace("(off)", "(on)");
                            }
                        }
                    }
                }
            }
            "Raindrops" => {
                raindrops.single_mut().priority = priority;

                if index == selection.line_pointed_to {
                    for (entity, maybe_settings) in raindrops_settings.iter() {
                        if should_toggle {
                            if maybe_settings.is_some() {
                                commands.entity(entity).remove::<RaindropsSettings>();
                                section.value = section.value.replace("(on)", "(off)");
                            } else {
                                commands.entity(entity).insert(RaindropsSettings::default());
                                section.value = section.value.replace("(off)", "(on)");
                            }
                        }
                    }
                }
            }
            others => panic!("Name is {others}"),
        }

        section.style.color = if selection.line_pointed_to == index && selection.is_selected {
            Color::GOLD
        } else if section.value.contains("(off)") {
            Color::GRAY
        } else {
            Color::WHITE
        };

        priority += 1.;
    }
}

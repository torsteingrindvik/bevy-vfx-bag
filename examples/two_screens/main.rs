#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{WindowRef, WindowResolution},
};

use bevy_vfx_bag::post_processing2::v3::{
    blur::BlurSettings, chromatic_aberration::ChromaticAberrationSettings, flip::FlipSettings,
    lut::LutSettings, masks::MaskSettings, pixelate::PixelateSettings,
    raindrops::RaindropsSettings, PostProcessingPlugin, VfxOrdering,
};

const NUM_EFFECTS: usize = 7;

#[derive(Resource, Deref)]
struct Effects([&'static str; NUM_EFFECTS]);

impl Effects {
    fn insert_default(name: &str, priority: f32, entity: Entity, commands: &mut Commands) {
        if name == "Pixelate" {
            commands.entity(entity).insert((
                PixelateSettings::default(),
                VfxOrdering::<PixelateSettings>::new(priority),
            ));
        } else if name == "Raindrops" {
            commands.entity(entity).insert((
                RaindropsSettings::default(),
                VfxOrdering::<RaindropsSettings>::new(priority),
            ));
        } else if name == "Flip" {
            commands.entity(entity).insert((
                FlipSettings::default(),
                VfxOrdering::<FlipSettings>::new(priority),
            ));
        } else if name == "Mask" {
            commands.entity(entity).insert((
                MaskSettings::default(),
                VfxOrdering::<MaskSettings>::new(priority),
            ));
        } else if name == "Lut" {
            commands.entity(entity).insert((
                LutSettings::default(),
                VfxOrdering::<LutSettings>::new(priority),
            ));
        } else if name == "Blur" {
            commands.entity(entity).insert((
                BlurSettings::default(),
                VfxOrdering::<BlurSettings>::new(priority),
            ));
        } else if name == "ChromaticAberration" {
            commands.entity(entity).insert((
                ChromaticAberrationSettings::default(),
                VfxOrdering::<ChromaticAberrationSettings>::new(priority),
            ));
        } else {
            panic!("Unknown effect name");
        }
    }

    fn remove(name: &str, entity: Entity, commands: &mut Commands) {
        if name == "Pixelate" {
            commands.entity(entity).remove::<PixelateSettings>();
        } else if name == "Raindrops" {
            commands.entity(entity).remove::<RaindropsSettings>();
        } else if name == "Flip" {
            commands.entity(entity).remove::<FlipSettings>();
        } else if name == "Mask" {
            commands.entity(entity).remove::<MaskSettings>();
        } else if name == "Lut" {
            commands.entity(entity).remove::<LutSettings>();
        } else if name == "Blur" {
            commands.entity(entity).remove::<BlurSettings>();
        } else if name == "ChromaticAberration" {
            commands
                .entity(entity)
                .remove::<ChromaticAberrationSettings>();
        } else {
            panic!("Unknown effect name");
        }
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PostProcessingPlugin {})
        .insert_resource(Effects([
            "Pixelate",
            "Raindrops",
            "Flip",
            "Mask",
            "Lut",
            "Blur",
            "ChromaticAberration",
        ]))
        .add_startup_system(setup)
        .add_system(change_selection)
        .add_system(update_text)
        .add_system(update_effects::<Window1>)
        .add_system(update_effects::<Window2>);
    // .add_system(update_effects_order);

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, effects: Res<Effects>) {
    let vfx_bundle = (
        PixelateSettings::default(),
        RaindropsSettings::default(),
        FlipSettings::default(),
        MaskSettings::default(),
        LutSettings::default(),
        BlurSettings::default(),
        ChromaticAberrationSettings::default(),
    );

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.0, 12., 10.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        vfx_bundle.clone(),
        Window1,
    ));

    let window_id = commands
        .spawn(Window {
            resolution: WindowResolution::new(0.4, 0.4),
            ..default()
        })
        .id();

    // second window camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.0, 12., 10.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(window_id)),
                ..default()
            },
            ..default()
        },
        vfx_bundle,
        Window2,
    ));

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
            let vfx = examples_common::List::new(
                asset_server.load("fonts/FiraSans-Bold.ttf"),
                effects.0.iter(),
            );
            let tb = vfx.as_text_bundle();

            parent
                .spawn(tb.clone())
                .insert((vfx.clone(), Window1, WindowRelation(window_id)));
            parent
                .spawn(tb)
                .insert((vfx, Window2, WindowRelation(window_id)));
        });
}

#[derive(Component, Deref)]
struct WindowRelation(Entity);

fn change_selection(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&WindowRelation, &mut examples_common::List)>,
    windows: Query<(Entity, &Window)>,
) {
    for (entity, window) in windows.iter() {
        if window.focused {
            for (window_id, mut list) in query.iter_mut() {
                if **window_id == entity {
                    if keyboard_input.just_pressed(KeyCode::Space) {
                        list.toggle_selected();
                    } else if keyboard_input.just_pressed(KeyCode::Up) {
                        list.up();
                    } else if keyboard_input.just_pressed(KeyCode::Down) {
                        list.down();
                    } else if keyboard_input.just_pressed(KeyCode::T) {
                        list.toggle_enabled();
                    }
                }
            }
        }
    }
}

fn update_text(mut query: Query<(&mut Text, &examples_common::List)>) {
    for (mut text, list) in query.iter_mut() {
        *text = list.as_text();
    }
}

fn update_effects<W: Component>(
    mut commands: Commands,
    window_list: Query<&examples_common::List, (With<W>, Changed<examples_common::List>)>,
    window_camera: Query<Entity, (With<W>, With<Camera>)>,
) {
    let mut priority = 0.0;

    let mut update = |entity, list: &examples_common::List| {
        for effect in list.enabled_items() {
            Effects::insert_default(effect, priority, entity, &mut commands);
            priority += 1.0;
        }
        for effect in list.disabled_items() {
            Effects::remove(effect, entity, &mut commands);
        }
    };

    if let Ok(list) = window_list.get_single() {
        let entity = window_camera.single();
        update(entity, list);
    }
}

#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{prelude::*, utils::HashMap};

use bevy_vfx_bag::{
    image::lut::{Lut, Lut3d, LutPlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

// Load the LUT presets from disk via the asset server,
// and give them a readable name.
// When the event fires that the image is actually loaded,
// move it into the vector of LUTs ready for use.
#[derive(Debug, Resource, Default)]
struct LutsState {
    handles: HashMap<Handle<Image>, &'static str>,
    ready: Vec<(&'static str, Lut3d)>,
}

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(LutPlugin)
        .init_resource::<LutsState>()
        .add_startup_system(startup)
        .add_system(make_luts)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands, assets: Res<AssetServer>, mut luts: ResMut<LutsState>) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(PostProcessingInput);

    *luts = LutsState {
        handles: HashMap::from_iter(vec![
            (assets.load("luts/neutral.png"), "Neutral"),
            (assets.load("luts/burlesque.png"), "Burlesque"),
            (assets.load("luts/neo.png"), "Neo"),
            (assets.load("luts/rouge.png"), "Rouge"),
            (assets.load("luts/sauna.png"), "Sauna"),
            (assets.load("luts/slate.png"), "Slate"),
            (assets.load("luts/arctic.png"), "Arctic"),
            (assets.load("luts/denim.png"), "Denim"),
        ]),
        ready: vec![],
    };
}

// Move loaded LUT images into the "ready" state by
// creating them as 3D LUTs.
fn make_luts(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    mut luts: ResMut<LutsState>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            if let Some(lut_name) = luts.handles.remove(handle) {
                luts.ready
                    .push((lut_name, Lut3d::from_image(&mut assets, handle)));
            }
        }
    }
}

// Cycle through some preset LUTs.
fn update(
    time: Res<Time>,
    mut choice: Local<usize>,
    mut lut: ResMut<Lut>,
    mut text: ResMut<examples_common::ExampleText>,
    luts_state: Res<LutsState>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let num_luts = luts_state.ready.len();
    if num_luts == 0 {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        lut.split_vertically = !lut.split_vertically;
    }

    let choice_now = time.elapsed_seconds() as usize % num_luts;

    let (name, lut3d) = &luts_state.ready[choice_now];

    if *choice != choice_now {
        *choice = choice_now;
        lut.set_texture(lut3d);
    }

    text.0 = format!(
        "LUT {}/{num_luts}: {name}, [S]plit: {:?}",
        *choice + 1,
        lut.split_vertically,
    );
}

#![allow(clippy::too_many_arguments)] // Bevy fns tend to have many args.
#![deny(clippy::unwrap_used)] // Let's try to explain invariants when we unwrap (so use expect).
#![deny(missing_docs)] // Let's try to have good habits.
#![doc = include_str!("../README.md")]

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::{App, *},
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResized,
};

use crate::quad::window_sized_quad;

/// Effects which are added to an image.
/// This image might be the output of a render pass of an app.
pub mod image;

/// Helpers for making quads.
pub mod quad;

/// For post processing effects to work, this marker should be added to a camera.
/// This camera will be changed to render to an image buffer which will then be applied
/// post processing to.
/// Note that UI will be disabled for the marked camera, and applied _after_ effects are added.
#[derive(Debug, Clone, Copy, Component)]
pub struct PostProcessingInput;

#[derive(Debug, Clone, Component)]
struct PostProcessing2dCamera(Handle<Image>);

/// This resource holds the image handles of the images which will be used for
/// sampling before applying effects.
#[derive(Debug, Resource)]
struct BevyVfxBagState {
    // The two image buffers in use.
    image_handle_a: Handle<Image>,
    image_handle_b: Handle<Image>,

    // The size of the above images (they are equally sized).
    extent: Extent3d,

    // If the next effect should use image A as input.
    image_a_next_input: bool,

    // Last used priority (used by effect cameras).
    priority: isize,
}

impl BevyVfxBagState {
    fn next_image_io_handles(&mut self) -> (Handle<Image>, Handle<Image>) {
        let (i, o) = if self.image_a_next_input {
            (
                self.image_handle_a.clone_weak(),
                self.image_handle_b.clone_weak(),
            )
        } else {
            (
                self.image_handle_b.clone_weak(),
                self.image_handle_a.clone_weak(),
            )
        };

        self.image_a_next_input = !self.image_a_next_input;

        (i, o)
    }

    fn next_priority(&mut self) -> isize {
        self.priority += 1;
        self.priority
    }
}

#[derive(Debug, Component)]
pub(crate) struct ShouldResize;

fn make_image(width: u32, height: u32, name: &'static str) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    image.resize(size);
    image
}

impl FromWorld for BevyVfxBagState {
    fn from_world(world: &mut World) -> Self {
        let (width, height) = {
            let windows = world.resource::<Windows>();
            let primary = windows.get_primary().expect("Should have primary window");

            (primary.physical_width(), primary.physical_height())
        };

        let mut image_assets = world.resource_mut::<Assets<Image>>();

        let image_a = make_image(width, height, "BevyVfxBag Image A");
        let image_b = make_image(width, height, "BevyVfxBag Image B");

        let image_handle_a = image_assets.add(image_a);
        let image_handle_b = image_assets.add(image_b);

        let extent = Extent3d {
            width,
            height,
            ..default()
        };

        Self {
            image_handle_a,
            image_handle_b,
            extent,

            priority: 0,

            // Yes, the first effect should use image A for reading/input.
            image_a_next_input: true,
        }
    }
}

/// The base plugin which sets up base resources and
/// systems which other plugins in this crate rely on.
pub struct BevyVfxBagPlugin;

fn new_effect_state(world: &mut World) -> EffectState {
    let mut state = world
        .get_resource_mut::<BevyVfxBagState>()
        .expect("Should exist");

    let priority = state.next_priority();

    let (input_image_handle, output_image_handle) = state.next_image_io_handles();

    EffectState {
        priority,
        render_layers: RenderLayers::layer(priority as u8),
        input_image_handle,
        output_image_handle,
    }
}

#[derive(Debug, Clone)]
struct EffectState {
    priority: isize,
    render_layers: RenderLayers,
    input_image_handle: Handle<Image>,
    output_image_handle: Handle<Image>,
}

trait HasEffectState {
    fn state(&self) -> EffectState;
}

pub(crate) fn setup_effect<M>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    bvb_image: Res<BevyVfxBagState>,
    mut effect_materials: ResMut<Assets<M>>,
    effect_material: Res<M>,
) where
    M: Material2d + FromWorld + Resource + HasEffectState,
{
    let effect_state = effect_material.state();

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                priority: effect_state.priority,
                ..default()
            },
            ..default()
        },
        effect_state.render_layers,
        PostProcessing2dCamera(effect_state.output_image_handle.clone_weak()),
    ));

    let extent = bvb_image.extent;

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        extent.width as f32,
        extent.height as f32,
    ))));

    let material_handle = effect_materials.add(effect_material.clone());

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        effect_state.render_layers,
        ShouldResize,
    ));
}

fn update(
    windows: Res<Windows>,
    mut resize_reader: EventReader<WindowResized>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    needs_new_quad: Query<&Mesh2dHandle, With<ShouldResize>>,
) {
    let main_window_id = windows.get_primary().expect("Should have window").id();

    for event in resize_reader.iter() {
        if event.id != main_window_id {
            continue;
        }

        let window = windows.get(event.id).expect("Main window should exist");

        for resize_me in &needs_new_quad {
            let mesh = window_sized_quad(window);

            *mesh_assets.get_mut(&resize_me.0).expect("Should find mesh") = mesh;
        }
    }
}

fn setup_post_processing_input(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Camera), With<PostProcessingInput>>,
    bvb_image: Res<BevyVfxBagState>,
) {
    let (entity, mut camera) = query.single_mut();

    // The camera that wants to be post processed must render to our first image buffer.
    camera.target = RenderTarget::Image(bvb_image.image_handle_a.clone_weak());

    // We apply post process effects before UI is shown, so turn it off for now.
    commands
        .entity(entity)
        .insert(UiCameraConfig { show_ui: false });
}

fn setup_post_processing_2d_cameras(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Camera, &mut Camera2d, &PostProcessing2dCamera)>,
) {
    let mut cameras_2d = query.iter_mut().collect::<Vec<_>>();

    // Sort cameras such that they are in priority order.
    // This will be the same order effect plugins were added.
    cameras_2d.sort_unstable_by_key(|(_, camera, _, _)| camera.priority);

    // If we chain effects, the effect will be wiped out of we clear the
    // next render pass. So don't.
    if let Some((_, except_first)) = cameras_2d.split_first_mut() {
        for (_, _, camera_2d, _) in except_first {
            camera_2d.clear_color = ClearColorConfig::None;
        }
    }

    if let Some((_, except_last)) = cameras_2d.split_last_mut() {
        for (e, camera, _, PostProcessing2dCamera(image_handle)) in except_last.iter_mut() {
            // The UI should only be rendered at the very end.
            commands
                .entity(*e)
                .insert(UiCameraConfig { show_ui: false });

            // 2D cameras should write to a specified image handle.
            // The last should not- it will render to screen.
            camera.target = RenderTarget::Image(image_handle.clone_weak());
        }
    }
}

impl Plugin for BevyVfxBagPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyVfxBagState>()
            .add_startup_system_to_stage(StartupStage::PostStartup, setup_post_processing_input)
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                setup_post_processing_2d_cameras,
            )
            .add_system(update);
    }
}

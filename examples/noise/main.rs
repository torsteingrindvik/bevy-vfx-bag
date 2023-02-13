#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use bevy_vfx_bag::BevyVfxBagPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(BevyVfxBagPlugin::default())
        .add_plugin(MaterialPlugin::<ValueNoise>::default())
        .add_plugin(MaterialPlugin::<Fbm>::default())
        .add_startup_system(startup)
        .add_system(update_value_noise)
        .add_system(update_fbm)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut noise_materials: ResMut<Assets<ValueNoise>>,
    mut fbm_materials: ResMut<Assets<Fbm>>,
) {
    // noise cubes
    for i in 1..=4 {
        let i = i as f32;
        let uv = Uv {
            scale: i * 4.,
            ..default()
        };

        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_xyz(-4. + (i * 2.), 0.5, 0.0),
            material: noise_materials.add(ValueNoise { uv }),
            ..default()
        });
    }

    // fbm cubes
    for i in 1..=4 {
        let i = i as f32;
        let uv = Uv {
            scale: i * 4.,
            ..default()
        };

        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_xyz(-4. + (i * 2.), -1.0, 0.0),
            material: fbm_materials.add(Fbm { uv }),
            ..default()
        });
    }

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-0.5, 0.5, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn update_value_noise(
    time: Res<Time>,
    query: Query<&Handle<ValueNoise>>,
    mut custom_material_assets: ResMut<Assets<ValueNoise>>,
) {
    for handle in query.iter() {
        if let Some(custom_material) = custom_material_assets.get_mut(handle) {
            let t = time.delta_seconds();
            custom_material.uv.offset_x += 3. * t;
            custom_material.uv.offset_y += t;
        }
    }
}

fn update_fbm(
    time: Res<Time>,
    query: Query<&Handle<Fbm>>,
    mut custom_material_assets: ResMut<Assets<Fbm>>,
) {
    for handle in query.iter() {
        if let Some(custom_material) = custom_material_assets.get_mut(handle) {
            let t = time.delta_seconds();
            custom_material.uv.offset_x += 3. * t;
            custom_material.uv.offset_y += t;
        }
    }
}

#[derive(Debug, ShaderType, Clone)]
struct Uv {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl Default for Uv {
    fn default() -> Self {
        Self {
            scale: 1.,
            offset_x: 0.,
            offset_y: 0.,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "9dc460be-ab02-11ed-905b-325096b39f47"]
pub struct ValueNoise {
    #[uniform(0)]
    uv: Uv,
}

impl Material for ValueNoise {
    fn fragment_shader() -> ShaderRef {
        "materials/value_noise.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "89801a24-abc2-11ed-8f97-325096b39f47"]
pub struct Fbm {
    #[uniform(0)]
    uv: Uv,
}

impl Material for Fbm {
    fn fragment_shader() -> ShaderRef {
        "materials/fbm.wgsl".into()
    }
}

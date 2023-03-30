#[path = "../examples_common.rs"]
mod examples_common;

// TODO: https://github.com/bevyengine/bevy/issues/6754
// Try bloom UI

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::{shape::Quad, *},
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderDefVal, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

fn main() {
    App::new()
        .add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::AnimatedFoxExamplePlugin::default())
        .add_plugin(Material2dPlugin::<HeartMaterial>::default())
        .add_startup_system(setup)
        .add_system(update_mouse)
        .add_system(update_heart_materials)
        .add_system(update_keyboard.before(update_heart_materials))
        .add_system(update_heart_quads.after(update_heart_materials))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HeartMaterial>>,
) {
    // let size = 280.;
    let size = 120.;

    for (idx, (color, variant)) in [
        (Color::RED, HeartMaterialKey::Heart),
        (Color::BLUE, HeartMaterialKey::Heart),
        (Color::BLACK, HeartMaterialKey::Ball),
        (Color::PINK, HeartMaterialKey::Bone),
        (Color::WHITE, HeartMaterialKey::Bone),
        (Color::LIME_GREEN, HeartMaterialKey::Hat),
        (Color::TURQUOISE, HeartMaterialKey::Hat),
        (Color::AZURE, HeartMaterialKey::Undecided),
    ]
    .iter()
    .enumerate()
    {
        commands.spawn(MaterialMesh2dBundle {
            material: materials.add(HeartMaterial::new(
                *color,
                size,
                6,
                Vec2::new(5., -5. - size * (idx as f32 * 0.8)),
                *variant,
            )),
            mesh: Mesh2dHandle(meshes.add(Quad::default().into())),
            ..default()
        });
    }

    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });
}

impl Material2d for HeartMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/heart.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let frag = descriptor.fragment.as_mut().unwrap();

        let key = key.bind_group_data;
        frag.shader_defs.push(key.shader_def());

        Ok(())
    }
}

#[derive(Debug, ShaderType, Clone, Default, Copy, Component)]
struct HeartData {
    // fade in/out
    opacity: f32,

    // scale in/out
    scale: f32,

    // spin in/out
    angle: f32,

    _padding: f32,
}

impl HeartData {
    fn transition_to_percentage(style: TransitionStyle, percentage: f32) -> Self {
        match style {
            TransitionStyle::Instant => Self {
                opacity: percentage,
                scale: percentage,
                angle: percentage,
                ..default()
            },
            TransitionStyle::Fade => Self {
                opacity: percentage,
                scale: 1.0,
                angle: 1.0,
                ..default()
            },
            TransitionStyle::Scale => Self {
                opacity: 1.0,
                scale: percentage,
                angle: 1.0,
                ..default()
            },
            TransitionStyle::Spin => Self {
                opacity: 1.0,
                scale: 1.0,
                angle: percentage,
                ..default()
            },
        }
    }
}

#[derive(Debug, Clone, Default, Copy)]
struct HeartSettings {
    // Todo: Make stack?
    transition: Option<Transition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeartMaterialKey {
    Heart,
    Ball,
    Bone,
    Hat,
    Undecided,
}

impl HeartMaterialKey {
    fn shader_def(&self) -> ShaderDefVal {
        match self {
            HeartMaterialKey::Heart => "BVB_UI_HEART",
            HeartMaterialKey::Ball => "BVB_UI_BALL",
            HeartMaterialKey::Bone => "BVB_UI_BONE",
            HeartMaterialKey::Hat => "BVB_UI_HAT",
            HeartMaterialKey::Undecided => "BVB_UI_UNDECIDED",
        }
        .into()
    }
}

impl<const N: usize> From<&HeartMaterial<N>> for HeartMaterialKey {
    fn from(material: &HeartMaterial<N>) -> Self {
        material.variant
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "ff664fca-c02f-11ed-bf9f-325096b39f47"]
#[bind_group_data(HeartMaterialKey)]
pub struct HeartMaterial<const N: usize = 32> {
    #[uniform(0)]
    color: Color,

    // how many hearts there should be room for in the quad
    #[uniform(1)]
    active_hearts: f32,

    // how many hearts there are logically
    target_num_hearts: usize,

    #[uniform(2)]
    mouse: Vec2,

    // TODO: Conditionally use storage buffers
    #[uniform(3)]
    hearts: [HeartData; N],

    heart_settings: [HeartSettings; N],

    size: f32,

    // screen space position, anchored at quad's top left corner
    position: Vec2,

    variant: HeartMaterialKey,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TransitionStyle {
    #[default]
    Instant,

    Fade,
    Scale,
    Spin,
}

#[derive(Debug, Clone, Copy)]
pub struct TransitionSettings {
    speed: f32,
    style: TransitionStyle,
}

#[derive(Debug, Clone, Copy)]
struct Transition {
    transitioning_in: bool,
    transition_percentage: f32,

    settings: TransitionSettings,
}

impl Transition {
    fn is_done(&self) -> bool {
        if self.transitioning_in {
            self.transition_percentage >= 1.0
        } else {
            self.transition_percentage <= 0.0
        }
    }
}

impl<const N: usize> HeartMaterial<N> {
    pub fn new(
        color: Color,
        size: f32,
        num: usize,
        position: Vec2,
        variant: HeartMaterialKey,
    ) -> Self {
        assert!(num <= 32);

        let mut all_data = [Default::default(); N];

        for data in all_data.iter_mut().take(num) {
            *data = HeartData {
                opacity: 1.0,
                scale: 1.0,
                angle: 1.0,
                _padding: 0.0,
            }
        }

        Self {
            color,
            active_hearts: num as f32,
            target_num_hearts: num,
            mouse: Default::default(),
            hearts: all_data,
            heart_settings: [Default::default(); N],
            size,
            position,
            variant,
        }
    }

    pub fn new_hearts(color: Color, size: f32, num: usize, position: Vec2) -> Self {
        Self::new(color, size, num, position, HeartMaterialKey::Heart)
    }

    pub fn new_balls(color: Color, size: f32, num: usize, position: Vec2) -> Self {
        Self::new(color, size, num, position, HeartMaterialKey::Ball)
    }

    pub fn new_bones(color: Color, size: f32, num: usize, position: Vec2) -> Self {
        Self::new(color, size, num, position, HeartMaterialKey::Bone)
    }

    pub fn new_hat(color: Color, size: f32, num: usize, position: Vec2) -> Self {
        Self::new(color, size, num, position, HeartMaterialKey::Hat)
    }

    pub fn add(&mut self, settings: TransitionSettings) {
        let num = self.target_num_hearts;

        if num == 32 {
            eprintln!("Too many");
            return;
        }

        let heart_settings = &mut self.heart_settings[num];

        let previous_transition_percentage = heart_settings
            .transition
            .map(|hs| hs.transition_percentage)
            .unwrap_or(0.0);

        heart_settings.transition = Some(Transition {
            transitioning_in: true,
            transition_percentage: previous_transition_percentage,
            settings,
        });

        // *data = HeartData::transition_to_percentage(settings.style, 0.0);

        self.target_num_hearts += 1;
    }

    pub fn remove(&mut self, settings: TransitionSettings) {
        let num = self.target_num_hearts;

        if num == 0 {
            eprintln!("Too few");
            return;
        }

        self.target_num_hearts -= 1;

        // Example: num was 1, how many hearts we had active.
        // We want to change the transition on that element.
        // That heart's settings lives in index 0.
        // Therefore we subtract first, then index the settings.
        let heart_settings = &mut self.heart_settings[self.target_num_hearts];

        let previous_transition_percentage = heart_settings
            .transition
            .map(|hs| hs.transition_percentage)
            .unwrap_or(1.0);

        heart_settings.transition = Some(Transition {
            transitioning_in: false,
            transition_percentage: previous_transition_percentage,
            settings,
        });
    }

    // The number of hearts, plus any hearts with active transitions.
    fn active_hearts(&self) -> usize {
        let n = self.target_num_hearts;

        self.heart_settings
            .iter()
            .skip(n)
            .take_while(|hs| hs.transition.is_some())
            .count()
            + n
    }
}

fn update_mouse(
    query: Query<&Handle<HeartMaterial>>,
    mut hearts: ResMut<Assets<HeartMaterial>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Query<&Window>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        for event in cursor_moved_events.iter() {
            let window = windows.single();

            for handle in query.iter() {
                if let Some(heart) = hearts.get_mut(handle) {
                    heart.mouse.x = event.position.x / window.resolution.width();
                    heart.mouse.y = event.position.y / window.resolution.height();
                }
            }
        }
    }
}

#[derive(Debug, Deref)]
struct Ease(CubicSegment<Vec2>);

impl Default for Ease {
    fn default() -> Self {
        // https://docs.rs/bevy/latest/bevy/math/cubic_splines/struct.CubicSegment.html#method.new_bezier
        Self(CubicSegment::new_bezier((0.25, 0.1), (0.25, 1.0)))
    }
}

fn update_heart_materials(
    query: Query<&Handle<HeartMaterial>>,
    mut hearts: ResMut<Assets<HeartMaterial>>,
    curve: Local<Ease>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for handle in query.iter() {
        if let Some(heart_material) = hearts.get_mut(handle) {
            let hearts = &mut heart_material.hearts;
            let material_settings = &mut heart_material.heart_settings;

            for (index, heart_settings) in material_settings.iter_mut().enumerate() {
                if let Some(transition) = heart_settings.transition.as_mut() {
                    transition.transition_percentage = match transition.settings.style {
                        TransitionStyle::Instant => {
                            if transition.transitioning_in {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        _others => {
                            if transition.transitioning_in {
                                transition.transition_percentage + dt * transition.settings.speed
                            } else {
                                transition.transition_percentage - dt * transition.settings.speed
                            }
                        }
                    };

                    let ease = curve.ease(transition.transition_percentage);

                    hearts[index] =
                        HeartData::transition_to_percentage(transition.settings.style, ease);

                    if transition.is_done() {
                        let _ = heart_settings.transition.take();
                    }
                }
            }

            heart_material.active_hearts = heart_material.active_hearts() as f32;
        }
    }
}

fn update_heart_quads(
    // TODO: Changed
    mut query: Query<(&Handle<HeartMaterial>, &Mesh2dHandle, &mut Transform)>,
    hearts: Res<Assets<HeartMaterial>>,
    mut quads: ResMut<Assets<Mesh>>,
    windows: Query<&Window>,
) {
    for (heart_handle, mesh_handle, mut transform) in query.iter_mut() {
        if let Some(heart_material) = hearts.get(heart_handle) {
            let mesh = quads.get_mut(&mesh_handle.0).unwrap();

            let size = Vec2::new(
                heart_material.size * heart_material.active_hearts() as f32,
                heart_material.size,
            );
            let half = size / 2.;

            *mesh = shape::Quad::new(size).into();

            let window = windows.single();

            let top_left = Vec2::new(
                // positive right
                -(window.physical_width() as f32 / 2.),
                // positive up
                window.physical_height() as f32 / 2.,
            );

            let pos = top_left + Vec2::new(half.x, -half.y) + heart_material.position;

            *transform = Transform::from_xyz(pos.x, pos.y, 0.0);
        }
    }
}

fn update_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<HeartMaterial>>,
    mut hearts: ResMut<Assets<HeartMaterial>>,
) {
    for heart_handle in query.iter() {
        let heart_material = hearts.get_mut(heart_handle).unwrap();
        let speed = 0.8;

        if keyboard_input.just_pressed(KeyCode::Q) {
            heart_material.add(TransitionSettings {
                speed,
                style: TransitionStyle::Instant,
            });
        } else if keyboard_input.just_pressed(KeyCode::W) {
            heart_material.add(TransitionSettings {
                speed,
                style: TransitionStyle::Fade,
            });
        } else if keyboard_input.just_pressed(KeyCode::E) {
            heart_material.add(TransitionSettings {
                speed,
                style: TransitionStyle::Scale,
            });
        } else if keyboard_input.just_pressed(KeyCode::R) {
            heart_material.add(TransitionSettings {
                speed,
                style: TransitionStyle::Spin,
            });
        } else if keyboard_input.just_pressed(KeyCode::A) {
            heart_material.remove(TransitionSettings {
                speed,
                style: TransitionStyle::Instant,
            });
        } else if keyboard_input.just_pressed(KeyCode::S) {
            heart_material.remove(TransitionSettings {
                speed,
                style: TransitionStyle::Fade,
            });
        } else if keyboard_input.just_pressed(KeyCode::D) {
            heart_material.remove(TransitionSettings {
                speed,
                style: TransitionStyle::Scale,
            });
        } else if keyboard_input.just_pressed(KeyCode::F) {
            heart_material.remove(TransitionSettings {
                speed,
                style: TransitionStyle::Spin,
            });
        }
    }
}

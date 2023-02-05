use bevy::{
    asset::load_internal_asset,
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{lifetimeless::Read, SystemParamItem},
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
            BindingResource, BindingType, CachedRenderPipelineId, Extent3d, SamplerBindingType,
            ShaderStages, TextureDimension, TextureFormat, TextureSampleType,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageType},
        RenderStage,
    },
};

use super::{DrawPostProcessing, PostProcessingPhaseItem, SetTextureSamplerGlobals, VfxOrdering};

pub(crate) const LUT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3719875149378986812);

const LUT_ARCTIC_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 11514769687270273032);
const LUT_NEO_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 18411885151390434307);
const LUT_SLATE_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 8809687374954616573);

type DrawLut = (
    // The pipeline must be set in order to use the correct bind group,
    // access the correct shaders, and so on.
    SetItemPipeline,
    // Common to post processing items is that they all use the same
    // first bind group, which has the input texture (the scene) and
    // the sampler for that.
    SetTextureSamplerGlobals<0>,
    // Here we set the bind group for the effect.
    SetLutImage<1>,
    // Lastly we draw vertices.
    // This is simple for a post processing effect, since we just draw
    // a full screen triangle.
    DrawPostProcessing,
);

#[derive(Debug, Component)]
struct LutBindGroup {
    bind_group: BindGroup,
}

/// TODO
struct SetLutImage<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetLutImage<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<LutBindGroup>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        lut_bind_group: ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &lut_bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}

#[derive(Resource)]
pub(crate) struct LutData {
    pub pipeline_id: CachedRenderPipelineId,
    pub layout: BindGroupLayout,
}

impl FromWorld for LutData {
    fn from_world(world: &mut World) -> Self {
        let (layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "LUT",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            LUT_SHADER_HANDLE.typed(),
        );

        LutData {
            pipeline_id,
            layout,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            LUT_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "lut.wgsl"),
            Shader::from_wgsl
        );

        let mut assets = app.world.resource_mut::<Assets<_>>();

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/luts/",
                "neo.png"
            )),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            false,
        )
        .expect("Should load LUT successfully");
        assets.set_untracked(LUT_NEO_IMAGE_HANDLE, image);

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/luts/",
                "slate.png"
            )),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            false,
        )
        .expect("Should load LUT successfully");
        assets.set_untracked(LUT_SLATE_IMAGE_HANDLE, image);

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/luts/",
                "arctic.png"
            )),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            false,
        )
        .expect("Should load LUT successfully");
        assets.set_untracked(LUT_ARCTIC_IMAGE_HANDLE, image);

        // This puts the uniform into the render world.
        app.add_plugin(ExtractComponentPlugin::<Lut>::default())
            .add_system(adapt_image_for_lut_use);

        super::render_app(app)
            .add_system_to_stage(
                RenderStage::Extract,
                super::extract_post_processing_camera_phases::<Lut>,
            )
            .init_resource::<LutData>()
            .add_system_to_stage(RenderStage::Prepare, prepare)
            .add_system_to_stage(RenderStage::Queue, queue)
            .add_render_command::<PostProcessingPhaseItem, DrawLut>();
    }
}

/// Marks a [`Lut`] as ready to be used.
/// This means the [`Image`] has been adapted to be used as a LUT.
#[derive(Debug, Component)]
pub struct PreparedLut;

// TODO: If the LUT changes at runtime, the entity will _still_ have PreparedLut, but the underlying
// handle points to something without modified image.
//
// Will need to fix this to instead look at Changed<Lut> (?), remove PreparedLut, then... something.
fn adapt_image_for_lut_use(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
    luts: Query<(Entity, &Lut)>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            if let Some((e, _)) = luts.iter().find(|(_, lut)| lut.texture == *handle) {
                let image = assets
                    .get_mut(handle)
                    .expect("Handle should point to asset");

                // The LUT is a 3d texture. It has 64 layers, each of which is a 64x64 image.
                image.texture_descriptor.size = Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 64,
                };
                image.texture_descriptor.dimension = TextureDimension::D3;
                image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

                image.texture_view_descriptor = Some(TextureViewDescriptor {
                    label: Some("LUT Texture View"),
                    format: Some(TextureFormat::Rgba8Unorm),
                    dimension: Some(TextureViewDimension::D3),
                    ..default()
                });

                info!("LUT prepared for handle {:?}", *handle);
                commands.get_or_spawn(e).insert(PreparedLut);
            }
        }
    }
}

fn prepare(
    data: Res<LutData>,
    mut views: Query<
        (
            Entity,
            &mut RenderPhase<PostProcessingPhaseItem>,
            &VfxOrdering<Lut>,
        ),
        With<PreparedLut>,
    >,
    draw_functions: Res<DrawFunctions<PostProcessingPhaseItem>>,
) {
    for (entity, mut phase, order) in views.iter_mut() {
        let draw_function = draw_functions.read().id::<DrawLut>();

        phase.add(PostProcessingPhaseItem {
            entity,
            sort_key: order.clone().into(),
            draw_function,
            pipeline_id: data.pipeline_id,
        });
    }
}

fn queue(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    data: Res<LutData>,
    images: Res<RenderAssets<Image>>,
    luts: Query<(Entity, &Lut), With<PreparedLut>>,
) {
    for (entity, lut) in luts.iter() {
        if let Some(lut_image) = images.get(&lut.texture) {
            let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("LUT Uniform Bind Group"),
                layout: &data.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&lut_image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&lut_image.sampler),
                    },
                ],
            });

            commands
                .get_or_spawn(entity)
                .insert(LutBindGroup { bind_group });
        }
    }
}

/// TODO
#[derive(Debug, Component, Clone)]
pub struct Lut {
    /// The 3D look-up texture
    pub texture: Handle<Image>,
}

impl Lut {
    /// The arctic color scheme LUT.
    pub fn arctic() -> Self {
        Self {
            texture: LUT_ARCTIC_IMAGE_HANDLE.typed_weak(),
        }
    }

    /// The neo color scheme LUT.
    pub fn neo() -> Self {
        Self::default()
    }

    /// The slate color scheme LUT.
    pub fn slate() -> Self {
        Self {
            texture: LUT_SLATE_IMAGE_HANDLE.typed_weak(),
        }
    }
}

impl Default for Lut {
    fn default() -> Self {
        Self {
            texture: LUT_NEO_IMAGE_HANDLE.typed(),
        }
    }
}

impl ExtractComponent for Lut {
    type Query = (&'static Self, &'static Camera);
    type Filter = With<PreparedLut>;
    type Out = (Self, PreparedLut);

    fn extract_component((settings, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        if !camera.is_active {
            return None;
        }

        Some((settings.clone(), PreparedLut))
    }
}

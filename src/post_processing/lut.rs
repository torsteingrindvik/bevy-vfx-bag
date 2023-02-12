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
        RenderSet,
    },
};

use super::{DrawPostProcessing, Order, PostProcessingPhaseItem, SetTextureSamplerGlobals};

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
            .add_system(adapt_image_for_lut_use.in_base_set(CoreSet::PostUpdate));

        super::render_app(app)
            .add_system_to_schedule(
                ExtractSchedule,
                super::extract_post_processing_camera_phases::<Lut>,
            )
            .init_resource::<LutData>()
            .add_system(prepare.in_set(RenderSet::Prepare))
            .add_system(queue.in_set(RenderSet::Queue))
            .add_render_command::<PostProcessingPhaseItem, DrawLut>();
    }
}

fn adapt_image_for_lut_use(
    mut assets: ResMut<Assets<Image>>,
    mut luts: Query<&mut Lut, Changed<Lut>>,
) {
    for mut lut in luts.iter_mut() {
        if lut.prepared {
            continue;
        }

        let image = assets
            .get_mut(&lut.texture)
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

        debug!("LUT prepared for handle {:?}", lut.texture);
        lut.prepared = true;
    }
}

fn prepare(
    data: Res<LutData>,
    mut views: Query<
        (
            Entity,
            &mut RenderPhase<PostProcessingPhaseItem>,
            &Order<Lut>,
        ),
        With<Lut>,
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
    luts: Query<(Entity, &Lut)>,
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

/// A look-up texture. Maps colors to colors. Useful for colorschemes.
#[derive(Debug, Component, Clone)]
pub struct Lut {
    /// The 3D look-up texture
    texture: Handle<Image>,

    prepared: bool,
}

impl Lut {
    /// Creates a new LUT component.
    /// The image should be a 64x64x64 3D texture.
    /// See the `make-neutral-lut` example.
    pub fn new(texture: Handle<Image>) -> Self {
        Self {
            texture,
            prepared: false,
        }
    }

    /// The arctic color scheme LUT.
    pub fn arctic() -> Self {
        Self::new(LUT_ARCTIC_IMAGE_HANDLE.typed_weak())
    }

    /// The neo color scheme LUT.
    pub fn neo() -> Self {
        Self::default()
    }

    /// The slate color scheme LUT.
    pub fn slate() -> Self {
        Self::new(LUT_SLATE_IMAGE_HANDLE.typed_weak())
    }
}

impl Default for Lut {
    fn default() -> Self {
        Self::new(LUT_NEO_IMAGE_HANDLE.typed_weak())
    }
}

impl ExtractComponent for Lut {
    type Query = (&'static Self, &'static Camera);
    type Filter = ();
    type Out = Self;

    fn extract_component((lut, camera): QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        if !camera.is_active || !lut.prepared {
            return None;
        }

        Some(lut.clone())
    }
}

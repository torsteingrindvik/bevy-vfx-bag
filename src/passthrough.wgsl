#define_import_path bevy_vfx_bag::post_processing_passthrough

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    var uv = coords_to_viewport_uv(position.xy, view.viewport);

    #ifdef PASSTHROUGH

    return vec4<f32>(textureSample(texture, our_sampler, uv).rgb, 1.0);

    #else

    return fragment_impl(position, uv);

    #endif
}

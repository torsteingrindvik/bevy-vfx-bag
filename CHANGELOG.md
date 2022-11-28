# Changelog

## v0.2.0

This version is a major rework.

### External

TODO.

### Internal

- Now uses Bevy's render graph features
    - One effect is one node (technically two since we add one for 2D and one for 3D)
    - We go through the necessary steps to extract uniforms and make them available for shaders
    - We can have specialized pipelines for things such as camera HDR-ness
    - We created some custom generic plugins which can reduce render graph boilerplate by a huge amount
- Post processing effects now use the `fullscreen_vertex_shader`
- Shaders are less hacky, no need for the "passthrough" feature
- Effect settings are components with an `enabled` field
    - But since we extract things properly, removing the effect component disables the effect (TODO test this)
- Since we extract from cameras, multi camera settups are supported (e.g. on two windows)

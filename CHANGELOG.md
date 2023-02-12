# Changelog

## v0.2.0

This version is a major rework.

### External

- There is no need for separate plugins per effect
- Effects are now components, not resources
- Effects can now be enabled/disabled by adding/removing the effect component to/from cameras
- Effects are now per-camera (i.e. can have several independent effect stacks)
- Effect ordering may be changed at runtime
- No longer a need for adding a `PostProcessingInput` marker struct

### Internal

- Now uses Bevy's render graph features
- Post processing effects now use the `fullscreen_vertex_shader`
- Shaders are less hacky, no need for the "passthrough" feature

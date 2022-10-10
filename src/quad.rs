use bevy::prelude::*;

/// Given a window, return a mesh created by a
/// quad which covers its physical size.
pub(crate) fn window_sized_quad(window: &Window) -> Mesh {
    Mesh::from(shape::Quad::new(Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    )))
}

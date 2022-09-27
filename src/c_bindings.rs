use crate::{PathBuilder, OutputVertex, FillMode};

#[no_mangle]
pub extern "C" fn wgr_new_builder() -> *mut PathBuilder {
    let pb = PathBuilder::new();
    Box::into_raw(Box::new(pb))
}

#[no_mangle]
pub extern "C" fn wgr_builder_move_to(pb: &mut PathBuilder, x: f32, y: f32) {
    pb.move_to(x, y);
}

#[no_mangle]
pub extern "C" fn wgr_builder_line_to(pb: &mut PathBuilder, x: f32, y: f32) {
    pb.line_to(x, y);
}

#[no_mangle]
pub extern "C" fn wgr_builder_curve_to(pb: &mut PathBuilder, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
    pb.curve_to(c1x, c1y, c2x, c2y, x, y);
}

#[no_mangle]
pub extern "C" fn wgr_builder_quad_to(pb: &mut PathBuilder, cx: f32, cy: f32, x: f32, y: f32) {
    pb.quad_to(cx, cy, x, y);
}

#[no_mangle]
pub extern "C" fn wgr_builder_close(pb: &mut PathBuilder) {
    pb.close();
}

#[no_mangle]
pub extern "C" fn wgr_builder_set_fill_mode(pb: &mut PathBuilder, fill_mode: FillMode) {
    pb.set_fill_mode(fill_mode)
}

#[repr(C)]
pub struct VertexBuffer {
    data: *const OutputVertex,
    len: usize
}

#[no_mangle]
pub extern "C" fn wgr_rasterize_to_tri_strip(pb: &PathBuilder, clip_x: i32, clip_y: i32, clip_width: i32, clip_height: i32) -> VertexBuffer
{
    let result = pb.rasterize_to_tri_strip(clip_x, clip_y, clip_width, clip_height);
    let (data, len) = (result.as_ptr(), result.len());
    std::mem::forget(result);
    VertexBuffer { data, len }
}

#[no_mangle]
pub extern "C" fn wgr_vertex_buffer_release(vb: VertexBuffer)
{
    unsafe { drop(Box::from_raw(std::slice::from_raw_parts_mut(vb.data as *mut OutputVertex, vb.len))) }
}

#[no_mangle]
pub unsafe extern "C" fn wgr_builder_release(pb: *mut PathBuilder) {
    drop(Box::from_raw(pb));
}
#![allow(unused_parens)]
#![allow(overflowing_literals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod bezier;
mod hwrasterizer;
mod aarasterizer;
mod aacoverage;
mod hwvertexbuffer;
mod helpers;
mod types;
mod geometry_sink;
mod matrix;
mod real;
mod fix;

use std::{ffi::c_void, rc::Rc, cell::RefCell};

use hwrasterizer::CHwRasterizer;
use hwvertexbuffer::{CHwVertexBufferBuilder, CHwVertexBuffer};
use matrix::CMatrix;
use types::{CoordinateSpace, CD3DDeviceLevel1, IShapeData, MilFillMode, PathPointTypeStart, MilPoint2F, PathPointTypeLine, HRESULT, MilVertexFormat, MilVertexFormatAttribute};

use crate::geometry_sink::IGeometrySink;
#[repr(C)]
#[derive(Debug)]
pub struct OutputVertex {
    pub x: f32,
    pub y: f32,
    pub coverage: f32
}

impl std::hash::Hash for OutputVertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.coverage.to_bits().hash(state);
    }
}

extern "C" {
    fn pathbuilder_new() -> *mut c_void;
    fn pathbuilder_line_to(ptr: *mut c_void, x: f32, y: f32);
    fn pathbuilder_move_to(ptr: *mut c_void, x: f32, y: f32);
    fn pathbuilder_curve_to(ptr: *mut c_void, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32);
    fn pathbuilder_close(ptr: *mut c_void);
    fn pathbuilder_rasterize(ptr: *mut c_void, out_len: *mut usize, clip_x: i32, clip_y: i32, clip_width: i32, clip_height: i32) -> *mut OutputVertex;
    fn pathbuilder_delete(ptr: *mut c_void);
}

pub struct PathBuilder {
    ptr: *mut c_void
}

impl PathBuilder {
    pub fn new() -> Self {
        Self { ptr: unsafe { pathbuilder_new() } }
    }
    pub fn line_to(&mut self, x: f32, y: f32) {
        unsafe { pathbuilder_line_to(self.ptr, x, y); }
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        unsafe { pathbuilder_move_to(self.ptr, x, y); }
    }
    pub fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        unsafe { pathbuilder_curve_to(self.ptr, c1x, c1y, c2x, c2y, x, y); }
    }
    pub fn close(&mut self) {
        unsafe { pathbuilder_close(self.ptr); }
    }
    pub fn rasterize_to_tri_strip(&mut self, clip_width: i32, clip_height: i32) -> Box<[OutputVertex]> {
        let mut len = 0;
        let ptr = unsafe { pathbuilder_rasterize(self.ptr, &mut len, 0, 0, clip_width, clip_height) };
        unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr, len)) }
    }
}

impl Drop for PathBuilder {
    fn drop(&mut self) {
        unsafe { pathbuilder_delete(self.ptr); }
    }
}

struct RectShape;

impl IShapeData for RectShape {
    fn GetFillMode(&self) -> MilFillMode {
        MilFillMode::Alternate
    }

    fn ConvertToGpPath(&self, points: &mut types::DynArray<types::MilPoint2F>, types: &mut types::DynArray<types::BYTE>) -> HRESULT {
        types.push(PathPointTypeStart);
        points.push(MilPoint2F{X: 10., Y: 10.});

        types.push(PathPointTypeLine);
        points.push(MilPoint2F{X: 40., Y: 10.});

        types.push(PathPointTypeLine);
        points.push(MilPoint2F{X: 40., Y: 40.});

        return types::S_OK;
    }
}


pub fn rasterize(clip_x: i32, clip_y: i32, clip_width: i32, clip_height: i32) {
    let mut rasterizer = CHwRasterizer::new();
    let mut device = CD3DDeviceLevel1::new();
    
    device.clipRect.X = clip_x;
    device.clipRect.Y = clip_y;
    device.clipRect.Width = clip_width;
    device.clipRect.Height = clip_height;
    let device = Rc::new(device);
    /* 
    device.m_rcViewport = device.clipRect;
*/
    let shape = RectShape{};
    let pointsScratch = Rc::new(RefCell::new(Vec::new()));
    let typesScratch = Rc::new(RefCell::new(Vec::new()));
    let worldToDevice: CMatrix<CoordinateSpace::Shape, CoordinateSpace::Device> = CMatrix::Identity();

    rasterizer.Setup(device.clone(), Rc::new(shape), pointsScratch, typesScratch, Some(&worldToDevice));

    let mut m_mvfIn: MilVertexFormat = MilVertexFormatAttribute::MILVFAttrNone as MilVertexFormat;
    let m_mvfGenerated: MilVertexFormat  = MilVertexFormatAttribute::MILVFAttrNone as MilVertexFormat;
    //let mvfaAALocation  = MILVFAttrNone;
    const HWPIPELINE_ANTIALIAS_LOCATION: MilVertexFormatAttribute = MilVertexFormatAttribute::MILVFAttrDiffuse;
    let mvfaAALocation = HWPIPELINE_ANTIALIAS_LOCATION;
    struct CHwPipeline {
        m_pDevice: Rc<CD3DDeviceLevel1>
    }
    let pipeline =  CHwPipeline { m_pDevice: device };
    let m_pHP = &pipeline;

    rasterizer.GetPerVertexDataType(&mut m_mvfIn);
    let mut vertexBuilder= Rc::new(RefCell::new(CHwVertexBufferBuilder::Create(m_mvfIn,                                          m_mvfIn | m_mvfGenerated,
        mvfaAALocation,
        m_pHP.m_pDevice.clone())));

    vertexBuilder.borrow_mut().BeginBuilding();

    rasterizer.SendGeometry(vertexBuilder.clone());
    let mut m_pVB: Rc<CHwVertexBuffer> = todo!();
    vertexBuilder.borrow().FlushTryGetVertexBuffer(&mut m_pVB);
}

#[cfg(test)]
mod tests {
    use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};
    use crate::*;
    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
    #[test]
    fn basic() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(10., 30.);
        p.line_to(30., 30.);
        p.line_to(30., 10.);
        p.close();
        let result = p.rasterize_to_tri_strip(100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x91582a1f5e431eb6);
    }

    #[test]
    fn rust() {
        rasterize(0, 0, 100, 100);
    }
}

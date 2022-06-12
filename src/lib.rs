#![allow(unused_parens)]
#![allow(overflowing_literals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[macro_use]
mod fix;
#[macro_use]
mod helpers;
#[macro_use]
mod real;
mod bezier;
#[macro_use]
mod aarasterizer;
mod hwrasterizer;
mod aacoverage;
mod hwvertexbuffer;

mod types;
mod geometry_sink;
mod matrix;

mod nullable_ref;

use std::{rc::Rc, cell::{RefCell, Cell}, mem::take};

use hwrasterizer::CHwRasterizer;
use hwvertexbuffer::CHwVertexBufferBuilder;
use matrix::CMatrix;
use types::{CoordinateSpace, CD3DDeviceLevel1, IShapeData, MilFillMode, PathPointTypeStart, MilPoint2F, PathPointTypeLine, HRESULT, MilVertexFormat, MilVertexFormatAttribute, DynArray, BYTE, PathPointTypeBezier, PathPointTypeCloseSubpath, CMILSurfaceRect};


#[repr(C)]
#[derive(Debug, Default)]
pub struct OutputVertex {
    pub x: f32,
    pub y: f32,
    pub coverage: f32
}

pub enum FillMode {
    EvenOdd = 0,
    Winding = 1,
}

impl std::hash::Hash for OutputVertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.coverage.to_bits().hash(state);
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


pub type PathBuilder = PathBuilderRust;

pub struct PathBuilderRust {
    points: DynArray<MilPoint2F>,
    types: DynArray<BYTE>,
    initial_point: Option<MilPoint2F>,
    fill_mode: MilFillMode,
    outside_bounds: Option<CMILSurfaceRect>,
    need_inside: bool
}

impl PathBuilderRust {
    pub fn new() -> Self {
        Self {
        points: Vec::new(),
        types: Vec::new(),
        initial_point: None,
        fill_mode: MilFillMode::Alternate,
        outside_bounds: None,
        need_inside: true,
        }
    }
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.types.push(PathPointTypeLine);
        self.points.push(MilPoint2F{X: x, Y: y});
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.types.push(PathPointTypeStart);
        self.points.push(MilPoint2F{X: x, Y: y});
        self.initial_point = Some(MilPoint2F{X: x, Y: y});
    }
    pub fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.types.push(PathPointTypeBezier);
        self.points.push(MilPoint2F{X:c1x, Y:c1y});
        self.types.push(PathPointTypeBezier);
        self.points.push(MilPoint2F{X:c2x, Y: c2y});
        self.types.push(PathPointTypeBezier);
        self.points.push(MilPoint2F{X: x, Y: y});
    }
    pub fn close(&mut self) {
        if let Some(initial_point) = &self.initial_point {
            self.points.push(*initial_point);
            self.types.push(PathPointTypeLine | PathPointTypeCloseSubpath);
            self.initial_point = None;
        }
    }
    pub fn set_fill_mode(&mut self, fill_mode: FillMode) {
        self.fill_mode = match fill_mode {
            FillMode::EvenOdd => MilFillMode::Alternate,
            FillMode::Winding => MilFillMode::Winding,
        }
    }
    /// Enables rendering geometry for areas outside the shape but
    /// within the bounds.  These areas will be created with
    /// zero alpha.
    /// 
    /// This is useful for creating geometry for other blend modes.
    /// For example:
    /// IN can be done with outside_bounds and need_inside = false
    /// IN with transparency can be done with outside_bounds and need_inside = true
    pub fn set_outside_bounds(&mut self, outside_bounds: Option<(i32, i32, i32, i32)>, need_inside: bool) {
        self.outside_bounds = outside_bounds.map(|r| CMILSurfaceRect { left: r.0, top: r.1, right: r.2, bottom: r.3 });
        self.need_inside = need_inside;
    }
    pub fn rasterize_to_tri_strip(&mut self, clip_x: i32, clip_y: i32, clip_width: i32, clip_height: i32) -> Box<[OutputVertex]> {
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

            struct PathShape {
                fill_mode: MilFillMode,
                points: Cell<DynArray<MilPoint2F>>,
                types: Cell<DynArray<BYTE>>,
            }

            impl IShapeData for PathShape {
                fn GetFillMode(&self) -> MilFillMode {
                    self.fill_mode
                }
            
                fn ConvertToGpPath(&self, points: &mut types::DynArray<types::MilPoint2F>, types: &mut types::DynArray<types::BYTE>) -> HRESULT {
                    points.append(&mut self.points.take());
                    types.append(&mut self.types.take());
            
                    return types::S_OK;
                }
            }

            let path = Rc::new(PathShape { fill_mode: self.fill_mode, points: Cell::new(take(&mut self.points)), types: Cell::new(take(&mut self.types))});
        
            rasterizer.Setup(device.clone(), path, pointsScratch, typesScratch, Some(&worldToDevice));
        
            let mut m_mvfIn: MilVertexFormat = MilVertexFormatAttribute::MILVFAttrNone as MilVertexFormat;
            let m_mvfGenerated: MilVertexFormat  = MilVertexFormatAttribute::MILVFAttrNone as MilVertexFormat;
            //let mvfaAALocation  = MILVFAttrNone;
            const HWPIPELINE_ANTIALIAS_LOCATION: MilVertexFormatAttribute = MilVertexFormatAttribute::MILVFAttrDiffuse;
            let mvfaAALocation = HWPIPELINE_ANTIALIAS_LOCATION;
            struct CHwPipeline {
                m_pDevice: Rc<CD3DDeviceLevel1>
            }
            let pipeline =  CHwPipeline { m_pDevice: device.clone() };
            let m_pHP = &pipeline;
        
            rasterizer.GetPerVertexDataType(&mut m_mvfIn);
            let vertexBuilder= Rc::new(RefCell::new(CHwVertexBufferBuilder::Create(m_mvfIn,                                          m_mvfIn | m_mvfGenerated,
                mvfaAALocation,
                m_pHP.m_pDevice.clone())));
        
            vertexBuilder.borrow_mut().SetOutsideBounds(self.outside_bounds.as_ref(), self.need_inside);
            vertexBuilder.borrow_mut().BeginBuilding();
        
            rasterizer.SendGeometry(vertexBuilder.clone());
            vertexBuilder.borrow_mut().FlushTryGetVertexBuffer(None);
            device.output.replace(Vec::new()).into_boxed_slice()

    }
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
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 10);
        assert_eq!(dbg!(calculate_hash(&result)), 0x91582a1f5e431eb6);
    }

    #[test]
    fn simple() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0xa33cb40dd676741e);
    }

    #[test]
    fn rust() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0xa33cb40dd676741e);
    }

    #[test]
    fn fill_mode() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        p.line_to(10., 40.);
        p.close();
        p.move_to(15., 15.);
        p.line_to(35., 15.);
        p.line_to(35., 35.);
        p.line_to(15., 35.);
        p.close();
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x81d3f6981834234b);

        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        p.line_to(10., 40.);
        p.close();
        p.move_to(15., 15.);
        p.line_to(35., 15.);
        p.line_to(35., 35.);
        p.line_to(15., 35.);
        p.close();
        p.set_fill_mode(FillMode::Winding);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x6ebf6d38d18c3fa9);

    }

    #[test]
    fn curve() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.curve_to(40., 10., 40., 10., 40., 40.);
        p.close();
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x2b4a3e89d19fb5d5);
    }

    #[test]
    fn partial_coverage_last_line() {
        let mut p = PathBuilder::new();
        
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 39.6);
        p.line_to(10., 39.6);

        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 16);
        assert_eq!(dbg!(calculate_hash(&result)), 0xf31cd214f48dafbf);
    }

    #[test]
    fn delta_upper_bound() {
        let mut p = PathBuilder::new();
        p.move_to(-122.3 + 200.,84.285);
        p.curve_to(-122.3 + 200., 84.285, -122.2 + 200.,86.179, -123.03 + 200., 86.16);
        p.curve_to(-123.85 + 200., 86.141, -140.3 + 200., 38.066, -160.83 + 200., 40.309);
        p.curve_to(-160.83 + 200., 40.309, -143.05 + 200., 32.956,  -122.3 + 200., 84.285);
        p.close();

        let result = p.rasterize_to_tri_strip(0, 0, 400, 400);
        assert_eq!(result.len(), 676);
        assert_eq!(dbg!(calculate_hash(&result)), 0x3208fd473e65e40a);
    }


    #[test]
    fn self_intersect() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(10., 40.);
        p.line_to(40., 40.);
        p.close();
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x27d610994219a978);
    }

    #[test]
    fn grid() {
        let mut p = PathBuilderRust::new();

        for i in 0..200 {
            let offset = i as f32 * 1.3;
            p.move_to(0. + offset, -8.);
            p.line_to(0.5 + offset, -8.);
            p.line_to(0.5 + offset, 40.);
            p.line_to(0. + offset, 40.);
            p.close();
        }
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        //assert_eq!(dbg!(calculate_hash(&result)), 0xab9e651ac1aa1d48);
    }

    #[test]
    fn outside() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(10., 40.);
        p.line_to(40., 40.);
        p.close();
        p.set_outside_bounds(Some((0, 0, 50, 50)), false);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x27edd1fbd58b4d90);
    }

    #[test]
    fn outside_inside() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(10., 40.);
        p.line_to(40., 40.);
        p.close();
        p.set_outside_bounds(Some((0, 0, 50, 50)), true);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x74454613e878570);
    }
}

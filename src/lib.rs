/*!
Converts a 2D path into a set of vertices of a triangle strip mesh that represents the antialiased fill of that path.

```rust
    use wpf_gpu_raster::PathBuilder;
    let mut p = PathBuilder::new();
    p.move_to(10., 10.);
    p.line_to(40., 10.);
    p.line_to(40., 40.);
    let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
```

*/
#![allow(unused_parens)]
#![allow(overflowing_literals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

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

#[cfg(feature = "c_bindings")]
pub mod c_bindings;

use std::{rc::Rc, cell::RefCell};

use hwrasterizer::CHwRasterizer;
use hwvertexbuffer::CHwVertexBufferBuilder;
use matrix::CMatrix;
use types::{CoordinateSpace, CD3DDeviceLevel1, IShapeData, MilFillMode, PathPointTypeStart, MilPoint2F, PathPointTypeLine, MilVertexFormat, MilVertexFormatAttribute, DynArray, BYTE, PathPointTypeBezier, PathPointTypeCloseSubpath, CMILSurfaceRect};


#[repr(C)]
#[derive(Debug, Default)]
pub struct OutputVertex {
    pub x: f32,
    pub y: f32,
    pub coverage: f32
}

#[repr(C)]
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

pub struct PathBuilder {
    points: DynArray<MilPoint2F>,
    types: DynArray<BYTE>,
    initial_point: Option<MilPoint2F>,
    in_shape: bool,
    fill_mode: MilFillMode,
    outside_bounds: Option<CMILSurfaceRect>,
    need_inside: bool
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
        points: Vec::new(),
        types: Vec::new(),
        initial_point: None,
        in_shape: false,
        fill_mode: MilFillMode::Alternate,
        outside_bounds: None,
        need_inside: true,
        }
    }
    pub fn line_to(&mut self, x: f32, y: f32) {
        if let Some(initial_point) = self.initial_point {
            if !self.in_shape {
                self.types.push(PathPointTypeStart);
                self.points.push(initial_point);
                self.in_shape = true;
            }
            self.types.push(PathPointTypeLine);
            self.points.push(MilPoint2F{X: x, Y: y});
        } else {
            self.initial_point = Some(MilPoint2F{X: x, Y: y})
        }
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.in_shape = false;
        self.initial_point = Some(MilPoint2F{X: x, Y: y});
    }
    pub fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        let initial_point = match self.initial_point {
            Some(initial_point) => initial_point,
            None => MilPoint2F{X:c1x, Y:c1y}
        };
        if !self.in_shape {
            self.types.push(PathPointTypeStart);
            self.points.push(initial_point);
            self.initial_point = Some(initial_point);
            self.in_shape = true;
        }
        self.types.push(PathPointTypeBezier);
        self.points.push(MilPoint2F{X:c1x, Y:c1y});
        self.types.push(PathPointTypeBezier);
        self.points.push(MilPoint2F{X:c2x, Y: c2y});
        self.types.push(PathPointTypeBezier);
        self.points.push(MilPoint2F{X: x, Y: y});
    }
    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        // For now we just implement quad_to on top of curve_to.
        // Long term we probably want to support quad curves
        // directly.
        let c0 = match self.initial_point {
            Some(initial_point) => initial_point,
            None => MilPoint2F{X:cx, Y:cy}
        };

        let c1x = c0.X + (2./3.) * (cx - c0.X);
        let c1y = c0.Y + (2./3.) * (cx - c0.Y);

        let c2x = x + (2./3.) * (cx - x);
        let c2y = y + (2./3.) * (cy - y);

        self.curve_to(c1x, c1y, c2x, c2y, x, y);
    }
    pub fn close(&mut self) {
        if let Some(last) = self.types.last_mut() {
            *last |= PathPointTypeCloseSubpath;
        }
        self.in_shape = false;
        self.initial_point = None;
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
    pub fn rasterize_to_tri_strip(&self, clip_x: i32, clip_y: i32, clip_width: i32, clip_height: i32) -> Box<[OutputVertex]> {
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
        let worldToDevice: CMatrix<CoordinateSpace::Shape, CoordinateSpace::Device> = CMatrix::Identity();

        struct PathShape {
            fill_mode: MilFillMode,
        }

        impl IShapeData for PathShape {
            fn GetFillMode(&self) -> MilFillMode {
                self.fill_mode
            }
        }

        let path = Rc::new(PathShape { fill_mode: self.fill_mode });
    
        rasterizer.Setup(device.clone(), path, Some(&worldToDevice));
    
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
    
        rasterizer.SendGeometry(vertexBuilder.clone(), &self.points, &self.types);
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
        assert_eq!(dbg!(calculate_hash(&result)), 0x5851570566450135);
    }

    #[test]
    fn simple() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x81a9af7769f88e68);
    }

    #[test]
    fn rust() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x81a9af7769f88e68);
    }

    #[test]
    fn fill_mode() {
        let mut p = PathBuilder::new();
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
        assert_eq!(dbg!(calculate_hash(&result)), 0xb34344234f2f75a8);

        let mut p = PathBuilder::new();
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
        assert_eq!(dbg!(calculate_hash(&result)), 0xee4ecd8a738fc42c);

    }

    #[test]
    fn range() {
        let mut p = PathBuilder::new();
        p.curve_to(8.872974e16, 0., 0., 0., 0., 0.);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn multiple_starts() {
        let mut p = PathBuilder::new();
        p.line_to(10., 10.);
        p.move_to(0., 0.);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn path_closing() {
        let mut p = PathBuilder::new();
        p.curve_to(0., 0., 0., 0., 0., 32.0);
        p.close();
        p.curve_to(0., 0., 0., 0., 0., 32.0);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn curve() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.curve_to(40., 10., 40., 10., 40., 40.);
        p.close();
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x6f92480332842ac9);
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
        assert_eq!(dbg!(calculate_hash(&result)), 0xf606699f20d45d96);
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
        assert_eq!(dbg!(calculate_hash(&result)), 0xd216dc8076add4b3);
    }


    #[test]
    fn self_intersect() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(10., 40.);
        p.line_to(40., 40.);
        p.close();
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0xb8cbea29b27f7598);
    }

    #[test]
    fn grid() {
        let mut p = PathBuilder::new();

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
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(10., 40.);
        p.line_to(40., 40.);
        p.close();
        p.set_outside_bounds(Some((0, 0, 50, 50)), false);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x1e734743e1785634);
    }

    #[test]
    fn outside_inside() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(10., 40.);
        p.line_to(40., 40.);
        p.close();
        p.set_outside_bounds(Some((0, 0, 50, 50)), true);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x1b741fc435aa1897);
    }

    #[test]
    fn clip_edge() {
        let mut p = PathBuilder::new();
        // tests the bigNumerator < 0 case of aarasterizer::ClipEdge
        p.curve_to(-24., -10., -300., 119., 0.0, 0.0);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        // The edge merging only happens between points inside the enumerate buffer. This means
        // that the vertex output can depend on the size of the enumerate buffer because there
        // the number of edges and positions of vertices will change depending on edge merging.
        if ENUMERATE_BUFFER_NUMBER!() == 32 {
            assert_eq!(result.len(), 170);
        } else {
            assert_eq!(result.len(), 238);
        }

    }

    #[test]
    fn enum_buffer_num() {
        let mut p = PathBuilder::new();
        p.curve_to(0.0, 0.0, 0.0, 12.0, 0.0, 44.919434);
        p.line_to(64.0, 36.0 );
        p.line_to(0.0, 80.0,);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 574);
    }

    #[test]
    fn fill_alternating_empty_interior_pairs() {
        let mut p = PathBuilder::new();
        p.line_to( 0., 2. );
        p.curve_to(0.0, 0.0,1., 6., 0.0, 0.0);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 18);
    }

    #[test]
    fn fill_winding_empty_interior_pairs() {
        let mut p = PathBuilder::new();
        p.curve_to(45., 61., 0.09, 0., 0., 0.);
        p.curve_to(45., 61., 0.09, 0., 0., 0.);
        p.curve_to(0., 0., 0., 38., 0.09, 15.);
        p.set_fill_mode(FillMode::Winding);
        let result = p.rasterize_to_tri_strip(0, 0, 100, 100);
        assert_eq!(result.len(), 820);
    }
}

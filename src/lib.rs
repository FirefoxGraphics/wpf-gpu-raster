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

use std::{ffi::c_void, rc::Rc, cell::{RefCell, Cell}, mem::take};

use hwrasterizer::CHwRasterizer;
use hwvertexbuffer::{CHwVertexBufferBuilder, CHwVertexBuffer};
use matrix::CMatrix;
use types::{CoordinateSpace, CD3DDeviceLevel1, IShapeData, MilFillMode, PathPointTypeStart, MilPoint2F, PathPointTypeLine, HRESULT, MilVertexFormat, MilVertexFormatAttribute, DynArray, BYTE, PathPointTypeBezier, PathPointTypeCloseSubpath};

use crate::geometry_sink::IGeometrySink;
#[repr(C)]
#[derive(Debug, Default)]
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




pub struct PathBuilderRust {
    points: DynArray<MilPoint2F>,
    types: DynArray<BYTE>,
    initial_point: Option<MilPoint2F>,
    fill_mode: MilFillMode,
}

/*struct PathBuilder : CShapeBase {
    DynArray<MilPoint2F> points;
    DynArray<BYTE> types;
    bool has_initial = false;
    MilPoint2F initial_point;
    MilFillMode::Enum fill_mode = MilFillMode::Alternate;
    override HRESULT ConvertToGpPath(
                                         __out_ecount(1) DynArray<MilPoint2F> &rgPoints,
                                         // Path points
                                         __out_ecount(1) DynArray<BYTE>      &rgTypes,
                                         // Path types
                                         IN  bool                fStroking
                                         // Stroking if true, filling otherwise (optional)
                                    ) const
    {
            rgPoints.Copy(points);
            rgTypes.Copy(types);
    }
    MilFillMode::Enum GetFillMode() const {
                    return fill_mode;
    }
    bool HasGaps() const { return false;
    }
    bool HasHollows() const { return false; }
    bool IsEmpty() const { return false; }
    UINT GetFigureCount() const { return 1; }
    bool IsAxisAlignedRectangle() const { return false; }

    virtual bool GetCachedBoundsCore(
    __out_ecount(1) MilRectF &rect) const { abort(); }
    virtual void SetCachedBounds(
    __in_ecount(1) const MilRectF &rect) const { abort(); };  // Bounding box to cache

    virtual __outro_ecount(1) const IFigureData &GetFigure(IN UINT index) const { abort(); }
    void line_to(float x, float y) {
            types.Add(PathPointTypeLine);
            points.Add({x, y});
    }
    void move_to(float x, float y) {
            types.Add(PathPointTypeStart);
            points.Add({x, y});
            initial_point = {x, y};
    }
    void curve_to(float c1x, float c1y, float c2x, float c2y, float x, float y) {
            points.Add({c1x, c1y});
            points.Add({c2x, c2y});
            points.Add({x, y});
            types.AddAndSet(3, PathPointTypeBezier);
    }
    void close() {
            if (has_initial) {
                    points.Add(initial_point);
                    types.Add(PathPointTypeLine | PathPointTypeCloseSubpath);
            }
    }

    OutputVertex *rasterize(size_t *outLen, int clip_x, int clip_y, int clip_width, int clip_height);

};*/

impl PathBuilderRust {
    pub fn new() -> Self {
        Self {
        points: Vec::new(),
        types: Vec::new(),
        initial_point: None,
        fill_mode: MilFillMode::Alternate,
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
        }
    }
    pub fn rasterize_to_tri_strip(&mut self, clip_width: i32, clip_height: i32) -> Box<[OutputVertex]> {
            let mut rasterizer = CHwRasterizer::new();
            let mut device = CD3DDeviceLevel1::new();
            
            device.clipRect.X = 0;
            device.clipRect.Y = 0;
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
                points: Cell<DynArray<MilPoint2F>>,
                types: Cell<DynArray<BYTE>>,
            };

            impl IShapeData for PathShape {
                fn GetFillMode(&self) -> MilFillMode {
                    MilFillMode::Alternate
                }
            
                fn ConvertToGpPath(&self, points: &mut types::DynArray<types::MilPoint2F>, types: &mut types::DynArray<types::BYTE>) -> HRESULT {
                    points.append(&mut self.points.take());
                    types.append(&mut self.types.take());
            
                    return types::S_OK;
                }
            }

            let path = Rc::new(PathShape { points: Cell::new(take(&mut self.points)), types: Cell::new(take(&mut self.types))});
        
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
            let mut vertexBuilder= Rc::new(RefCell::new(CHwVertexBufferBuilder::Create(m_mvfIn,                                          m_mvfIn | m_mvfGenerated,
                mvfaAALocation,
                m_pHP.m_pDevice.clone())));
        
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
        let result = p.rasterize_to_tri_strip(100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0x91582a1f5e431eb6);
    }


    #[test]
    fn simple() {
        let mut p = PathBuilder::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        let result = p.rasterize_to_tri_strip(100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0xa33cb40dd676741e);
    }

    #[test]
    fn rust() {
        let mut p = PathBuilderRust::new();
        p.move_to(10., 10.);
        p.line_to(40., 10.);
        p.line_to(40., 40.);
        let result = p.rasterize_to_tri_strip(100, 100);
        assert_eq!(dbg!(calculate_hash(&result)), 0xa33cb40dd676741e);
    }
}

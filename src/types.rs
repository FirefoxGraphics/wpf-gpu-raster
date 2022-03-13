pub(crate) type LONG = i32;
pub(crate) type INT = i32;
pub(crate) type UINT = u32;
pub(crate) type ULONG = u32;
pub(crate) type DWORD = ULONG;
pub(crate) type WORD = u16;
pub(crate) type LONGLONG = i64;
pub(crate) type ULONGLONG = u64;
pub(crate) type BYTE = u8;
pub(crate) type FLOAT = f32;
pub(crate) type REAL = FLOAT;
pub(crate) type HRESULT = LONG;

pub(crate) const S_OK: HRESULT = 0;
pub(crate) const INTSAFE_E_ARITHMETIC_OVERFLOW: HRESULT = 0x80070216;
pub(crate) const WGXERR_VALUEOVERFLOW: HRESULT = INTSAFE_E_ARITHMETIC_OVERFLOW;
pub(crate) const WINCODEC_ERR_VALUEOVERFLOW: HRESULT = INTSAFE_E_ARITHMETIC_OVERFLOW;
const fn MAKE_HRESULT(sev: LONG,fac: LONG,code: LONG) -> HRESULT {
    ( (((sev)<<31) | ((fac)<<16) | ((code))) )
}

const FACILITY_WGX: LONG = 0x898;


const fn MAKE_WGXHR( sev: LONG, code: LONG) -> HRESULT {
        MAKE_HRESULT( sev, FACILITY_WGX, (code) )
}
    
const fn MAKE_WGXHR_ERR( code: LONG ) -> HRESULT 
{
        MAKE_WGXHR( 1, code )
}

pub const WGXHR_CLIPPEDTOEMPTY: HRESULT =                MAKE_WGXHR(0, 1);
pub const WGXHR_EMPTYFILL: HRESULT =                      MAKE_WGXHR(0, 2);
pub const WGXHR_INTERNALTEMPORARYSUCCESS: HRESULT =      MAKE_WGXHR(0, 3);
pub const WGXHR_RESETSHAREDHANDLEMANAGER: HRESULT =      MAKE_WGXHR(0, 4);

pub const WGXERR_BADNUMBER: HRESULT =                     MAKE_WGXHR_ERR(0x00A);   //  4438

pub fn FAILED(hr: HRESULT) -> bool {
    hr != S_OK
}
pub trait NullPtr {
    fn make() -> Self;
}

impl<T> NullPtr for *mut T {
    fn make() -> Self {
        std::ptr::null_mut()
    }
}

impl<T> NullPtr for *const T {
    fn make() -> Self {
        std::ptr::null()
    }
}

pub fn NULL<T: NullPtr>() -> T {
    T::make()
}
#[derive(Default, Clone)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}
#[derive(Default, Clone, Copy)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG
}

pub struct MilPoint2F
{
    pub X: FLOAT,
    pub Y: FLOAT,
}

#[derive(Default)]
pub struct MilPointAndSizeL
{
    pub X: INT,
    pub Y: INT,
    pub Width: INT,
    pub Height: INT,
}

pub struct CMILSurfaceRect;

#[derive(PartialEq)]
pub enum MilAntiAliasMode {
    None = 0,
    EightByEight = 1,
}
#[derive(PartialEq, Clone, Copy)]
pub enum MilFillMode {
    Alternate = 0,
    Winding = 1,
}

pub const    PathPointTypeStart: u8           = 0;    // move
pub const    PathPointTypeLine: u8            = 1;    // line
pub const    PathPointTypeBezier: u8          = 3;    // default Bezier (= cubic Bezier)
pub const    PathPointTypePathTypeMask: u8    = 0x07; // type mask (lowest 3 bits).
pub const    PathPointTypeCloseSubpath: u8    = 0x80; // closed flag


pub type DynArray<T> = Vec<T>;

pub trait DynArrayExts<T> {
    fn Reset(&mut self, shrink: bool);
    fn GetCount(&self) -> usize;
    fn SetCount(&mut self, count: usize);
    fn GetDataBuffer(&self) -> &[T];
}

impl<T> DynArrayExts<T> for DynArray<T> {
    fn Reset(&mut self, shrink: bool) {
        self.clear();
        if shrink {
            self.shrink_to_fit();
        }
    }
    fn GetCount(&self) -> usize {
        self.len()
    }
    fn SetCount(&mut self, count: usize) {
        assert!(count <= self.len());
        self.truncate(count);
    }

    fn GetDataBuffer(&self) -> &[T] {
        self
    }
}


pub struct CD3DDeviceLevel1 {
    pub clipRect: MilPointAndSizeL
}
impl CD3DDeviceLevel1 {
    pub fn new() -> Self { todo!() }
    pub fn GetClipRect(&self, rect: &mut MilPointAndSizeL) {
        todo!();
    }
    pub fn GetViewport(&self) -> MilPointAndSizeL { todo!() }
}
pub struct CHwPipelineBuilder;

pub mod CoordinateSpace {
    #[derive(Default, Clone)]
    pub struct Shape;
    #[derive(Default, Clone)]
    pub struct Device;
}

pub trait IShapeData {
    fn GetFillMode(&self) -> MilFillMode;
    fn ConvertToGpPath(&self, points: &mut DynArray<MilPoint2F>, types: &mut DynArray<BYTE>) -> HRESULT;
}

pub enum MilVertexFormat {
    MILVFAttrNone,
    MILVFAttrXY
}

pub enum MilVertexFormatAttribute {}

pub struct CHwPipeline;

pub struct CBufferDispenser;

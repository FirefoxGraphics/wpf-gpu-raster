    #define DeclareTag(tag, szOwner, szDescription)
    #define IsTagEnabled(tag) (FALSE)
#define TraceTag(x)
#define ExternTag(tag)

// avaondebugp.h
#define     Mt(x)                               ((void)#x,0)
#define     MtDefine(tag, szOwner, szDescrip)
#define     MtAdd(mt, lCnt, lVal)
#define MIL_FORCEINLINE inline
#include <stdint.h>
     #include <stdlib.h>
#define        WINCODEC_ERR_VALUEOVERFLOW  (0x80070216)

// precomp.hxx
#define IGNORE_HR(x) ((void)(x))

/*typedef int32_t BOOL;
typedef int INT;
typedef long long LONGLONG;
#define CONST const
#define IN
#define OUT
#define OPTIONAL
#define FAR*/
//#include "salextra.h"
//#include "pal_mstypes.h"
#define Int32x32To64(a, b)  (((__int64)((long)(a))) * ((__int64)((long)(b))))
#include <stdlib.h>
#include <stdarg.h>
#include <float.h>
#include "palrt.h"
#include "pal_assert.h"
#include "intsafe.h"
#include "windef.h"
// debug.hxx
#define RIP(msg) do { abort(); } while (0)
// std.h
#define __pfx_assert(Exp, Msg) do {} while ( UNCONDITIONAL_EXPR(false) )
typedef struct _D3DMATRIX {
    union {
        struct {
            float        _11, _12, _13, _14;
            float        _21, _22, _23, _24;
            float        _31, _32, _33, _34;
            float        _41, _42, _43, _44;

        };
        float m[4][4];
    };
} D3DMATRIX;

struct  D3DXMATRIX : D3DMATRIX {
        D3DXMATRIX(const float * pf) {
                _11 = pf[0];
                _12 = pf[1];
                _13 = pf[2];
                _14 = pf[3];
                _21 = pf[4];
                _22 = pf[5];
                _23 = pf[6];
                _24 = pf[7];
                _31 = pf[8];
                _32 = pf[9];
                _33 = pf[10];
                _34 = pf[11];
                _41 = pf[12];
                _42 = pf[13];
                _43 = pf[14];
                _44 = pf[15];
        }
        D3DXMATRIX(float m00, float m01, float m02, float m03,
                float m10, float m11, float m12, float m13,
                float m20, float m21, float m22, float m23,
                float m30, float m31, float m32, float m33) {
                _11 = m00;
                _12 = m01;
                _13 = m02;
                _14 = m03;
                _21 = m10;
                _22 = m11;
                _23 = m12;
                _24 = m13;
                _31 = m20;
                _32 = m21;
                _33 = m22;
                _34 = m23;
                _41 = m30;
                _42 = m31;
                _43 = m32;
                _44 = m33;
        }

        D3DXMATRIX(const D3DMATRIX &a) {
                memcpy(m, a.m, sizeof(m));
        }
        D3DXMATRIX() {}

        // From DirectXMathMatrix.inl
        D3DXMATRIX XMMatrixMultiply(D3DXMATRIX M1, D3DXMATRIX M2) const {
        
            D3DXMATRIX mResult;
    // Cache the invariants in registers
    float x = M1.m[0][0];
    float y = M1.m[0][1];
    float z = M1.m[0][2];
    float w = M1.m[0][3];
    // Perform the operation on the first row
    mResult.m[0][0] = (M2.m[0][0] * x) + (M2.m[1][0] * y) + (M2.m[2][0] * z) + (M2.m[3][0] * w);
    mResult.m[0][1] = (M2.m[0][1] * x) + (M2.m[1][1] * y) + (M2.m[2][1] * z) + (M2.m[3][1] * w);
    mResult.m[0][2] = (M2.m[0][2] * x) + (M2.m[1][2] * y) + (M2.m[2][2] * z) + (M2.m[3][2] * w);
    mResult.m[0][3] = (M2.m[0][3] * x) + (M2.m[1][3] * y) + (M2.m[2][3] * z) + (M2.m[3][3] * w);
    // Repeat for all the other rows
    x = M1.m[1][0];
    y = M1.m[1][1];
    z = M1.m[1][2];
    w = M1.m[1][3];
    mResult.m[1][0] = (M2.m[0][0] * x) + (M2.m[1][0] * y) + (M2.m[2][0] * z) + (M2.m[3][0] * w);
    mResult.m[1][1] = (M2.m[0][1] * x) + (M2.m[1][1] * y) + (M2.m[2][1] * z) + (M2.m[3][1] * w);
    mResult.m[1][2] = (M2.m[0][2] * x) + (M2.m[1][2] * y) + (M2.m[2][2] * z) + (M2.m[3][2] * w);
    mResult.m[1][3] = (M2.m[0][3] * x) + (M2.m[1][3] * y) + (M2.m[2][3] * z) + (M2.m[3][3] * w);
    x = M1.m[2][0];
    y = M1.m[2][1];
    z = M1.m[2][2];
    w = M1.m[2][3];
    mResult.m[2][0] = (M2.m[0][0] * x) + (M2.m[1][0] * y) + (M2.m[2][0] * z) + (M2.m[3][0] * w);
    mResult.m[2][1] = (M2.m[0][1] * x) + (M2.m[1][1] * y) + (M2.m[2][1] * z) + (M2.m[3][1] * w);
    mResult.m[2][2] = (M2.m[0][2] * x) + (M2.m[1][2] * y) + (M2.m[2][2] * z) + (M2.m[3][2] * w);
    mResult.m[2][3] = (M2.m[0][3] * x) + (M2.m[1][3] * y) + (M2.m[2][3] * z) + (M2.m[3][3] * w);
    x = M1.m[3][0];
    y = M1.m[3][1];
    z = M1.m[3][2];
    w = M1.m[3][3];
    mResult.m[3][0] = (M2.m[0][0] * x) + (M2.m[1][0] * y) + (M2.m[2][0] * z) + (M2.m[3][0] * w);
    mResult.m[3][1] = (M2.m[0][1] * x) + (M2.m[1][1] * y) + (M2.m[2][1] * z) + (M2.m[3][1] * w);
    mResult.m[3][2] = (M2.m[0][2] * x) + (M2.m[1][2] * y) + (M2.m[2][2] * z) + (M2.m[3][2] * w);
    mResult.m[3][3] = (M2.m[0][3] * x) + (M2.m[1][3] * y) + (M2.m[2][3] * z) + (M2.m[3][3] * w);
    return mResult;}

/*        D3DXMATRIX&
        operator *= (float f)
        {
                _11 *= f; _12 *= f; _13 *= f; _14 *= f;
                _21 *= f; _22 *= f; _23 *= f; _24 *= f;
                _31 *= f; _32 *= f; _33 *= f; _34 *= f;
                _41 *= f; _42 *= f; _43 *= f; _44 *= f;
                return *this;
        }
*/
        D3DXMATRIX
        operator * ( float f ) const
{
    return D3DXMATRIX(_11 * f, _12 * f, _13 * f, _14 * f,
                      _21 * f, _22 * f, _23 * f, _24 * f,
                      _31 * f, _32 * f, _33 * f, _34 * f,
                      _41 * f, _42 * f, _43 * f, _44 * f);
} 
        D3DMATRIX
        operator*(const D3DMATRIX& m) const {
                return XMMatrixMultiply(*this, m);
        }

                // D3DXMatrixDeterminant
        inline float determinant() const
        {
            abort();
            return 0;
        }
        void reset_to_identity() {
                _11 = 1;
                _12 = 0;
                _13 = 0;
                _14 = 0;
                _21 = 0;
                _22 = 1;
                _23 = 0;
                _24 = 0;
                _31 = 0;
                _32 = 0;
                _33 = 1;
                _34 = 0;
                _41 = 0;
                _42 = 0;
                _43 = 0;
                _44 = 1;

        }
};


class IWICBitmapSource;
#define MILINSTRUMENTATIONFLAGS_DONOTHING               0x00
#define SET_MILINSTRUMENTATION_FLAGS(f) \
    static const DWORD MILINSTRUMENTATIONFLAGS = (f)

#define CCHILDREN_TITLEBAR 5
#ifndef BEGIN_MILENUM
#define BEGIN_MILENUM(type)                     \
    namespace type {                            \
            enum Enum {	                        \

#define END_MILENUM                             \
            FORCE_DWORD = 0xffffffff            \
        };                                      \
    }

#define MILENUM(type) type::Enum

#endif /*BEGIN_MILENUM*/

#ifndef BEGIN_MILFLAGENUM

#define BEGIN_MILFLAGENUM(type)                 \
    namespace type {                            \
        enum FlagsEnum {                        \

#define END_MILFLAGENUM                         \
           FORCE_DWORD = 0xffffffff             \
        };                                      \
                                                \
        typedef TMILFlagsEnum<FlagsEnum> Flags; \
    }
    
#define MILFLAGENUM(type) type::Flags

#define WINCODEC_SDK_VERSION_WPF 0x0236
#define Assert(x) assert(x)
DEFINE_GUID(CLSID_WICImagingFactoryWPF, 0xcacaf262, 0x9370, 0x4615, 0xa1, 0x3b, 0x9f, 0x55, 0x39, 0xda, 0x4c, 0xa);

template<class E>
struct TMILFlagsEnum
{
    E flags;

    TMILFlagsEnum() { }
    TMILFlagsEnum(const E &_Right) { flags = _Right; }
    TMILFlagsEnum(const int &_Right) { flags = static_cast<E>(_Right); }

    operator const E &() const { return flags; }

    TMILFlagsEnum &operator|=(const int &_Right)
    {
        flags = static_cast<E>(flags | _Right);
        return *this;
    }

    TMILFlagsEnum &operator&=(const int &_Right)
    {
        flags = static_cast<E>(flags & _Right);
        return *this;
    }

    TMILFlagsEnum &operator^=(const int &_Right)
    {
        flags = static_cast<E>(flags ^ _Right);
        return *this;
    }
};


#endif /*BEGIN_MILFLAGENUM*/
typedef long            FXPT2DOT30;
typedef struct tagCIEXYZ {
  FXPT2DOT30 ciexyzX;
  FXPT2DOT30 ciexyzY;
  FXPT2DOT30 ciexyzZ;
} CIEXYZ;

#include "wincodec_private_generated.h"
#include "wgx_core_types.h"
#include "basetypes.h"
#include "core/common/basetypes.h"
#include "wgx_render_types.h"
#include "wgx_error.h"
//#include "wgx_misc.h"

// UtilMisc.h
#ifdef min
#undef min
#endif

template < class T > inline T min ( T a, T b ) { return a < b ? a : b; }

#ifdef max
#undef max
#endif

template < class T > inline T max ( T a, T b ) { return a > b ? a : b; }

#define ReleaseInterfaceNoNULL(x) do { if (x) { (x)->Release(); } } while (UNCONDITIONAL_EXPR(false))

// MemUtils.h
#define DECLARE_METERHEAP_ALLOC(a, b)
typedef INT_PTR PERFMETERTAG;
__checkReturn __allocator
inline HRESULT HrMalloc(
    PERFMETERTAG mt,
    size_t cbElementSize,
    size_t cElements,
    __deref_bcount(cbElementSize*cElements) void **ppvmemblock
    ) {
            HRESULT hr = S_OK;
        *ppvmemblock = calloc(cbElementSize, cElements);

            return hr;
}
inline
__checkReturn __allocator
HRESULT HrAlloc(
    PERFMETERTAG mt,
    size_t cbSize,
    __deref_bcount(cbSize) void **ppvmemblock
    )
{
    // Callers must ensure a NULL initialized pointer location for the
    // output buffer.

    Assert(ppvmemblock);
    Assert(NULL == *ppvmemblock);
    Assert(cbSize > 0);

    HRESULT hr = S_OK;

    if (ppvmemblock == NULL || cbSize == 0)
    {
        hr = E_INVALIDARG;
    }
    else
    {
        *ppvmemblock = malloc(cbSize);

        if (NULL == *ppvmemblock)
        {
            hr = E_OUTOFMEMORY;
        }
    }

    return hr;
}


#define WPFAlloc(h, mt, cb) ( malloc(cb))
#define WPFAllocType(type, h, mt, cb) (reinterpret_cast<type>(malloc(cb)))
#define WPFAllocClear(h, mt, cb) ( calloc(1, cb))
#define WPFAllocTypeClear(type, h, mt, cb) calloc(1, cb)
#define WPFRealloc(h, mt, ppv, cb) ReallocAnnotationHelper(h, mt, cb, ppv)
class Heap;
extern Heap *ProcessHeap;
__forceinline HRESULT ReallocAnnotationHelper(
    __in_ecount(1) Heap* pheap,
    PERFMETERTAG mt,
    size_t cbSize,
    __deref_bcount(cbSize) void ** ppv
    )
{
    *ppv = realloc(*ppv, cbSize);
    return !!*ppv;
}

#define WPFFree(h, pv) free(pv)
//void __dummfunc() {}


#include "spec_string.h"
typedef INT_PTR PERFMETERTAG;
#define     MtExtern(tag)                       extern PERFMETERTAG g_mt##tag;
#include "MILRect.h"
#include "BaseMatrix.h"
#include "MILMatrix.h"
#include "scanoperation.h"
#include "mem.h"

#include "real.h"

// avaondebugp.h
#define THR(x) (x)
#define UNCONDITIONAL_EXPR(Exp) (Exp)
#define TraceHR(a, b, c, d, e)          (a)
#define ASSIGN_HR_PREFASTOKAY(destVar, errExpr)         (TraceHR((destVar = (errExpr)), FALSE, #errExpr, __FILE__, __LINE__))
#define WHEN_DBG_ANALYSIS(x)
#define ANALYSIS_COMMA_PARAM(x)

class CSpanSink;
class CSpanClipper;
class CAntialiasedFiller;
// rtutils.h
typedef
    TMilRect_<
        INT,
        MilRectL,
        MilPointAndSizeL,
        RectUniqueness::_CMILSurfaceRect_ // Uniqueness specified to prevent confusion with CMilRectL
        > CMILSurfaceRect;

#include "instrumentation.h"
#include "InstrumentationConfig.h"
#include "instrumentationapi.h"
#include "aarasterizer.h"
#include "aacoverage.h"
#include "matrix3x2.h"
#include "GeometrySink.h"

// based off of Chakra/lib/Common/CommonDefines.h
#define MEMORY_ALLOCATION_ALIGNMENT (sizeof(void*)*2)

#include "BufferDispenser.h"
#if 1
//#include <assert.h>
// Arithmetic.h
MIL_FORCEINLINE HRESULT AddUINT(UINT a, UINT b, __out_ecount(1) __deref_out_range(==,a+b) UINT &sum)
{
    UINT c = a + b;
    if (c < a)
    {
        // c is also less than b.
  //      assert(c < b, "UINT overflow test does not hold");
        return WINCODEC_ERR_VALUEOVERFLOW;
    }
    else
    {
        // c is also >= b.
    //    assert(c >= b, "UINT overflow test does not hold");
        sum = c;
        return S_OK;
    }
}
#endif
#include "dynarrayimpl.h"
#include "dynarray.h"
#include "geometry/utils.h"
class CPlainPen;
class IFigureData;
class CShape;
class IShapeBuilder;
class CWideningSink;
class CFillTessellator;
class CParallelogram;
class CHitTest;
class CShapeBase;
class CBounds;
class IPopulationSink;
class CHwColorComponentSource {
public:

    enum VertexComponent
    {
        Diffuse,
        Specular,
        Total
    };
        public:
        void Release() {
                abort();
        }

};
class CD3DDeviceLevel1;
class CHwPipelineBuilder {
        public:
                    HRESULT Set_AAColorSource(
        __in_ecount(1) CHwColorComponentSource *pAAColorSource
        ) { abort(); }
};
class CHwVertexBuffer;
class CHwConstantColorSource {
       public:
            void GetColor(
        __out_ecount(1) MilColorF &color
        ) const { abort(); }
};
class CHwConstantAlphaScalableColorSource {};
class CHwTexturedColorSource {};


// mem.cp
inline void *GpMalloc(PERFMETERTAG mt, size_t size)
{
    return malloc(size);
}

/**************************************************************************\
*
* Function Description:
*
*   Frees a block of memory.
*
* Arguments:
*
*   [IN] memblock - block to free
*
* Notes:
*
*   If memblock is NULL, does nothing.
*
*
\**************************************************************************/

inline void GpFree(void *memblock)
{
    free(memblock);
}
#include "ShapeData.h"
// common.h
#include "CoordinateSpace.h"
#include "fix.h"
#include "Rect.h"
#include "matrix.h"
#include "ShapeBase.h"
#include "bezier.h"
// from RefCountBase.h
#define override
#include "hwrasterizer.h"
// from ziglang/zig/lib/libc/include/any-windows-any/ntdef.h
#define RTL_NUMBER_OF_V1(A) (sizeof(A)/sizeof((A)[0]))
#define RTL_NUMBER_OF_V2(A) RTL_NUMBER_OF_V1(A)
#define ARRAYSIZE(A)    RTL_NUMBER_OF_V2(A)
#define ARRAY_SIZE(a) (ARRAYSIZE(a))
#define D3DFVF_XYZ              0x002
#define D3DFVF_NORMAL           0x010
#define D3DFVF_DIFFUSE          0x040
#define D3DFVF_SPECULAR         0x080
#define D3DFVF_TEX2             0x200
#define D3DFVF_TEX4             0x400
#define D3DFVF_TEX6             0x600
#define D3DFVF_TEX8             0x800
typedef struct D3DVECTOR {
  float x;
  float y;
  float z;
} D3DVECTOR;

typedef DWORD D3DCOLOR;
#include "d3dvertex.h"
#include "MILRectF_WH.h"

//pixelformatutils.h
inline MilColorB
Convert_MilColorF_scRGB_To_Premultiplied_MilColorB_sRGB(
    const MilColorF * pColor    // scRGB MilColorF to convert
    ) { abort(); }

#include "Waffler.h"
#include "HwVertexBuffer.h"
class CHwPipeline {
        public:
        // This is public for the use of the vertex buffer builder to send
    // the device state when it flushes.
    HRESULT RealizeColorSourcesAndSendState(
        __in_ecount_opt(1) const CHwVertexBuffer *pVB
        ) { abort(); }
        enum {
        kGeneralScratchSpace =
            sizeof(CHwConstantAlphaScalableColorSource) +
            sizeof(CHwTexturedColorSource),

        kScratchAllocationSpace = kMaxVertexBuilderSize + kGeneralScratchSpace
    };
    CDispensableBuffer<kScratchAllocationSpace, 3> m_dbScratch;
    CD3DDeviceLevel1 *m_pDevice = nullptr;
};


class IDirect3DVertexBuffer9;
class IDirect3DIndexBuffer9;
class CHwD3DVertexBuffer {
        public:
            HRESULT Lock(
        UINT cVertices,
        UINT uVertexStride,
        __deref_out_bcount_part(cVertices * uVertexStride, 0) void ** const ppLockedVertices,
        __out_ecount(1) UINT * const puStartVertex
        ) { abort(); }

                HRESULT Unlock(
        UINT cVerticesUsed
        ) { abort(); }

                    __out_ecount(1) IDirect3DVertexBuffer9 *GetD3DBuffer() const
        { abort(); return nullptr; }
};
class CHwD3DIndexBuffer {
        public:
            HRESULT Lock(
        UINT cIndices,
        __deref_out_ecount_part(cIndices, 0) WORD ** const ppwLockedIndices,
        __out_ecount(1) UINT *puStartIndex
        ) { abort(); }
                HRESULT Unlock() {  abort(); }

                        __out_ecount(1) IDirect3DIndexBuffer9 *GetD3DBuffer() const
        { abort(); return nullptr; }
};
class CD3DDeviceLevel1{
        public:
    void GetClipRect(
        __out_ecount(1) MilPointAndSizeL * const prcClipRect
        ) const {
            *prcClipRect = clipRect;

    }
        void GetColorComponentSource(
        CHwColorComponentSource::VertexComponent eComponent,
        __deref_out_ecount(1) CHwColorComponentSource ** const ppColorComponentSource
        ) {abort(); }

            HRESULT DrawIndexedTriangleList(
        UINT uBaseVertexIndex,
        UINT uMinIndex,
        UINT cVertices,
        UINT uStartIndex,
        UINT cPrimitives
        ) { 
                    abort();
            }

                CHwTVertexBuffer<CD3DVertexXYZDUV2> *GetVB_XYZDUV2()
    {
        return &m_vBufferXYZDUV2;
    }

                    CHwTVertexBuffer<CD3DVertexXYZDUV8> *GetVB_XYZRHWDUV8()
    {
        return &m_vBufferXYZRHWDUV8;
    }

                    __out_ecount(1) CHwD3DVertexBuffer *Get3DVertexBuffer()
        { abort(); return m_pHwVertexBuffer; }

    __out_ecount(1) CHwD3DIndexBuffer *Get3DIndexBuffer()
        { abort(); return m_pHwIndexBuffer; }

                    MilPointAndSizeL GetViewport() const
        { abort(); return m_rcViewport; }
                  MIL_FORCEINLINE HRESULT SetStreamSource(
        __in_ecount_opt(1) IDirect3DVertexBuffer9 *pStream,
        UINT uVertexStride
        )
    {
            abort();
        return S_OK;

    }

                      MIL_FORCEINLINE HRESULT SetIndices(
        __in_ecount_opt(1) IDirect3DIndexBuffer9 *pStream
        )
    {
            abort();
            return S_OK;
    }
MilPointAndSizeL m_rcViewport;
                    CHwTVertexBuffer<CD3DVertexXYZDUV8> m_vBufferXYZRHWDUV8;
CHwTVertexBuffer<CD3DVertexXYZDUV2> m_vBufferXYZDUV2;

        MilPointAndSizeL clipRect;
          CHwD3DIndexBuffer *m_pHwIndexBuffer = nullptr;
    CHwD3DVertexBuffer *m_pHwVertexBuffer = nullptr;
};


#include "common/shared/utils.h"
#include <limits.h>

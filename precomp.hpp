    #define DeclareTag(tag, szOwner, szDescription)
    #define IsTagEnabled(tag) (FALSE)
#define     Mt(x)                               ((void)#x,0)
#define     MtDefine(tag, szOwner, szDescrip)
#define MIL_FORCEINLINE inline
#include <stdint.h>
     #include <stdlib.h>
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
HRESULT HrMalloc(
    PERFMETERTAG mt,
    size_t cbElementSize,
    size_t cElements,
    __deref_bcount(cbElementSize*cElements) void **ppvmemblock
    ) {
        *ppvmemblock = calloc(cbElementSize, cElements);
}
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

class CSpanSink;
class CSpanClipper;
class CAntialiasedFiller;
class CMILSurfaceRect;
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
class CD3DDeviceLevel1 {
        public:
    void GetClipRect(
        __out_ecount(1) MilPointAndSizeL * const prcClipRect
        ) const;
        void GetColorComponentSource(
        CHwColorComponentSource::VertexComponent eComponent,
        __deref_out_ecount(1) CHwColorComponentSource ** const ppColorComponentSource
        );
};

class CHwPipelineBuilder {
        public:
                    HRESULT Set_AAColorSource(
        __in_ecount(1) CHwColorComponentSource *pAAColorSource
        );
};
#include "ShapeData.h"
// common.h
#include "CoordinateSpace.h"
#include "fix.h"
#include "Rect.h"
#include "matrix.h"
#include "ShapeBase.h"
#define        WINCODEC_ERR_VALUEOVERFLOW  (0x80070216)
#include "bezier.h"
// from RefCountBase.h
#define override
#include "hwrasterizer.h"

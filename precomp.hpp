    #define DeclareTag(tag, szOwner, szDescription)
#define     MtDefine(tag, szOwner, szDescrip)
#define     Mt(x)                               g_mt##x
#define MIL_FORCEINLINE inline
#include <stdint.h>
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

struct  D3DXMATRIX  {
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


        union {
        struct {
            float        _11, _12, _13, _14;
            float        _21, _22, _23, _24;
            float        _31, _32, _33, _34;
            float        _41, _42, _43, _44;

        };
        float m[4][4];
    };
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
#include "wgx_render_types.h"
#include "wgx_error.h"
//#include "wgx_misc.h"
#include "spec_string.h"
typedef INT_PTR PERFMETERTAG;
#define     MtExtern(tag)                       extern PERFMETERTAG g_mt##tag;
#include "MILRect.h"
#include "BaseMatrix.h"
#include "MILMatrix.h"
#include "scanoperation.h"
#include "mem.h"

class CSpanSink;
class CSpanClipper;
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
#define INLINED_RRETURN(hr) return hr
#define RRETURN(x) return (x);
#define WHEN_DBG_ANALYSIS(x)
#define IFC(x) { hr = (x); if (FAILED(hr)) goto Cleanup; }
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
class CD3DDeviceLevel1;
#include "ShapeData.h"
// common.h
#include "CoordinateSpace.h"
#include "fix.h"
#include "Rect.h"
#include "matrix.h"
#include "ShapeBase.h"
// from RefCountBase.h
#define override
#include "hwrasterizer.h"

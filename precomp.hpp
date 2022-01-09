    #define DeclareTag(tag, szOwner, szDescription)
#define     MtDefine(tag, szOwner, szDescrip)
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
#include "wgx_render_types_generated.h"
//#include "wgx_misc.h"
#include "spec_string.h"
typedef INT_PTR PERFMETERTAG;
#define     MtExtern(tag)                       extern PERFMETERTAG g_mt##tag;
#include "MILMatrix.h"
#include "aarasterizer.h"

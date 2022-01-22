#include "precomp.hpp"

PALIMPORT
VOID
PALAPI
DebugBreak(
       VOID) {
        abort();
}

__declspec(noinline) void
DoStackCapture(
    HRESULT hr,         // HRESULT that caused this stack capture
    UINT uLine          // Line number the failure occured on.
    )
{
}

void CBaseMatrix::SetToIdentity()
{
    reset_to_identity();
}

struct Vec {
        float x, y;
};

float dot(Vec a, Vec b) {
        return a.x * b.x + a.y * b.y;
}

Vec perp(Vec v) {
        return Vec { -v.y, v.x };
}

HRESULT
CShapeBase::ConvertToGpPath(
    __out_ecount(1) DynArray<MilPoint2F> &rgPoints,
        // Path points
    __out_ecount(1) DynArray<BYTE>      &rgTypes,
        // Path types
    IN  bool                fStroking
        // Stroking if true, filling otherwise (optional)
    ) const
{
#if 1
        auto a = Vec{ 0., -1. };
        auto b = Vec { 1., 0. };
        auto r = 15.;
        auto r_sin_a = r * a.y;
        auto r_cos_a = r * a.x;
        auto r_sin_b = r * b.y;
        auto r_cos_b = r * b.x;

        auto mid  = Vec { a.x + b.x, a.y + b.y  };
        auto mid2 = Vec { a.x + mid.x, a.y + mid.y };
        auto xc = 10.;
        auto yc = 25.;
        auto h = (4. / 3. )* dot(perp(a), mid2)/ dot(a, mid2);
        rgPoints.Add({xc + r_cos_a, yc + r_sin_a});
        rgTypes.Add(PathPointTypeStart);

        rgPoints.Add({xc + r_cos_a - h * r_sin_a, yc + r_sin_a + h * r_cos_a});
        rgPoints.Add({        xc + r_cos_b + h * r_sin_b,        yc + r_sin_b - h * r_cos_b});
        rgPoints.Add({xc + r_cos_b, yc + r_sin_b});
        rgTypes.Add(PathPointTypeBezier);

        rgPoints.Add({xc, yc}); rgTypes.Add(PathPointTypeLine);

        rgPoints.Add({xc + r_cos_a, yc + r_sin_a}); rgTypes.Add(PathPointTypeLine);
        rgTypes.Add(PathPointTypeLine | PathPointTypeCloseSubpath);


#elif 0
        rgPoints.Add({10, 10});
        rgTypes.Add(PathPointTypeStart);

        rgPoints.Add({30, 10}); rgTypes.Add(PathPointTypeLine);
        rgPoints.Add({10, 30}); rgTypes.Add(PathPointTypeLine);
        rgPoints.Add({30, 30}); rgTypes.Add(PathPointTypeLine);


        rgPoints.Add({10, 10});
        rgTypes.Add(PathPointTypeLine | PathPointTypeCloseSubpath);
#else
        rgPoints.Add({10, 10});
        rgTypes.Add(PathPointTypeStart);

        rgPoints.Add({30, 10}); rgTypes.Add(PathPointTypeLine);
        rgPoints.Add({30, 30}); rgTypes.Add(PathPointTypeLine);
        rgPoints.Add({10, 30}); rgTypes.Add(PathPointTypeLine);


        rgPoints.Add({10, 10});
        rgTypes.Add(PathPointTypeLine | PathPointTypeCloseSubpath);
#endif
}

HRESULT
CShapeBase::GetTightBounds(
    __out_ecount(1) CMilRectF &rect,
        // The bounds of this shape
    __in_ecount_opt(1) const CPlainPen *pPen,
        // The pen (NULL OK)
    __in_ecount_opt(1) const CMILMatrix *pMatrix,
        // Transformation (NULL OK)
    __in double rTolerance,
        // Error tolerance (optional)
    __in bool fRelative,
        // True if the tolerance is relative (optional)
    __in bool fSkipHollows) const
        // If true, skip non-fillable figures when computing fill bound
{
        abort();
}

class RectShape : public CShapeBase {
        MilFillMode::Enum GetFillMode() const {
                        return MilFillMode::Alternate;
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
};

Heap* ProcessHeap;

template <class TVertex>
HRESULT CHwTVertexBuffer<TVertex>::SendVertexFormat(
        __inout_ecount(1) CD3DDeviceLevel1 *pDevice
        ) const
{
        abort();
}

template <class TVertex>
HRESULT CHwTVertexBuffer<TVertex>::DrawPrimitive(
        __inout_ecount(1) CD3DDeviceLevel1 *pDevice
        ) const
{
        abort();
}

template <>
HRESULT CHwTVertexBuffer<CD3DVertexXYZDUV2>::DrawPrimitive(
        __inout_ecount(1) CD3DDeviceLevel1 *pDevice
        ) const
{
        OutputVertex *output = (OutputVertex*)malloc(m_rgVerticesTriStrip.GetCount() * sizeof(OutputVertex));
        CD3DVertexXYZDUV2* data = m_rgVerticesTriStrip.GetDataBuffer();
        for (int i = 0; i < m_rgVerticesTriStrip.GetCount(); i++) {
                float color;
                memcpy(&color, &data[i].Diffuse, sizeof(color));
                output[i].x = data[i].X + 0.5;
                output[i].y = data[i].Y + 0.5;
                output[i].coverage = color;
        }
        pDevice->outputLen = m_rgVerticesTriStrip.GetCount();
        pDevice->output = output;
}


void output_obj_file(OutputVertex *data, size_t len) {
        for (size_t i = 0; i < len; i++) {
                float color = data[i].coverage;
                printf("v %f %f %f %f %f %f\n", data[i].x, data[i].y, 0., color, color, color);
        }

        // output a standard triangle strip face list
        for (int n = 1; n < len-1; n++) {
                if (n % 2 == 1) {
                        printf("f %d %d %d\n", n, n+1, n+2);
                } else {
                        printf("f %d %d %d\n", n+1, n, n+2);
                }
        }

}

int main() {
        CHwRasterizer rasterizer;
        CD3DDeviceLevel1 device;
        device.clipRect.X = 0;
        device.clipRect.Y = 0;
        device.clipRect.Width = 100;
        device.clipRect.Height = 100;
        device.m_rcViewport = device.clipRect;
        DynArray<MilPoint2F> pointsScratch;
        DynArray<BYTE> typesScratch;
        RectShape shape;
        CMatrix<CoordinateSpace::Shape,CoordinateSpace::Device> worldToDevice(true);

        rasterizer.Setup(&device, &shape, &pointsScratch, &typesScratch, &worldToDevice);
        MilVertexFormat m_mvfIn;
        MilVertexFormat m_mvfGenerated = MILVFAttrNone;
        MilVertexFormatAttribute mvfaAALocation = MILVFAttrNone;
#define HWPIPELINE_ANTIALIAS_LOCATION MILVFAttrDiffuse
        mvfaAALocation = HWPIPELINE_ANTIALIAS_LOCATION;
        rasterizer.GetPerVertexDataType(m_mvfIn);
        CHwVertexBuffer::Builder *vertexBuilder;
        CHwPipeline pipeline;
        pipeline.m_pDevice  = &device;
        CHwPipeline * const m_pHP = &pipeline;
        CHwVertexBuffer::Builder::Create(
                                         m_mvfIn,
                                         m_mvfIn | m_mvfGenerated,
                                         mvfaAALocation,
                                         m_pHP,
                                         m_pHP->m_pDevice,
                                         &m_pHP->m_dbScratch,
                                         &vertexBuilder
                                        );
        vertexBuilder->BeginBuilding();
        rasterizer.SendGeometry(vertexBuilder);
        CHwVertexBuffer *m_pVB;
        vertexBuilder->FlushTryGetVertexBuffer(&m_pVB);
        delete vertexBuilder;
        output_obj_file(device.output, device.outputLen);
}

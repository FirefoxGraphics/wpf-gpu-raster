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
        rgPoints.Add({10, 10});
        rgTypes.Add(PathPointTypeStart);

        rgPoints.Add({10, 28});
        rgTypes.Add(PathPointTypeLine);

        rgPoints.Add({30, 30});
        rgTypes.Add(PathPointTypeLine);

        rgPoints.Add({30, 10});
        rgTypes.Add(PathPointTypeLine);


        rgPoints.Add({10, 10});
        rgTypes.Add(PathPointTypeLine | PathPointTypeCloseSubpath);

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
class Sink : public IGeometrySink {
        virtual HRESULT AddVertex(
        __in_ecount(1) const MilPoint2F &ptPosition,
            // In: Vertex coordinates
        __out_ecount(1) WORD *pidxOut
            // Out: Index of vertex
        ) { abort(); }

    virtual HRESULT AddIndexedVertices(
        UINT cVertices,
            // In: number of vertices
        __in_bcount(cVertices*uVertexStride) const void *pVertexBuffer,
            // In: vertex buffer containing the vertices
        UINT uVertexStride,
            // In: size of each vertex
        MilVertexFormat mvfFormat,
            // In: format of each vertex
        UINT cIndices,
            // In: Number of indices
        __in_ecount(cIndices) const UINT *puIndexBuffer
            // In: index buffer                                                             
        ) { abort(); }

    virtual void SetTransformMapping(
        __in_ecount(1) const MILMatrix3x2 &mat2DTransform
        ) { abort(); }

    virtual HRESULT AddTriangle(
        DWORD idx1,
            // In: Index of triangle's first vertex
        DWORD idx2,
            // In: Index of triangle's second vertex
        DWORD idx3
            // In: Index of triangle's third vertex
        ) { abort(); }

    //
    // Trapezoidal AA geometry output
    //

    virtual HRESULT AddComplexScan(
        INT nPixelY,
            // In: y coordinate in pixel space
        __in_ecount(1) const CCoverageInterval *pIntervalSpanStart
            // In: coverage segments
        ) { abort(); }
    
    virtual HRESULT AddTrapezoid(
        float rYMin,
            // In: y coordinate of top of trapezoid
        float rXLeftYMin,
            // In: x coordinate for top left
        float rXRightYMin,
            // In: x coordinate for top right
        float rYMax,
            // In: y coordinate of bottom of trapezoid
        float rXLeftYMax,
            // In: x coordinate for bottom left
        float rXRightYMax,
            // In: x coordinate for bottom right
        float rXDeltaLeft,
            // In: trapezoid expand radius
        float rXDeltaRight
            // In: trapezoid expand radius
        ) { printf("AddTrap: %f %f %f %f %f %f\n", rYMin, rXLeftYMin, rXRightYMin, rYMax, rXLeftYMax, rXRightYMax);
        empty = false;
        }
    
    virtual HRESULT AddParallelogram(
        __in_ecount(4) const MilPoint2F *rgPosition
        ) { abort(); }
    
    //
    // Query sink status
    //

    // Some geometry generators don't actually know if they have output
    // any triangles, so they need to get this information from the geometry sink.

    virtual BOOL IsEmpty() { return empty; }
        bool empty = true;

};

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

int main() {
        CHwRasterizer rasterizer;
        CD3DDeviceLevel1 device;
        device.clipRect.X = 0;
        device.clipRect.Y = 0;
        device.clipRect.Width = 100;
        device.clipRect.Height = 100;
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
        delete vertexBuilder;
}

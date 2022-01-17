#include "precomp.hpp"


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

        rgPoints.Add({10, 30});
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
        CMatrix<CoordinateSpace::Shape,CoordinateSpace::Device> worldToDevice;

        rasterizer.Setup(&device, &shape, &pointsScratch, &typesScratch, &worldToDevice);
}

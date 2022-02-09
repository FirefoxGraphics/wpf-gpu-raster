// Licensed to the .NET Foundation under one or more agreements.
// The .NET Foundation licenses this file to you under the MIT license.
// See the LICENSE file in the project root for more information.

type LONG = i32;
type INT = i32;
type LONGLONG = i64;
type BYTE = u8;
type FLOAT = f32;
type REAL = FLOAT;
use crate::bezier::CMILBezier;
//+----------------------------------------------------------------------------
//

//
//  Description:  Code for rasterizing the fill of a path.
//
//  >>>> Note that some of this code is duplicated in hw\hwrasterizer.cpp,
//  >>>> so changes to this file may need to propagate.
//
//   pursue reduced code duplication
//


// This option may potentially increase performance for many
// paths that have edges adjacent at their top point and cover
// more than one span.  The code has been tested, but performance
// has not been thoroughly investigated.
const SORT_EDGES_INCLUDING_SLOPE: bool = false;


/////////////////////////////////////////////////////////////////////////
// The x86 C compiler insists on making a divide and modulus operation
// into two DIVs, when it can in fact be done in one.  So we use this
// macro.
//
// Note: QUOTIENT_REMAINDER implicitly takes unsigned arguments.
//
// QUOTIENT_REMAINDER_64_32 takes a 64-bit numerator and produces 32-bit
// results.

macro_rules! QUOTIENT_REMAINDER {
    ($ulNumerator: ident, $ulDenominator: ident, $ulQuotient: ident, $ulRemainder: ident) => {
        $ulQuotient  = ($ulNumerator as ULONG) / ($ulDenominator as ULONG);
        $ulRemainder = ($ulNumerator as ULONG) % ($ulDenominator as ULONG);
    }
}

macro_rules! QUOTIENT_REMAINDER_64_32 {
    ($ulNumerator: ident, $ulDenominator: ident, $ulQuotient: ident, $ulRemainder: ident) => {
        $ulQuotient  = (($ulNumerator as ULONGLONG) / ($ulDenominator as ULONG)) as ULONGLONG;
        $ulRemainder = (($ulNumerator as ULONGLONG) % ($ulDenominator as ULONG)) as ULONGLONG;
    }
}

// SWAP macro:
macro_rules! SWAP {
    ($temp: ident, $a: expr, $b: expr) => { $temp = $a; $a = $b; $b = $temp; }
}

/**************************************************************************\
*
* Function Description:
*
*   The edge initializer is out of room in its current 'store' buffer;
*   get it a new one.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

impl CEdgeStore {
    fn NextAddBuffer(&self,
    /*__deref_out_ecount(*puRemaining)*/ ppCurentEdge: &mut &[CEdge],
    puRemainging: &mut UINT
    )
{
    let hr = S_OK;

    let cNewTotalCount = 0;

    // The caller has completely filled up this chunk:

    assert!(*puRemaining == 0);

    // Check to make sure that "TotalCount" will be able to represent the current capacity
    cNewTotalCount = self.TotalCount + self.CurrentBuffer.Count;

    if (cNewTotalCount < self.TotalCount)
    {
        return WINCODEC_ERR_VALUEOVERFLOW;
    }

    // And that it can represent the new capacity as well, with at least 2 to spare.
    // This "magic" 2 comes from the fact that the usage pattern of this class has callers
    // needing to allocate space for TotalCount + 2 edges.
    if (cNewTotalCount + (UINT)(EDGE_STORE_ALLOCATION_NUMBER + 2) < cNewTotalCount)
    {
        return WINCODEC_ERR_VALUEOVERFLOW;
    }

    // We have to grow our data structure by adding a new buffer
    // and adding it to the list:

    CEdgeAllocation *newBuffer;
    newBuffer = static_cast<CEdgeAllocation*>
        (GpMalloc(Mt(MAARasterizerEdge),
                  sizeof(CEdgeAllocation) +
                  sizeof(CEdge) * (EDGE_STORE_ALLOCATION_NUMBER
                                  - EDGE_STORE_STACK_NUMBER)));
    IFCOOM(newBuffer);

    newBuffer.Next = NULL;
    newBuffer.Count = EDGE_STORE_ALLOCATION_NUMBER;

    self.TotalCount = cNewTotalCount;

    self.CurrentBuffer.Next = newBuffer;
    self.CurrentBuffer = newBuffer;

    *ppCurrentEdge = self.CurrentEdge = &newBuffer.EdgeArray[0];
    *puRemaining = self.CurrentRemaining = EDGE_STORE_ALLOCATION_NUMBER;

    return hr
}
}


/**************************************************************************\
*
* Function Description:
*
*   Some debug code for verifying the state of the active edge list.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn 
AssertActiveList(
    mut list: *const CEdge,
    yCurrent: INT
    ) -> bool
{
    let b = true;
    let activeCount = 0;

    assert!(list.X == INT::MIN);
    b &= (list.X == INT::MIN);

    // Skip the head sentinel:

    list = list.Next;

    while (list.X != INT::MAX)
    {
        assert!(list.X != INT::MIN);
        b &= (list.X != INT::MIN);

        assert!(list.X <= list.Next.X);
        b &= (list.X <= list.Next.X);

        assert!((list.StartY <= yCurrent) && (yCurrent < list.EndY));
        b &= ((list.StartY <= yCurrent) && (yCurrent < list.EndY));

        activeCount+=1;
        list = list.Next;
    }

    assert!(list.X == INT::MAX);
    b &= (list.X == INT::MAX);

    // There should always be a multiple of 2 edges in the active list.
    //
    // NOTE: If you hit this assert, do NOT simply comment it out!
    //       It usually means that all the edges didn't get initialized
    //       properly.  For every scan-line, there has to be a left edge
    //       and a right edge (or a multiple thereof).  So if you give
    //       even a single bad edge to the edge initializer (or you miss
    //       one), you'll probably hit this assert.

    assert!((activeCount & 1) == 0);
    b &= ((activeCount & 1) == 0);

    return(b);
}

/**************************************************************************\
*
* Function Description:
*
*   Some debug code for verifying the state of the active edge list.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn
AssertActiveListOrder(
    list: *const CEdge
    )
{
    let activeCount = 0;

    assert!(list.X == INT::MIN);

    // Skip the head sentinel:

    list = list.Next;

    while (list.X != INT::MAX)
    {
        assert!(list.X != INT::MIN);
        assert!(list.X <= list.Next.X);

        activeCount += 1;
        list = list.Next;
    }

    assert!(list.X == INT::MAX);
}


/**************************************************************************\
*
* Function Description:
*
*   Clip the edge vertically.
*
*   We've pulled this routine out-of-line from InitializeEdges mainly
*   because it needs to call inline Asm, and when there is in-line
*   Asm in a routine the compiler generally does a much less efficient
*   job optimizing the whole routine.  InitializeEdges is rather
*   performance critical, so we avoid polluting the whole routine
*   by having this functionality out-of-line.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/
fn
ClipEdge(
    edgeBuffer: *const CEdge,
    yClipTopInteger: INT,
    dMOriginal: INT
    )
{
    let xDelta;
    let error;

    // Cases where bigNumerator will exceed 32-bits in precision
    // will be rare, but could happen, and we can't fall over
    // in those cases.

    let dN = edgeBuffer.ErrorDown;
    let bigNumerator = Int32x32To64(dMOriginal,
                                         yClipTopInteger - edgeBuffer.StartY)
                          + (edgeBuffer.Error + dN);
    if (bigNumerator >= 0)
    {
        QUOTIENT_REMAINDER_64_32(bigNumerator, dN, xDelta, error);
    }
    else
    {
        bigNumerator = -bigNumerator;
        QUOTIENT_REMAINDER_64_32(bigNumerator, dN, xDelta, error);

        xDelta = -xDelta;
        if (error != 0)
        {
            xDelta -= 1;
            error = dN - error;
        }
    }

    // Update the edge data structure with the results:

    edgeBuffer.StartY  = yClipTopInteger;
    edgeBuffer.X      += xDelta;
    edgeBuffer.Error   = error - dN;      // Renormalize error
}
//+-----------------------------------------------------------------------------
//
//  Function:  TransformRasterizerPointsTo28_4
//
//  Synopsis:
//      Transform rasterizer points to 28.4.  If overflow occurs, return that
//      information.
//
//------------------------------------------------------------------------------
fn TransformRasterizerPointsTo28_4(
    pmat: &CMILMatrix,
        // Transform to take us to 28.4
    mut pPtsSource: &[MilPoint2F],
        // Source points
    cPoints: UINT,
        // Count of points
    mut pPtsDest: &mut [POINT]
        // Destination points
    ) -> HRESULT
{
    let hr = S_OK;

    assert!(cPoints > 0);

    //
    // We want coordinates in the 28.4 range in the end.  The matrix we get
    // as input includes the scale by 16 to get to 28.4, so we want to
    // ensure that we are in integer range.  Assuming a sign bit and
    // five bits for the rasterizer working range, we want coordinates in the
    // -2^26 to 2^26.
    //
    // Note that the 5-bit requirement comes from the
    // implementation of InitializeEdges.
    // (See line with "error -= dN * (16 - (xStart & 15))")
    //
    // Anti-aliasing uses another c_nShift bits, so we get a
    // desired range of -2^(26-c_nShift) to 2^(26-c_nShift)
    //

    let rPixelCoordinateMax = (1 << (26 - c_nShift)) as f32;
    let rPixelCoordinateMin = -rPixelCoordinateMax;

    while
    {
        //
        // Transform coordinates
        //

        let rPixelX = (pmat.GetM11() * pPtsSource.X) + (pmat.GetM21() * pPtsSource.Y) + pmat.GetDx();
        let rPixelY = (pmat.GetM12() * pPtsSource.X) + (pmat.GetM22() * pPtsSource.Y) + pmat.GetDy();

        //
        // Check for NaNs or overflow
        //

        if (!(rPixelX <= rPixelCoordinateMax &&
            rPixelX >= rPixelCoordinateMin &&
            rPixelY <= rPixelCoordinateMax &&
            rPixelY >= rPixelCoordinateMin))
        {
            return WGXERR_BADNUMBER;
        }

        //
        // Assign coordinates
        //

        pPtsDest.x = CFloatFPU::Round(rPixelX);
        pPtsDest.y = CFloatFPU::Round(rPixelY);

        pPtsDest = &pPtsDest[1..];
        pPtsSource = &pPtsSource[1..];
        cPoints -= 1;
        cPoints != 0
    } {}


    return hr
}

fn AppendScaleToMatrix(
    pmat: &mut CMILMatrix,
    scaleX: REAL,
    scaleY: REAL
    )
{
    pmat.SetM11(pmat.GetM11() * scaleX);
    pmat.SetM21(pmat.GetM21() * scaleX);
    pmat.SetM12(pmat.GetM12() * scaleY);
    pmat.SetM22(pmat.GetM22() * scaleY);
    pmat.SetDx(pmat.GetDx() * scaleX);
    pmat.SetDy(pmat.GetDy() * scaleY);
}

/**************************************************************************\
*
* Function Description:
*
*   Add edges to the edge list.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn
InitializeEdges(
    pEdgeContext: &mut CInitializeEdgesContext,
    /*__inout_ecount(vertexCount)*/ mut pointArray: &mut [POINT],    // Points to a 28.4 array of size 'vertexCount'
                                                   //   Note that we may modify the contents!
    /*__in_range(>=, 2)*/  vertexCount: UINT
    ) -> HRESULT
{
    // Disable instrumentation checks for this function
    //SET_MILINSTRUMENTATION_FLAGS(MILINSTRUMENTATIONFLAGS_DONOTHING);

    let hr = S_OK;

    let xStart;
    let yStart;
    let yStartInteger;
    let yEndInteger;
    let dMOriginal;
    let dM;
    let dN;
    let dX;
    let errorUp;
    let quotient;
    let remainder;
    let error;
    let windingDirection;
    let edgeBuffer: *const CEdge;
    let bufferCount: UINT;
    let yClipTopInteger;
    let yClipTop;
    let yClipBottom;
    let xClipLeft;
    let xClipRight;

    let yMax              = pEdgeContext.MaxY;
    let store     = pEdgeContext.Store;
    let clipRect  = pEdgeContext.ClipRect;

    let edgeCount = vertexCount - 1;
    assert!(edgeCount >= 1);

    if (clipRect != NULL)
    {
        yClipTopInteger = clipRect.top >> 4;
        yClipTop = clipRect.top;
        yClipBottom = clipRect.bottom;
        xClipLeft = clipRect.left;
        xClipRight = clipRect.right;

        assert!(yClipBottom > 0);
        assert!(yClipTop <= yClipBottom);
    } else {
        yClipBottom = 0;
        yClipTopInteger = INT::MIN >> c_nShift;

        // These 3 values are only used when clipRect is non-NULL
        yClipTop = 0;
        xClipLeft = 0;
        xClipRight = 0;

    }

    if (pEdgeContext.AntiAliasMode != MilAntiAliasMode::None)
    {
        // If antialiasing, apply the supersampling scaling here before we
        // calculate the DDAs.  We do this here and not in the Matrix
        // transform we give to FixedPointPathEnumerate mainly so that the
        // Bezier flattener can continue to operate in its optimal 28.4
        // format.
        //
        // PS#856364-2003/07/01-JasonHa  Remove pixel center fixup
        //
        // We also apply a half-pixel offset here so that the antialiasing
        // code can assume that the pixel centers are at half-pixel
        // coordinates, not on the integer coordinates.

        POINT *point = pointArray;
        INT i = vertexCount;

        do {
            point.x = (point.x + 8) << c_nShift;
            point.y = (point.y + 8) << c_nShift;

        } while (point++, --i != 0);

        yClipTopInteger <<= c_nShift;
        yClipTop <<= c_nShift;
        yClipBottom <<= c_nShift;
        xClipLeft <<= c_nShift;
        xClipRight <<= c_nShift;
    }

    // Make 'yClipBottom' inclusive by subtracting off one pixel
    // (keeping in mind that we're in 28.4 device space):

    yClipBottom -= 16;

    // Warm up the store where we keep the edge data:

    store.StartAddBuffer(&edgeBuffer, &bufferCount);

    while {
        // Handle trivial rejection:

        if (yClipBottom >= 0)
        {
            // Throw out any edges that are above or below the clipping.
            // This has to be a precise check, because we assume later
            // on that every edge intersects in the vertical dimension
            // with the clip rectangle.  That asssumption is made in two
            // places:
            //
            // 1.  When we sort the edges, we assume either zero edges,
            //     or two or more.
            // 2.  When we start the DDAs, we assume either zero edges,
            //     or that there's at least one scan of DDAs to output.
            //
            // Plus, of course, it's less efficient if we let things
            // through.
            //
            // Note that 'yClipBottom' is inclusive:

            let clipHigh = ((pointArray).y <= yClipTop) &&
                            ((pointArray + 1).y <= yClipTop);

            let clipLow = ((pointArray).y > yClipBottom) &&
                             ((pointArray + 1).y > yClipBottom);

            #[cfg(debug)]
            {
                let (yRectTop, yRectBottom, y0, y1, yTop, yBottom);

                // Getting the trivial rejection code right is tricky.
                // So on checked builds let's verify that we're doing it
                // correctly, using a different approach:

                let clipped = false;
                if (clipRect != NULL)
                {
                    yRectTop = clipRect.top >> 4;
                    yRectBottom = clipRect.bottom >> 4;
                    if (pEdgeContext.AntiAliasMode != MilAntiAliasMode::None)
                    {
                        yRectTop <<= c_nShift;
                        yRectBottom <<= c_nShift;
                    }
                    y0 = ((pointArray).y + 15) >> 4;
                    y1 = ((pointArray + 1).y + 15) >> 4;
                    yTop = min(y0, y1);
                    yBottom = max(y0, y1);

                    clipped = ((yTop >= yRectBottom) || (yBottom <= yRectTop));
                }

                assert!(clipped == (clipHigh || clipLow));
            }

            if (clipHigh || clipLow) {
                continue;               // ======================>
            }

            if (edgeCount > 1)
            {
                // Here we'll collapse two edges down to one if both are
                // to the left or to the right of the clipping rectangle.

                if (((pointArray[0]).x < xClipLeft) &&
                    ((pointArray[1]).x < xClipLeft) &&
                    ((pointArray[2]).x < xClipLeft))
                {
                    // Note this is one reason why 'pointArray' can't be 'const':

                    *(pointArray[1]) = *(pointArray[0]);

                    continue;           // ======================>
                }

                if (((pointArray[0]).x > xClipRight) &&
                    ((pointArray[1]).x > xClipRight) &&
                    ((pointArray[2]).x > xClipRight))
                {
                    // Note this is one reason why 'pointArray' can't be 'const':

                    pointArray[1] = pointArray[0];

                    continue;           // ======================>
                }
            }

        }

        dM = (pointArray[1]).x - (pointArray[0]).x;
        dN = (pointArray[1]).y - (pointArray[0]).y;

        if (dN >= 0)
        {
            // The vector points downward:

            xStart = (pointArray[0]).x;
            yStart = (pointArray[0]).y;

            yStartInteger = (yStart + 15) >> 4;
            yEndInteger   = ((pointArray[1]).y + 15) >> 4;

            windingDirection = 1;
        }
        else
        {
            // The vector points upward, so we have to essentially
            // 'swap' the end points:

            dN = -dN;
            dM = -dM;

            xStart = (pointArray[1]).x;
            yStart = (pointArray[1]).y;

            yStartInteger = (yStart + 15) >> 4;
            yEndInteger   = ((pointArray[0]).y + 15) >> 4;

            windingDirection = -1;
        }

        // The edgeBuffer must span an integer y-value in order to be
        // added to the edgeBuffer list.  This serves to get rid of
        // horizontal edges, which cause trouble for our divides.

        if (yEndInteger > yStartInteger)
        {
            yMax = max(yMax, yEndInteger);

            dMOriginal = dM;
            if (dM < 0)
            {
                dM = -dM;
                if (dM < dN)            // Can't be '<='
                {
                    dX      = -1;
                    errorUp = dN - dM;
                }
                else
                {
                    QUOTIENT_REMAINDER(dM, dN, quotient, remainder);

                    dX      = -quotient;
                    errorUp = remainder;
                    if (remainder > 0)
                    {
                        dX      = -quotient - 1;
                        errorUp = dN - remainder;
                    }
                }
            }
            else
            {
                if (dM < dN)
                {
                    dX      = 0;
                    errorUp = dM;
                }
                else
                {
                    QUOTIENT_REMAINDER(dM, dN, quotient, remainder);

                    dX      = quotient;
                    errorUp = remainder;
                }
            }

            error = -1;     // Error is initially zero (add dN - 1 for
                            //   the ceiling, but subtract off dN so that
                            //   we can check the sign instead of comparing
                            //   to dN)

            if ((yStart & 15) != 0)
            {
                // Advance to the next integer y coordinate

                for (INT i = 16 - (yStart & 15); i != 0; i--)
                {
                    xStart += dX;
                    error += errorUp;
                    if (error >= 0)
                    {
                        error -= dN;
                        xStart += 1;
                    }
                }
            }

            if ((xStart & 15) != 0)
            {
                error -= dN * (16 - (xStart & 15));
                xStart += 15;       // We'll want the ceiling in just a bit...
            }

            xStart >>= 4;
            error >>= 4;

            if (bufferCount == 0)
            {
                IFC(store.NextAddBuffer(&edgeBuffer, &bufferCount));
            }

            edgeBuffer.X                = xStart;
            edgeBuffer.Dx               = dX;
            edgeBuffer.Error            = error;
            edgeBuffer.ErrorUp          = errorUp;
            edgeBuffer.ErrorDown        = dN;
            edgeBuffer.WindingDirection = windingDirection;
            edgeBuffer.StartY           = yStartInteger;
            edgeBuffer.EndY             = yEndInteger;       // Exclusive of end

            assert!(error < 0);

            // Here we handle the case where the edge starts above the
            // clipping rectangle, and we need to jump down in the 'y'
            // direction to the first unclipped scan-line.
            //
            // Consequently, we advance the DDA here:

            if (yClipTopInteger > yStartInteger)
            {
                assert!(edgeBuffer.EndY > yClipTopInteger);

                ClipEdge(edgeBuffer, yClipTopInteger, dMOriginal);
            }

            // Advance to handle the next edge:

            edgeBuffer += 1;
            bufferCount -= 1;
        }
        pointArray = &pointArray[1..];
        edgeCount -= 1;
        edgeCount != 0
    } {}

    // We're done with this batch.  Let the store know how many edges
    // we ended up with:

    println("bufferCount {}", bufferCount);
    store.EndAddBuffer(edgeBuffer, bufferCount);

    pEdgeContext.MaxY = yMax;

    return hr;
}

/**************************************************************************\
*
* Function Description:
*
*   Does complete parameter checking on the 'types' array of a path.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/
fn
ValidatePathTypes(
    typesArray: &[BYTE],
    count: INT
    ) -> bool
{
    let mut types = typesArray;

    if (count == 0) {
        return(true);
    }

    loop
    {
        // The first point in every subpath has to be an unadorned
        // 'start' point:

        if ((types[0] & PathPointTypePathTypeMask) != PathPointTypeStart)
        {
            TraceTag((tagMILWarning, "Bad subpath start"));
            return(false);
        }

        // Advance to the first point after the 'start' point:
        count -= 1;
        if (count == 0)
        {
            TraceTag((tagMILWarning, "Path ended after start-path"));
            return(false);
        }

        if ((types[1] & PathPointTypePathTypeMask) == PathPointTypeStart)
        {
            TraceTag((tagMILWarning, "Can't have a start followed by a start!"));
            return(false);
        }

        // Process runs of lines and Bezier curves:

        loop {
            match(types[1] & PathPointTypePathTypeMask)
            {
                PathPointTypeLine => {
                    types = &types[1..];
                    count -= 1;
                    if (count == 0) {
                        return(TRUE);
                    }

                }

                PathPointTypeBezier => {
                    if(count < 3)
                    {
                        TraceTag((tagMILWarning,
                            "Path ended before multiple of 3 Bezier points"));
                        return(false);
                    }

                    if((types[1] & PathPointTypePathTypeMask) != PathPointTypeBezier)
                    {
                        TraceTag((tagMILWarning,
                            "Bad subpath start"));
                        return(false);
                    }

                    if((types[2] & PathPointTypePathTypeMask) != PathPointTypeBezier)
                    {
                        TraceTag((tagMILWarning,
                            "Expected plain Bezier control point for 3rd vertex"));
                        return(false);
                    }

                    if((types[3] & PathPointTypePathTypeMask) != PathPointTypeBezier)
                    {
                        TraceTag((tagMILWarning,
                            "Expected Bezier control point for 4th vertex"));
                        return(false);
                    }

                    types = &types[3..];
                    count -= 3;
                    if (count == 0) {
                        return(true);
                    }

                }

                _ => {
                    TraceTag((tagMILWarning, "Illegal type"));
                    return(false);
                }
            }

            // A close-subpath marker or a start-subpath marker marks the
            // end of a subpath:
            !(types[0] & PathPointTypeCloseSubpath) &&
            ((types[1] & PathPointTypePathTypeMask) != PathPointTypeStart)
        } {};
    }
}

/**************************************************************************\
*
* Function Description:
*
*   Some debug code for verifying the path.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn
AssertPath(
    rgTypes: &[BYTE],
    cPoints: UINT
    )
{
    // Make sure that the 'types' array is well-formed, otherwise we
    // may fall over in the FixedPointPathEnumerate function.
    //
    // NOTE: If you hit this assert, DO NOT SIMPLY COMMENT THIS Assert OUT!
    //
    //       Instead, fix the ValidatePathTypes code if it's letting through
    //       valid paths, or (more likely) fix the code that's letting bogus
    //       paths through.  The FixedPointPathEnumerate routine has some
    //       subtle assumptions that require the path to be perfectly valid!
    //
    //       No internal code should be producing invalid paths, and all
    //       paths created by the application must be parameter checked!
    assert!(ValidatePathTypes(rgTypes, cPoints));
}

//+----------------------------------------------------------------------------
//
//  Member:
//      FixedPointPathEnumerate
//
//  Synopsis:
//
//      Enumerate the path.
//
//      NOTE: The 'enumerateFunction' function is allowed to modify the
//            contents of our call-back buffer!  (This is mainly done to allow
//            'InitializeEdges' to be simpler for some clipping trivial
//            rejection cases.)
//
//      NOTICE-2006/03/22-milesc  This function was initially built to be a
//      general path enumeration function. However, we were only using it for
//      one specific purpose... for Initializing edges of a path to be filled.
//      In doing security work, I simplified this function to just do edge
//      initialization. The name is therefore now overly general. I have kept
//      the name to be a reminder that this function has been written to be
//      more general than would otherwise be evident.
//

fn
FixedPointPathEnumerate(
    rgpt: &[MilPoint2F],
    rgTypes: &[BYTE],
    cPoints: UINT,
    matrix: &CMILMatrix,
    clipRect: &RECT,       // In scaled 28.4 format
    enumerateContext: &mut CInitializeEdgesContext
    ) -> HRESULT
{
    let hr = S_OK;
    let bufferStart: [POINT; ENUMERATE_BUFFER_NUMBER];
    let bezierBuffer: [POINT; 4];
    let buffer: &POINT;
    let bufferSize: UINT;
    let startFigure: POINT;
    let iStart: UINT;
    let iEnd: UINT;
    let runSize: UINT;
    let thisCount: UINT;
    let isMore: bool;
    let xLast: INT;
    let yLast: INT;

    ASSERTPATH(rgTypes, cPoints);

    // Every valid subpath has at least two vertices in it, hence the
    // check of 'cPoints - 1':

    iStart = 0;

    assert!(cPoints > 1);
    while (iStart < cPoints - 1)
    {
        assert!((rgTypes[iStart] & PathPointTypePathTypeMask)
                    == PathPointTypeStart);
        assert!((rgTypes[iStart + 1] & PathPointTypePathTypeMask)
                    != PathPointTypeStart);

        // Add the start point to the beginning of the batch, and
        // remember it for handling the close figure:

        IFC(TransformRasterizerPointsTo28_4(matrix, &rgpt[iStart..], 1, &startFigure));

        bufferStart[0].x = startFigure.x;
        bufferStart[0].y = startFigure.y;
        buffer = bufferStart + 1;
        bufferSize = ENUMERATE_BUFFER_NUMBER - 1;

        // We need to enter our loop with 'iStart' pointing one past
        // the start figure:

        iStart += 1;

        while {
            // Try finding a run of lines:

            if ((rgTypes[iStart] & PathPointTypePathTypeMask)
                                == PathPointTypeLine)
            {
                iEnd = iStart + 1;

                while ((iEnd < cPoints) &&
                       ((rgTypes[iEnd] & PathPointTypePathTypeMask)
                                == PathPointTypeLine))
                {
                    iEnd += 1;
                }

                // Okay, we've found a run of lines.  Break it up into our
                // buffer size:

                runSize = (iEnd - iStart);
                loop {
                    thisCount = min(bufferSize, runSize);

                    IFC(TransformRasterizerPointsTo28_4(matrix, &rgpt[iStart], thisCount, buffer));

                    __analysis_assume(buffer + bufferSize == bufferStart + ENUMERATE_BUFFER_NUMBER);
                    Assert(buffer + bufferSize == bufferStart + ENUMERATE_BUFFER_NUMBER);

                    iStart += thisCount;
                    buffer += thisCount;
                    runSize -= thisCount;
                    bufferSize -= thisCount;

                    if (bufferSize > 0) {
                        break;
                    }

                    xLast = bufferStart[ENUMERATE_BUFFER_NUMBER - 1].x;
                    yLast = bufferStart[ENUMERATE_BUFFER_NUMBER - 1].y;
                    IFC(InitializeEdges(
                        enumerateContext,
                        bufferStart,
                        ENUMERATE_BUFFER_NUMBER
                        ));

                    // Continue the last vertex as the first in the new batch:

                    bufferStart[0].x = xLast;
                    bufferStart[0].y = yLast;
                    buffer = bufferStart + 1;
                    bufferSize = ENUMERATE_BUFFER_NUMBER - 1;
                    if !(runSize != 0) { break }
                }
            }
            else
            {
                assert!(iStart + 3 <= cPoints);
                assert!((rgTypes[iStart] & PathPointTypePathTypeMask)
                        == PathPointTypeBezier);
                assert!((rgTypes[iStart + 1] & PathPointTypePathTypeMask)
                        == PathPointTypeBezier);
                assert!((rgTypes[iStart + 2] & PathPointTypePathTypeMask)
                        == PathPointTypeBezier);

                IFC(TransformRasterizerPointsTo28_4(matrix, &rgpt[iStart - 1], 4, &bezierBuffer));

                // Prepare for the next iteration:

                iStart += 3;

                // Process the Bezier:

                let bezier = CMILBezier::new(bezierBuffer, clipRect);
                while {
                    thisCount = bezier.Flatten(buffer, bufferSize, &isMore);

                    __analysis_assume(buffer + bufferSize == bufferStart + ENUMERATE_BUFFER_NUMBER);
                    Assert(buffer + bufferSize == bufferStart + ENUMERATE_BUFFER_NUMBER);

                    buffer += thisCount;
                    bufferSize -= thisCount;

                    if (bufferSize > 0) {
                        break;
                    }

                    xLast = bufferStart[ENUMERATE_BUFFER_NUMBER - 1].x;
                    yLast = bufferStart[ENUMERATE_BUFFER_NUMBER - 1].y;
                    IFC(InitializeEdges(
                        enumerateContext,
                        bufferStart,
                        ENUMERATE_BUFFER_NUMBER
                        ));

                    // Continue the last vertex as the first in the new batch:

                    bufferStart[0].x = xLast;
                    bufferStart[0].y = yLast;
                    buffer = bufferStart + 1;
                    bufferSize = ENUMERATE_BUFFER_NUMBER - 1;
                    if !isMore { break }
                } {};
            }

        ((iStart < cPoints) &&
                 ((rgTypes[iStart] & PathPointTypePathTypeMask)
                    != PathPointTypeStart))
        } {};

        // Okay, the subpath is done.  But we still have to handle the
        // 'close figure' (which is implicit for a fill):
        // Add the close-figure point:

        buffer.x = startFigure.x;
        buffer.y = startFigure.y;
        bufferSize -= 1;

        // We have to flush anything we might have in the batch, unless
        // there's only one vertex in there!  (The latter case may happen
        // for the stroke case with no close figure if we just flushed a
        // batch.)
        // If we're flattening, we must call the one additional time to
        // correctly handle closing the subpath, even if there is only
        // one entry in the batch. The flattening callback handles the
        // one point case and closes the subpath properly without adding
        // extraneous points.

        let verticesInBatch = ENUMERATE_BUFFER_NUMBER - bufferSize;
        if (verticesInBatch > 1)
        {
            IFC!(InitializeEdges(
                enumerateContext,
                bufferStart,
                static_cast<UINT>(verticesInBatch)
                ));
        }
    }

    return hr;
}

/**************************************************************************\
*
* Function Description:
*
*   We want to sort in the inactive list; the primary key is 'y', and
*   the secondary key is 'x'.  This routine creates a single LONGLONG
*   key that represents both.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn YX(
    x: INT,
    y: INT,
    p: &mut LONGLONG
    )
{
    // Bias 'x' by INT_MAX so that it's effectively unsigned:

    reinterpret_cast<LARGE_INTEGER*>(p)->HighPart = y;
    reinterpret_cast<LARGE_INTEGER*>(p)->LowPart = x + INT_MAX;
}

/**************************************************************************\
*
* Function Description:
*
*   Recursive function to quick-sort our inactive edge list.  Note that
*   for performance, the results are not completely sorted; an insertion
*   sort has to be run after the quicksort in order to do a lighter-weight
*   sort of the subtables.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

const QUICKSORT_THRESHOLD: u32 = 8;

fn
QuickSortEdges(
    /*__inout_xcount(f - l + 1 elements)*/f: *mut CInactiveEdge,
    /*__inout_xcount(array starts at f)*/ l: *mut CInactiveEdge
    )
{
    let e: *mut CEdge;
    let y: LONGLONG;
    let first: LONGLONG;
    let second: LONGLONG;
    let last: LONGLONG;

    // Find the median of the first, middle, and last elements:

    CInactiveEdge *m = f + ((l - f) >> 1);

    SWAP!(y, (f + 1).Yx, m.Yx);
    SWAP!(e, (f + 1).Edge, m.Edge);

    if ((second = (f + 1).Yx) > (last = l.Yx))
    {
        (f + 1).Yx = last;
        l.Yx = second;

        SWAP(e, (f + 1).Edge, l.Edge);
    }
    if ((first = f.Yx) > (last = l.Yx))
    {
        f.Yx = last;
        l.Yx = first;

        SWAP(e, f.Edge, l.Edge);
    }
    if ((second = (f + 1).Yx) > (first = f.Yx))
    {
        (f + 1).Yx = first;
        f.Yx = second;

        SWAP(e, (f + 1).Edge, f.Edge);
    }

    // f->Yx is now the desired median, and (f + 1)->Yx <= f->Yx <= l->Yx

    assert!(((f + 1)->Yx <= f->Yx) && (f->Yx <= l->Yx));

    let median = f.Yx;

    let i: *mut CInactiveEdge = f + 2;
    while (i.Yx < median) {
        i += 1;
    }

    CInactiveEdge *j = l - 1;
    while (j.Yx > median) {
        j -= 1;
    }

    while (i < j)
    {
        SWAP(y, i.Yx, j.Yx);
        SWAP(e, i.Edge, j.Edge);

        while {
            i += 1;
            i.Yx < median
        } {}

        while {
            j -= 1;
            j.Yx > median
        } {}
    }

    SWAP(y, f.Yx, j.Yx);
    SWAP(e, f.Edge, j.Edge);

    let a = j - f;
    let b = l - j;

    // Use less stack space by recursing on the shorter subtable.  Also,
    // have the less-overhead insertion-sort handle small subtables.

    if (a <= b)
    {
        if (a > QUICKSORT_THRESHOLD)
        {
            // 'a' is the smallest, so do it first:

            QuickSortEdges(f, j - 1);
            QuickSortEdges(j + 1, l);
        }
        else if (b > QUICKSORT_THRESHOLD)
        {
            QuickSortEdges(j + 1, l);
        }
    }
    else
    {
        if (b > QUICKSORT_THRESHOLD)
        {
            // 'b' is the smallest, so do it first:

            QuickSortEdges(j + 1, l);
            QuickSortEdges(f, j - 1);
        }
        else if (a > QUICKSORT_THRESHOLD)
        {
            QuickSortEdges(f, j- 1);
        }
    }
}

/**************************************************************************\
*
* Function Description:
*
*   Do a sort of the inactive table using an insertion-sort.  Expects
*   large tables to have already been sorted via quick-sort.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn
InsertionSortEdges(
    /* __inout_xcount(count forward & -1 back)*/ inactive: *mut CInactiveEdge,
    count: INT
    )
{
    let p: *mut CInactiveEdge;
    let e: *mut CEdge;
    let y: LONGLONG;
    let yPrevious: LONGLONG;

    assert!((inactive - 1).Yx == _I64_MIN);
    assert!(count >= 2);

    inactive += 1;     // Skip first entry (by definition it's already in order!)
    count -= 1;

    while {
        p = inactive;

        // Copy the current stuff to temporary variables to make a hole:

        e = inactive.Edge;
        y = inactive.Yx;

        // Shift everything one slot to the right (effectively moving
        // the hole one position to the left):

        while (y < (yPrevious = (p - 1).Yx))
        {
            p.Yx = yPrevious;
            p.Edge = (p - 1).Edge;
            p -= 1;
        }

        // Drop the temporary stuff into the final hole:

        p.Yx = y;
        p.Edge = e;

        // The quicksort should have ensured that we don't have to move
        // any entry terribly far:

        assert!(inactive - p <= QUICKSORT_THRESHOLD);

        inactive += 1;
        count -= 0;
        counnt != 0
    } {}
}


/**************************************************************************\
*
* Function Description:
*
*   Assert the state of the inactive array.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

fn
AssertInactiveArray(
    /*__in_ecount(count)*/ inactive: *const CInactiveEdge,   // Annotation should allow the -1 element
    count: INT
    )
{
    // Verify the head:

/*#if !ANALYSIS*/
    // #if needed because prefast don't know that the -1 element is avaliable
    assert!((inactive - 1).Yx == _I64_MIN);
/*#endif*/
    assert!(inactive->Yx != _I64_MIN);

    while {
        let yx: LONGLONG;
        YX(inactive.Edge.X, inactive.Edge.StartY, &yx);

        assert!(inactive.Yx == yx);
    /*#if !ANALYSIS*/
        // #if needed because tools don't know that the -1 element is avaliable
        assert!(inactive.Yx >= (inactive - 1).Yx);
    /*#endif*/
        inactive += 1;
        count -= 1;
        count != 0
    } {}

    // Verify that the tail is setup appropriately:

    assert!(inactive.Edge.StartY == INT_MAX);
}


/**************************************************************************\
*
* Function Description:
*
*   Initialize and sort the inactive array.
*
* Returns:
*
*   'y' value of topmost edge.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

INT
InitializeInactiveArray(
    __in_ecount(1) CEdgeStore *pEdgeStore,
    __in_ecount(count+2) CInactiveEdge *rgInactiveArray,
    UINT count,
    __in_ecount(1) CEdge *tailEdge                    // Tail sentinel for inactive list
    )
{
    BOOL isMore;
    CEdge *pActiveEdge;
    CEdge *pActiveEdgeEnd;

    // First initialize the inactive array.  Skip the first entry,
    // which we reserve as a head sentinel for the insertion sort:

    CInactiveEdge *pInactiveEdge = rgInactiveArray + 1;

    do {
        isMore = pEdgeStore->Enumerate(&pActiveEdge, &pActiveEdgeEnd);

        while (pActiveEdge != pActiveEdgeEnd)
        {
            pInactiveEdge->Edge = pActiveEdge;
            YX(pActiveEdge->X, pActiveEdge->StartY, &pInactiveEdge->Yx);
            pInactiveEdge++;
            pActiveEdge++;
        }
    } while (isMore);

    Assert(static_cast<UINT>(pInactiveEdge - rgInactiveArray) == count + 1);

    // Add the tail, which is used when reading back the array.  This
    // is why we had to allocate the array as 'count + 1':

    pInactiveEdge->Edge = tailEdge;

    // Add the head, which is used for the insertion sort.  This is why
    // we had to allocate the array as 'count + 2':

    rgInactiveArray->Yx = _I64_MIN;

    // Only invoke the quicksort routine if it's worth the overhead:

    if (count > QUICKSORT_THRESHOLD)
    {
        // Quick-sort this, skipping the first and last elements,
        // which are sentinels.
        //
        // We do 'inactiveArray + count' to be inclusive of the last
        // element:

        QuickSortEdges(rgInactiveArray + 1, rgInactiveArray + count);
    }

    // Do a quick sort to handle the mostly sorted result:

    InsertionSortEdges(rgInactiveArray + 1, count);

    ASSERTINACTIVEARRAY(rgInactiveArray + 1, count);

    // Return the 'y' value of the topmost edge:

    return(rgInactiveArray[1].Edge->StartY);
}

/**************************************************************************\
*
* Function Description:
*
*   Insert edges into the active edge list.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

VOID
InsertNewEdges(
    __inout_ecount(1) CEdge *pActiveList,
    INT iCurrentY,
    __deref_inout_xcount(array terminated by an edge with StartY != iCurrentY)
        CInactiveEdge **ppInactiveEdge,
    __out_ecount(1) INT *pYNextInactive
        // will be INT_MAX when no more
    )
{
    CInactiveEdge *inactive = *ppInactiveEdge;

    Assert(inactive->Edge->StartY == iCurrentY);

    do {
        CEdge *newActive = inactive->Edge;

        // The activeList edge list sentinel has X = INT_MAX, so this always
        // terminates:

        while (pActiveList->Next->X < newActive->X)
            pActiveList = pActiveList->Next;

#if SORT_EDGES_INCLUDING_SLOPE
        // The activeList edge list sentinel has Dx = INT_MAX, so this always
        // terminates:

        while ((pActiveList->Next->X == newActive->X) &&
               (pActiveList->Next->Dx < newActive->Dx))
        {
            pActiveList = pActiveList->Next;
        }
#endif

        newActive->Next = pActiveList->Next;
        pActiveList->Next = newActive;

        inactive++;

    } while (inactive->Edge->StartY == iCurrentY);

    *pYNextInactive = inactive->Edge->StartY;
    *ppInactiveEdge = inactive;
}

/**************************************************************************\
*
* Function Description:
*
*   Sort the edges so that they're in ascending 'x' order.
*
*   We use a bubble-sort for this stage, because edges maintain good
*   locality and don't often switch ordering positions.
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/

VOID
FASTCALL
SortActiveEdges(
    __inout_ecount(1) CEdge *list
    )
{
    BOOL swapOccurred;
    CEdge *tmp;

    // We should never be called with an empty active edge list:

    Assert(list->Next->X != INT_MAX);

    do {
        swapOccurred = FALSE;

        CEdge *previous = list;
        CEdge *current = list->Next;
        CEdge *next = current->Next;
        INT nextX = next->X;

        do {
            if (nextX < current->X)
            {
                swapOccurred = TRUE;

                previous->Next = next;
                current->Next = next->Next;
                next->Next = current;

                SWAP(tmp, next, current);
            }

            previous = current;
            current = next;
            next = next->Next;

        } while ((nextX = next->X) != INT_MAX);

    } while (swapOccurred);
}

/**************************************************************************\
*
* Function Description:
*
* For each scan-line to be filled:
*
*   1.  Remove any stale edges from the active edge list
*   2.  Insert into the active edge list any edges new to this scan-line
*   3.  Advance the DDAs of every active edge
*   4.  If any active edges are out of order, re-sort the active edge list
*   5.  Now that the active edges are ready for this scan, call the filler
*       to traverse the edges and output the spans appropriately
*   6.  Lather, rinse, and repeat
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/
#if 0
void
CAliasedFiller::RasterizeEdges(
    __inout_ecount(1) CEdge *activeList,
    __inout_xcount(array terminated by an edge with StartY != iCurrentY)
        CInactiveEdge *inactiveArray,
    INT iCurrentY,
    INT yBottom,
    MilFillMode::Enum fillMode
    )
{
    INT yNextInactive;

    InsertNewEdges(activeList, iCurrentY, &inactiveArray, &yNextInactive);

    ASSERTACTIVELIST(activeList, iCurrentY);

    FillEdges(fillMode, activeList, iCurrentY);

    while (++iCurrentY < yBottom)
    {
        AdvanceDDAAndUpdateActiveEdgeList(iCurrentY, activeList);

        if (iCurrentY == yNextInactive)
        {
            InsertNewEdges(activeList, iCurrentY, &inactiveArray, &yNextInactive);
        }

        ASSERTACTIVELIST(activeList, iCurrentY);

        // Do the appropriate alternate or winding, supersampled or
        // non-supersampled fill:

        FillEdges(fillMode, activeList, iCurrentY);
    }
}


/**************************************************************************\
*
* Function Description:
*  
*   Generate complemented output for the case where there are no input
*   edges.
*
* Created:
*
*   04/30/2006 Danwo
*
\**************************************************************************/
HRESULT
CAntialiasedFiller::RasterizeNoEdges()
{
    Assert(CreateComplementGeometry());
    
    for (int y = m_rcComplementBounds.top; y < m_rcComplementBounds.bottom; ++y)
    {
        GenerateOutput(y << c_nShift);
    }

    return S_OK;
}


/**************************************************************************\
*
* Function Description:
*
* For each scan-line to be filled:
*
*   1.  Remove any stale edges from the active edge list
*   2.  Insert into the active edge list any edges new to this scan-line
*   3.  Advance the DDAs of every active edge
*   4.  If any active edges are out of order, re-sort the active edge list
*   5.  Now that the active edges are ready for this scan, call the filler
*       to traverse the edges and output the spans appropriately
*   6.  Lather, rinse, and repeat
*
* Created:
*
*   03/25/2000 andrewgo
*
\**************************************************************************/
HRESULT
CAntialiasedFiller::RasterizeEdges(
    __inout_ecount(1) CEdge *pEdgeActiveList,
    __inout_xcount(array terminated by an edge with StartY >= nSubpixelYBottom)
        CInactiveEdge *pInactiveEdgeArray,
    INT nSubpixelYCurrent,
    INT nSubpixelYBottom,
    MilFillMode::Enum fillMode
    )
{
    // Disable instrumentation checks for this function
    SET_MILINSTRUMENTATION_FLAGS(MILINSTRUMENTATIONFLAGS_DONOTHING);

    HRESULT hr = S_OK;
    CEdge *pEdgePrevious;
    CEdge *pEdgeCurrent;
    INT nSubpixelYNextInactive;
    INT nSubpixelYNext;

    InsertNewEdges(pEdgeActiveList, nSubpixelYCurrent, &pInactiveEdgeArray, &nSubpixelYNextInactive);

    if (CreateComplementGeometry())
    {
        // Generate spans for rows in complement above start of shape.
        int yFirst = nSubpixelYCurrent >> c_nShift;
        for (int y = m_rcComplementBounds.top; y < yFirst; ++y)
        {
            GenerateOutput(y << c_nShift);
        }
    }

    while (nSubpixelYCurrent < nSubpixelYBottom)
    {
        ASSERTACTIVELIST(pEdgeActiveList, nSubpixelYCurrent);

        //
        // Detect two vertical edges for fast path rasterization
        //

        pEdgePrevious = pEdgeActiveList;
        pEdgeCurrent = pEdgeActiveList->Next;

        // It is important that we check pEdgeCurrent->EndY != INT_MIN before using pEdgeCurrent->Next,
        // so, the ordering of this check has been carefully selected.

        if ((nSubpixelYCurrent & c_nShiftMask) == 0                         // scanline aligned
            && nSubpixelYNextInactive >= nSubpixelYCurrent + c_nShiftSize   // next inactive after next scanline
            && pEdgeCurrent->EndY >= nSubpixelYCurrent + c_nShiftSize       // endy after next scanline
            && pEdgeCurrent->Dx == 0                                        // left edge is vertical
            && pEdgeCurrent->ErrorUp == 0
            && pEdgeCurrent->Next->EndY >= nSubpixelYCurrent + c_nShiftSize // endy after next scanline
            && pEdgeCurrent->Next->Dx == 0                                  // right edge is vertical
            && pEdgeCurrent->Next->ErrorUp == 0
            && pEdgeCurrent->Next->Next->EndY == INT_MIN                    // only two edges
            )
        {
            // Edges are paired, so we can assert we have another one
            Assert(pEdgeCurrent->Next->EndY != INT_MIN);

            // Compute end of our vertical fill area
            nSubpixelYNext = min(pEdgeCurrent->EndY, min(pEdgeCurrent->Next->EndY, nSubpixelYNextInactive));

            // Clip to nSubpixelYBottom
            nSubpixelYNext = min(nSubpixelYNext, nSubpixelYBottom);

            // Snap to scanline boundary
            nSubpixelYNext &= ~c_nShiftMask;

            // Compute coverage and display
            if (pEdgeCurrent->X == pEdgeCurrent->Next->X)
            {
                // It's empty, so just advance nSubpixelYCurrent;
                nSubpixelYCurrent = nSubpixelYNext;
            }
            else
            {
                // Compute the coverage
                for (int i = 0; i < c_nShiftSize; i++)
                {
                    IFC(m_coverageBuffer.AddInterval(pEdgeCurrent->X, pEdgeCurrent->Next->X));
                }

                // Output the scans
                while (nSubpixelYCurrent < nSubpixelYNext)
                {
                    GenerateOutput(nSubpixelYCurrent);
                    nSubpixelYCurrent += c_nShiftSize;
                }
                m_coverageBuffer.Reset();
            }

            Assert(nSubpixelYCurrent == nSubpixelYNext);

            // Remove stale edges.
            while (pEdgeCurrent->EndY != INT_MIN)
            {
                if (pEdgeCurrent->EndY <= nSubpixelYCurrent)
                {
                    // Unlink and advance

                    pEdgeCurrent = pEdgeCurrent->Next;
                    pEdgePrevious->Next = pEdgeCurrent;
                }
                else
                {
                    // Advance

                    pEdgePrevious = pEdgeCurrent;
                    pEdgeCurrent = pEdgeCurrent->Next;
                }
            }
        }
        else
        {
            //
            // Not two vertical edges, so fall back to the general case.
            //

            IFC(FillEdges(fillMode, pEdgeActiveList, nSubpixelYCurrent));

            // Advance nSubpixelYCurrent
            nSubpixelYCurrent += 1;

            // Advance DDA and update edge list
            AdvanceDDAAndUpdateActiveEdgeList(nSubpixelYCurrent, pEdgeActiveList);
        }

        //
        // Update edge list
        //

        if (nSubpixelYCurrent == nSubpixelYNextInactive)
        {
            InsertNewEdges(
                pEdgeActiveList,
                nSubpixelYCurrent,
                &pInactiveEdgeArray,
                &nSubpixelYNextInactive
                );
        }
    }

    //
    // Output the last scanline that has partial coverage
    //

    if ((nSubpixelYCurrent & c_nShiftMask) != 0)
    {
        GenerateOutput(nSubpixelYCurrent);
    }

    if (CreateComplementGeometry())
    {
        // Generate spans for scanlines in complement below start of shape.
        m_coverageBuffer.Reset();
        // +c_nShiftMask makes sure we advance to next full Y not generated.
        int y = (nSubpixelYCurrent + c_nShiftMask) >> c_nShift;
        while (y < m_rcComplementBounds.bottom)
        {
            GenerateOutput(y << c_nShift);
            ++y;
        }
    }

Cleanup:
    RRETURN(hr);
}
//+-----------------------------------------------------------------------------
//
//  Function:  RasterizePath
//
//  Synopsis:  Fill (or sometimes stroke) that path.
//

HRESULT
RasterizePath(
    __in_ecount(cPoints)   const MilPoint2F *rgPoints,      // Points of the path to stroke/fill
    __in_ecount(cPoints)   const BYTE *rgTypes,            // Types array of the path
    __in_range(>=,2) const UINT cPoints,             // Number of points in the path
    __in_ecount(1) const CBaseMatrix *pMatPointsToDevice,
    MilFillMode::Enum fillMode,
    MilAntiAliasMode::Enum antiAliasMode,
    __inout_ecount(1) CSpanSink *pSpanSink,                // The sink for the spans produced by the
                                                           // rasterizer. For AA, this sink must
                                                           // include an operation to apply the AA
                                                           // coverage.
    __in_ecount(1) CSpanClipper *pClipper,                 // Clipper.
    __in_ecount(1) const MilPointAndSizeL *prcBounds,               // Bounding rectangle of the path points.
    float rComplementFactor,
    __in_ecount_opt(1) const CMILSurfaceRect *prcComplementBounds
    )
{
    HRESULT hr = S_OK;
    CInactiveEdge inactiveArrayStack[INACTIVE_LIST_NUMBER];
    CInactiveEdge *inactiveArray;
    CInactiveEdge *inactiveArrayAllocation = NULL;
    CEdge headEdge;
    CEdge tailEdge;
    CEdge *activeList;
    CEdgeStore edgeStore;
    CInitializeEdgesContext edgeContext;

    Assert(rComplementFactor < 0 || antiAliasMode == MilAntiAliasMode::EightByEight);
    Assert(rComplementFactor < 0 || prcComplementBounds);
    
    Assert(pMatPointsToDevice);

    edgeContext.ClipRect = NULL;

    tailEdge.X = INT_MAX;       // Terminator to active list
#if SORT_EDGES_INCLUDING_SLOPE
    tailEdge.Dx = INT_MAX;      // Terminator to active list
#endif
    tailEdge.StartY = INT_MAX;  // Terminator to inactive list

    tailEdge.EndY = INT_MIN;
    headEdge.X = INT_MIN;       // Beginning of active list
    edgeContext.MaxY = INT_MIN;

    headEdge.Next = &tailEdge;
    activeList = &headEdge;
    edgeContext.Store = &edgeStore;

    edgeContext.AntiAliasMode = antiAliasMode;

    //////////////////////////////////////////////////////////////////////////

    CMILSurfaceRect rc;
    pClipper->GetClipBounds(&rc);
    pClipper->SetOutputSpan(pSpanSink);

    MilPointAndSizeL rcTemp;
    MilPointAndSizeL rcMilPointAndSizeL = {rc.left, rc.top, rc.Width(), rc.Height()};

    INT yClipBottom = rc.bottom;
    UINT totalCount = 0;
    
    // check to see if we're fully clipped.
    // If the path contains 0 or 1 points, we can ignore it.
    if ((cPoints > 1) && IntersectRect(&rcTemp, &rcMilPointAndSizeL, prcBounds))
    {
        //   Need input path validation
        //  This check is a band-aid. Generally speaking, RasterizePath assumes (and asserts) that
        //  the input path is valid (and so this check should be an assertion).
        //
        //  The advantage of this is that other internal code which generates paths (e.g. widening)
        //  can use RasterizePath without needing full consistency checking.
        //
        //  But what we are missing, is path-validation code at the MILRender entry point level.


        // Scale the clip bounds rectangle by 16 to account for our
        // scaling to 28.4 coordinates:

        RECT clipBounds;
        clipBounds.left = rc.left * 16;
        clipBounds.top = rc.top * 16;
        clipBounds.right = rc.right * 16;
        clipBounds.bottom = rc.bottom * 16;

        edgeContext.ClipRect = &clipBounds;

        // The clipper should call the sink's OutputSpan

        {
            //////////////////////////////////////////////////////////////////////////

            // Convert all our points to 28.4 fixed point:

            CMILMatrix matrix(*pMatPointsToDevice);

            // Given matrix transforms points to device space in half-pixel-center
            // notation. We need integer-pixel-center notation here, so we
            // adjust the matrix to shift all the coordinates by 1/2 of pixel.
            matrix.SetDx(matrix.GetDx() - 0.5f);
            matrix.SetDy(matrix.GetDy() - 0.5f);

            AppendScaleToMatrix(&matrix, TOREAL(16), TOREAL(16));

            // Enumerate the path and construct the edge table:

            MIL_THR(FixedPointPathEnumerate(
                        rgPoints,
                        rgTypes,
                        cPoints,
                        &matrix,
                        edgeContext.ClipRect,
                        &edgeContext
                        ));

            if (FAILED(hr))
            {
                if (hr == WGXERR_VALUEOVERFLOW)
                {
                    // Draw nothing on value overflow and return
                    hr = S_OK;
                }
                goto Cleanup;
            }
        }
        totalCount = edgeStore.StartEnumeration();        
    }

    if (totalCount == 0)
    {
        // Path empty or totally clipped.  We're almost done.
        // May need to take care of complement geometry.
        if (rComplementFactor >= 0)
        {
            // Complement factor only support in AA rendering.
            Assert(antiAliasMode != MilAntiAliasMode::None);
            
            CAntialiasedFiller filler(pClipper, antiAliasMode);
            filler.SetComplementFactor(
                rComplementFactor,
                prcComplementBounds
                );

            pSpanSink->SetAntialiasedFiller(&filler);

            IFC(filler.RasterizeNoEdges());
        }

        hr = S_OK;
        goto Cleanup;
    }

    // At this point, there has to be at least two edges.  If there's only
    // one, it means that we didn't do the trivial rejection properly.

    Assert(totalCount >= 2);

    inactiveArray = &inactiveArrayStack[0];
    if (totalCount > (INACTIVE_LIST_NUMBER - 2))
    {
        UINT tempCount = 0;
        IFC (UIntAdd(totalCount, 2, &tempCount));
        IFC(HrMalloc(
            Mt(MAARasterizerEdge),
            sizeof(CInactiveEdge),
            tempCount,
            (void **)&inactiveArrayAllocation
            ));

        inactiveArray = inactiveArrayAllocation;
    }

    // Initialize and sort the inactive array:

    INT iCurrentY, yBottom;
    iCurrentY = InitializeInactiveArray(
        &edgeStore,
        inactiveArray,
        totalCount,
        &tailEdge
        );

    yBottom = edgeContext.MaxY;

    Assert(yBottom > 0);

    // Skip the head sentinel on the inactive array:

    inactiveArray++;

    if (antiAliasMode != MilAntiAliasMode::None)
    {
        CAntialiasedFiller filler(pClipper, antiAliasMode);
        if (rComplementFactor >= 0)
        {
            filler.SetComplementFactor(
                rComplementFactor,
                prcComplementBounds
                );
        }

        pSpanSink->SetAntialiasedFiller(&filler);

        // 'yClipBottom' is in 28.4 format, and has to be converted
        // to the 30.2 (or 29.3) format we use for antialiasing:

        yBottom = min(yBottom, yClipBottom << c_nShift);

        // 'totalCount' should have been zero if all the edges were
        // clipped out (RasterizeEdges assumes there's at least one edge
        // to be drawn):

        Assert(yBottom > iCurrentY);

        IFC(filler.RasterizeEdges(
            activeList,
            inactiveArray,
            iCurrentY,
            yBottom,
            fillMode
            ));
    }
    else
    {
        CAliasedFiller filler(pClipper);
        Assert(!(rComplementFactor >= 0));

        yBottom = min(yBottom, yClipBottom);

        // 'totalCount' should have been zero if all the edges were
        // clipped out (RasterizeEdges assumes there's at least one edge
        // to be drawn):

        Assert(yBottom > iCurrentY);

        filler.RasterizeEdges(
            activeList,
            inactiveArray,
            iCurrentY,
            yBottom,
            fillMode
            );
    }

Cleanup:
    // Free any objects and get outta here:

    if (inactiveArrayAllocation != NULL)
    {
        GpFree(inactiveArrayAllocation);
    }

    RRETURN(hr);
}

//+-----------------------------------------------------------------------------
//
//  Function:  IsPPAAMode
//
//  Synopsis:  Returns TRUE if the given MilAntiAliasMode::Enum is a per-primitive
//             antialiasing (PPAA) mode.
//

BOOL IsPPAAMode(
    MilAntiAliasMode::Enum aam
    )
{
    switch (aam)
    {
    case MilAntiAliasMode::None:
        return FALSE;

    case MilAntiAliasMode::EightByEight:
        return TRUE;

    default:
        AssertMsg(FALSE, "Unrecognized antialias mode");
        return FALSE;
    }
}



#endif




// Licensed to the .NET Foundation under one or more agreements.
// The .NET Foundation licenses this file to you under the MIT license.
// See the LICENSE file in the project root for more information.


//------------------------------------------------------------------------------
//

//
//  Description:
//      Coverage buffer implementation
//
use crate::aarasterizer::{AssertActiveList, CEdge};
use crate::{types::*, IFC, ASSERTACTIVELIST, IFCOOM};
//struct CEdge;
//struct CInactiveEdge;

//-------------------------------------------------------------------------
//
// TrapezoidalAA only supports 8x8 mode, so the shifts/masks are all
// constants.  Also, since we must be symmetrical, x and y shifts are
// merged into one shift unlike the implementation in aarasterizer.
//
//-------------------------------------------------------------------------

pub const c_nShift: INT = 3; 
pub const c_nShiftSize: INT = 8; 
pub const c_nShiftSizeSquared: INT = c_nShiftSize * c_nShiftSize; 
pub const c_nHalfShiftSize: INT = 4; 
pub const c_nShiftMask: INT = 7; 
pub const c_rShiftSize: f32 = 8.0;
pub const c_rHalfShiftSize: f32 = 4.0;
pub const c_rInvShiftSize: f32 = 1.0/8.0;
pub const c_antiAliasMode: MilAntiAliasMode = MilAntiAliasMode::EightByEight;

//
// Interval coverage descriptor for our antialiased filler
//

pub struct CCoverageInterval
{
    m_pNext: *mut CCoverageInterval, // m_pNext interval (look for sentinel, not NULL)
    m_nPixelX: INT,              // Interval's left edge (m_pNext->X is the right edge)
    m_nCoverage: INT,            // Pixel coverage for interval
}

impl Default for CCoverageInterval {
    fn default() -> Self {
        Self { m_pNext: NULL(), m_nPixelX: Default::default(), m_nCoverage: Default::default() }
    }
}

// Define our on-stack storage use.  The 'free' versions are nicely tuned
// to avoid allocations in most common scenarios, while at the same time
// not chewing up toooo much stack space.  
//
// We make the debug versions small so that we hit the 'grow' cases more
// frequently, for better testing:

#[cfg(debug)]
    // Must be at least 6 now: 4 for the "minus4" logic in hwrasterizer.*, and then 
    // 1 each for the head and tail sentinels (since their allocation doesn't use Grow).
    const INTERVAL_BUFFER_NUMBER: usize = 8;        
#[cfg(not(debug))]   
    const INTERVAL_BUFFER_NUMBER: usize = 32;


//
// Allocator structure for the antialiased fill interval data
//

struct CCoverageIntervalBuffer
{
    m_pNext: *mut CCoverageIntervalBuffer,
    m_interval: [CCoverageInterval; INTERVAL_BUFFER_NUMBER],
}

impl Default for CCoverageIntervalBuffer {
    fn default() -> Self {
        Self { m_pNext: NULL(), m_interval: Default::default() }
    }
}

//------------------------------------------------------------------------------
//
//  Class: CCoverageBuffer
//
//  Description:
//      Coverage buffer implementation that maintains coverage information
//      for one scanline.  
//
//      This implementation will maintain a linked list of intervals consisting
//      of x value in pixel space and a coverage value that applies for all pixels
//      between pInterval->X and pInterval->Next->X.
//
//      For example, if we add the following interval (assuming 8x8 anti-aliasing)
//      to the coverage buffer:
//       _____ _____ _____ _____
//      |     |     |     |     |
//      |  -------------------  |
//      |_____|_____|_____|_____|
//    (0,0) (1,0) (2,0) (3,0) (4,0)
//
//      Then we will get the following coverage buffer:
//
//     m_nPixelX: INT_MIN  |  0  |  1  |  3  |  4  | INT_MAX
//   m_nCoverage: 0        |  4  |  8  |  4  |  0  | 0xdeadbeef
//       m_pNext: -------->|---->|---->|---->|---->| NULL
//              
//------------------------------------------------------------------------------
pub struct CCoverageBuffer
{
    /*
public:
    //
    // Init/Destroy methods
    //

    VOID Initialize();
    VOID Destroy();

    //
    // Setup the buffer so that it can accept another scanline
    //

    VOID Reset();

    //
    // Add a subpixel interval to the coverage buffer
    //

    HRESULT FillEdgesAlternating(
        __in_ecount(1) const CEdge *pEdgeActiveList,
        INT nSubpixelYCurrent
        );

    HRESULT FillEdgesWinding(
        __in_ecount(1) const CEdge *pEdgeActiveList,
        INT nSubpixelYCurrent
        );

    HRESULT AddInterval(INT nSubpixelXLeft, INT nSubpixelXRight);

private:

    HRESULT Grow(
        __deref_out_ecount(1) CCoverageInterval **ppIntervalNew, 
        __deref_out_ecount(1) CCoverageInterval **ppIntervalEndMinus4
        );

public:*/
    pub m_pIntervalStart: *mut CCoverageInterval,           // Points to list head entry

//private:
    m_pIntervalNew: *mut CCoverageInterval,

    // The Minus4 in the below variable refers to the position at which
    // we need to Grow the buffer.  The buffer is grown once before an
    // AddInterval, so the Grow has to ensure that there are enough 
    // intervals for the AddInterval worst case which is the following:
    //
    //  1     2           3     4
    //  *_____*_____ _____*_____* 
    //  |     |     |     |     |
    //  |  ---|-----------|---  |
    //  |_____|_____|_____|_____|
    //
    // Note that the *'s above mark potentional insert points in the list,
    // so we need to ensure that at least 4 intervals can be allocated.
    //

    m_pIntervalEndMinus4:  *mut CCoverageInterval,

    m_pIntervalBufferBuiltin: CCoverageIntervalBuffer,
    m_pIntervalBufferCurrent: *mut CCoverageIntervalBuffer,
       
    // Disable instrumentation checks within all methods of this class
    //SET_MILINSTRUMENTATION_FLAGS(MILINSTRUMENTATIONFLAGS_DONOTHING);
}

impl Default for CCoverageBuffer {
    fn default() -> Self {
        Self { m_pIntervalStart: NULL(), m_pIntervalNew: NULL(), m_pIntervalEndMinus4: NULL(), m_pIntervalBufferBuiltin: Default::default(), m_pIntervalBufferCurrent: NULL() }
    }
}


//
// Inlines
//
impl CCoverageBuffer {
//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::AddInterval
//
//  Synopsis:   Add a subpixel resolution interval to the coverage buffer
// 
//-------------------------------------------------------------------------
pub fn AddInterval(&mut self, nSubpixelXLeft: INT, nSubpixelXRight: INT) -> HRESULT
{
    unsafe {
    let hr: HRESULT = S_OK;
    let mut nPixelXNext: INT;
    let nPixelXLeft: INT;
    let nPixelXRight: INT;
    let nCoverageLeft: INT;  // coverage from right edge of pixel for interval start
    let nCoverageRight: INT; // coverage from left edge of pixel for interval end

    let mut pInterval = self.m_pIntervalStart;
    let mut pIntervalNew = self.m_pIntervalNew;
    let mut pIntervalEndMinus4 = self.m_pIntervalEndMinus4;

    // Make sure we have enough room to add two intervals if
    // necessary:

    if (pIntervalNew >= pIntervalEndMinus4)
    {
        IFC!(self.Grow(&mut pIntervalNew, &mut pIntervalEndMinus4));
    }

    // Convert interval to pixel space so that we can insert it 
    // into the coverage buffer

    assert!(nSubpixelXLeft < nSubpixelXRight);
    nPixelXLeft = nSubpixelXLeft >> c_nShift;
    nPixelXRight = nSubpixelXRight >> c_nShift; 

    // Skip any intervals less than 'nPixelLeft':

    loop {
        nPixelXNext = (*(*pInterval).m_pNext).m_nPixelX;
        if !(nPixelXNext < nPixelXLeft) { break }

        pInterval = (*pInterval).m_pNext;
    }

    // Insert a new interval if necessary:

    if (nPixelXNext != nPixelXLeft)
    {
        (*pIntervalNew).m_nPixelX = nPixelXLeft;
        (*pIntervalNew).m_nCoverage = (*pInterval).m_nCoverage;

        (*pIntervalNew).m_pNext = (*pInterval).m_pNext;
        (*pInterval).m_pNext = pIntervalNew;

        pInterval = pIntervalNew;

        pIntervalNew = pIntervalNew.offset(1);
    }
    else
    {
        pInterval = (*pInterval).m_pNext;
    }

    //
    // Compute coverage for left segment as shown by the *'s below
    //
    //  |_____|_____|_____|_
    //  |     |     |     |
    //  |  ***----------  |
    //  |_____|_____|_____|
    //

    nCoverageLeft = c_nShiftSize - (nSubpixelXLeft & c_nShiftMask);

    // If nCoverageLeft == 0, then the value of nPixelXLeft is wrong
    // and should have been equal to nPixelXLeft+1.
    assert!(nCoverageLeft > 0);

    // If we have partial coverage, then ensure that we have a position
    // for the end of the pixel 

    if ((nCoverageLeft < c_nShiftSize || (nPixelXLeft == nPixelXRight))
        && nPixelXLeft + 1 != (*(*pInterval).m_pNext).m_nPixelX)
    {
        (*pIntervalNew).m_nPixelX = nPixelXLeft + 1;
        (*pIntervalNew).m_nCoverage = (*pInterval).m_nCoverage;

        (*pIntervalNew).m_pNext = (*pInterval).m_pNext;
        (*pInterval).m_pNext = pIntervalNew;

        pIntervalNew = pIntervalNew.offset(1);
    }
    
    //
    // If the interval only includes one pixel, then the coverage is
    // nSubpixelXRight - nSubpixelXLeft
    //

    if (nPixelXLeft == nPixelXRight)
    {
        (*pInterval).m_nCoverage += nSubpixelXRight - nSubpixelXLeft;
        assert!((*pInterval).m_nCoverage <= c_nShiftSize*c_nShiftSize);
        //goto Cleanup;

        //Cleanup:
        // Update the coverage buffer new interval
        self.m_pIntervalNew = pIntervalNew;
    }

    // Update coverage of current interval
    (*pInterval).m_nCoverage += nCoverageLeft;
    assert!((*pInterval).m_nCoverage <= c_nShiftSize*c_nShiftSize);

    // Increase the coverage for any intervals between 'nPixelXLeft'
    // and 'nPixelXRight':

    loop {
        (nPixelXNext = (*(*pInterval).m_pNext).m_nPixelX);
    
        if !(nPixelXNext < nPixelXRight) {
            break;
        }
        pInterval = (*pInterval).m_pNext;
        (*pInterval).m_nCoverage += c_nShiftSize;
        assert!((*pInterval).m_nCoverage <= c_nShiftSize*c_nShiftSize);
    }

    // Insert another new interval if necessary:

    if (nPixelXNext != nPixelXRight)
    {
        (*pIntervalNew).m_nPixelX = nPixelXRight;
        (*pIntervalNew).m_nCoverage = (*pInterval).m_nCoverage - c_nShiftSize;

        (*pIntervalNew).m_pNext = (*pInterval).m_pNext;
        (*pInterval).m_pNext = pIntervalNew;

        pInterval = pIntervalNew;

        pIntervalNew = pIntervalNew.offset(1);
    }
    else
    {
        pInterval = (*pInterval).m_pNext;
    }

    //
    // Compute coverage for right segment as shown by the *'s below
    //
    //  |_____|_____|_____|_
    //  |     |     |     |
    //  |  ---------****  |
    //  |_____|_____|_____|
    //

    nCoverageRight = nSubpixelXRight & c_nShiftMask;
    if (nCoverageRight > 0)
    {
        if (nPixelXRight + 1 != (*(*pInterval).m_pNext).m_nPixelX)
        {
            (*pIntervalNew).m_nPixelX = nPixelXRight + 1;
            (*pIntervalNew).m_nCoverage = (*pInterval).m_nCoverage;

            (*pIntervalNew).m_pNext = (*pInterval).m_pNext;
            (*pInterval).m_pNext = pIntervalNew;

            pIntervalNew = pIntervalNew.offset(1);
        }

        (*pInterval).m_nCoverage += nCoverageRight;
        assert!((*pInterval).m_nCoverage <= c_nShiftSize*c_nShiftSize);
    }

//Cleanup:
    // Update the coverage buffer new interval

    self.m_pIntervalNew = pIntervalNew;
    

    return hr;
    }
}


//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::FillEdgesAlternating
//
//  Synopsis:   
//      Given the active edge list for the current scan, do an alternate-mode
//      antialiased fill.
//
//-------------------------------------------------------------------------
pub fn FillEdgesAlternating(&mut self,
    pEdgeActiveList: *const CEdge,
    nSubpixelYCurrent: INT
    ) -> HRESULT
{
unsafe {
    let hr: HRESULT = S_OK;
    let mut pEdgeStart: *const CEdge = (*pEdgeActiveList).Next;
    let mut pEdgeEnd: *const CEdge;
    let mut nSubpixelXLeft: INT;
    let mut nSubpixelXRight: INT;

    ASSERTACTIVELIST!(pEdgeActiveList, nSubpixelYCurrent);

    while ((*pEdgeStart).X != INT::MAX)
    {
        pEdgeEnd = (*pEdgeStart).Next;

        // We skip empty pairs:
        (nSubpixelXLeft = (*pEdgeStart).X);
        if (nSubpixelXLeft != (*pEdgeEnd).X)
        {
            // We now know we have a non-empty interval.  Skip any
            // empty interior pairs:

            while ({(nSubpixelXRight = (*pEdgeEnd).X); (*pEdgeEnd).X == (*(*pEdgeEnd).Next).X})
            {
                pEdgeEnd = (*(*pEdgeEnd).Next).Next;
            }

            assert!((nSubpixelXLeft < nSubpixelXRight) && (nSubpixelXRight < INT::MAX));

            IFC!(self.AddInterval(nSubpixelXLeft, nSubpixelXRight));
        }

        // Prepare for the next iteration:
        pEdgeStart = (*pEdgeEnd).Next;
    } 

//Cleanup:
    return hr
}
}

//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::FillEdgesWinding
//
//  Synopsis:   
//      Given the active edge list for the current scan, do an alternate-mode
//      antialiased fill.
//
//-------------------------------------------------------------------------
pub fn FillEdgesWinding(&mut self,
    pEdgeActiveList: *const CEdge,
    nSubpixelYCurrent: INT
    ) -> HRESULT
{
unsafe {
    let hr: HRESULT = S_OK;
    let mut pEdgeStart: *const CEdge = (*pEdgeActiveList).Next;
    let mut pEdgeEnd: *const CEdge;
    let mut nSubpixelXLeft: INT;
    let mut nSubpixelXRight: INT;
    let mut nWindingValue: INT;

    ASSERTACTIVELIST!(pEdgeActiveList, nSubpixelYCurrent);

    while ((*pEdgeStart).X != INT::MAX)
    {
        pEdgeEnd = (*pEdgeStart).Next;

        nWindingValue = (*pEdgeStart).WindingDirection;
        while ({nWindingValue += (*pEdgeEnd).WindingDirection; nWindingValue != 0})
        {
            pEdgeEnd = (*pEdgeEnd).Next;
        }

        assert!((*pEdgeEnd).X != INT::MAX);

        // We skip empty pairs:

        if ({nSubpixelXLeft = (*pEdgeStart).X; nSubpixelXLeft != (*pEdgeEnd).X})
        {
            // We now know we have a non-empty interval.  Skip any
            // empty interior pairs:

            while ({nSubpixelXRight = (*pEdgeEnd).X; nSubpixelXRight == (*(*pEdgeEnd).Next).X})
            {
                pEdgeStart = (*pEdgeEnd).Next;
                pEdgeEnd = (*pEdgeStart).Next;

                nWindingValue = (*pEdgeStart).WindingDirection;
                while ({nWindingValue += (*pEdgeEnd).WindingDirection; nWindingValue != 0})
                {
                    pEdgeEnd = (*pEdgeEnd).Next;
                }
            }

            assert!((nSubpixelXLeft < nSubpixelXRight) && (nSubpixelXRight < INT::MAX));

            IFC!(self.AddInterval(nSubpixelXLeft, nSubpixelXRight));
        }

        // Prepare for the next iteration:

        pEdgeStart = (*pEdgeEnd).Next;
    } 

//Cleanup:
    return hr;//RRETURN(hr);
}
}

//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::Initialize
//
//  Synopsis:   Set the coverage buffer to a valid initial state
// 
//-------------------------------------------------------------------------
pub fn Initialize(&mut self) 
{
    self.m_pIntervalBufferBuiltin.m_interval[0].m_nPixelX = INT::MIN;
    self.m_pIntervalBufferBuiltin.m_interval[0].m_nCoverage = 0;
    self.m_pIntervalBufferBuiltin.m_interval[0].m_pNext = &mut self.m_pIntervalBufferBuiltin.m_interval[1];

    self.m_pIntervalBufferBuiltin.m_interval[1].m_nPixelX = INT::MAX;
    self.m_pIntervalBufferBuiltin.m_interval[1].m_nCoverage = 0xdeadbeef;
    self.m_pIntervalBufferBuiltin.m_interval[1].m_pNext = NULL();

    self.m_pIntervalBufferBuiltin.m_pNext = NULL();
    self.m_pIntervalBufferCurrent = &mut self.m_pIntervalBufferBuiltin;

    self.m_pIntervalStart = &mut self.m_pIntervalBufferBuiltin.m_interval[0];
    self.m_pIntervalNew = &mut self.m_pIntervalBufferBuiltin.m_interval[2];
    self.m_pIntervalEndMinus4 = &mut self.m_pIntervalBufferBuiltin.m_interval[INTERVAL_BUFFER_NUMBER - 4];
}

//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::Destroy
//
//  Synopsis:   Free all allocated buffers
// 
//-------------------------------------------------------------------------
pub fn Destroy(&mut self)
{
    // Free the linked-list of allocations (skipping 'm_pIntervalBufferBuiltin',
    // which is built into the class):

    let mut pIntervalBuffer = self.m_pIntervalBufferBuiltin.m_pNext;
    while (pIntervalBuffer != NULL())
    {
        let pIntervalBufferNext = unsafe { (*pIntervalBuffer).m_pNext };
        unsafe { drop(Box::from_raw(pIntervalBuffer)) };
        pIntervalBuffer = pIntervalBufferNext;
    }
}

//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::Reset
//
//  Synopsis:   Reset the coverage buffer
// 
//-------------------------------------------------------------------------
pub fn Reset(&mut self)
{
    // Reset our coverage structure.  Point the head back to the tail,
    // and reset where the next new entry will be placed:

    self.m_pIntervalBufferBuiltin.m_interval[0].m_pNext = &mut self.m_pIntervalBufferBuiltin.m_interval[1];

    self.m_pIntervalBufferCurrent = &mut self.m_pIntervalBufferBuiltin;
    self.m_pIntervalNew = &mut self.m_pIntervalBufferBuiltin.m_interval[2];
    self.m_pIntervalEndMinus4 = &mut self.m_pIntervalBufferBuiltin.m_interval[INTERVAL_BUFFER_NUMBER - 4];
}

//-------------------------------------------------------------------------
//
//  Function:   CCoverageBuffer::Grow
//
//  Synopsis:   
//      Grow our interval buffer.
//
//-------------------------------------------------------------------------
fn Grow(&mut self,
    ppIntervalNew: &mut *mut CCoverageInterval, 
    ppIntervalEndMinus4: &mut *mut CCoverageInterval
    ) -> HRESULT
{
    unsafe {
    let hr: HRESULT = S_OK;
    let mut pIntervalBufferNew = (*self.m_pIntervalBufferCurrent).m_pNext;

    if (pIntervalBufferNew != NULL())
    {
        pIntervalBufferNew = Box::into_raw(Box::<CCoverageIntervalBuffer>::new(Default::default()));

        IFCOOM!(pIntervalBufferNew);

        (*pIntervalBufferNew).m_pNext = NULL();
        (*self.m_pIntervalBufferCurrent).m_pNext = pIntervalBufferNew;
    }

    self.m_pIntervalBufferCurrent = pIntervalBufferNew;

    self.m_pIntervalNew = &mut (*pIntervalBufferNew).m_interval[2];
    self.m_pIntervalEndMinus4 = &mut (*pIntervalBufferNew).m_interval[INTERVAL_BUFFER_NUMBER - 4];

    *ppIntervalNew = self.m_pIntervalNew;
    *ppIntervalEndMinus4 = self.m_pIntervalEndMinus4;

    return hr;
    }
}

}

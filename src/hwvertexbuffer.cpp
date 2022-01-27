// Licensed to the .NET Foundation under one or more agreements.
// The .NET Foundation licenses this file to you under the MIT license.
// See the LICENSE file in the project root for more information.


//-----------------------------------------------------------------------------
//

//
//  Description:
//      Contains HW Vertex Buffer and Builder class implementations
//
//
//  Notes:
//        
//        +--------------------------------------+
//        |                                      |
//        |           Start Stratum              |
//     1  |                                      |
//        |                                      |
//        +--------------------------------------+
//     2  |======================================|
//        +--------------------------------------+
//        |          /   \             / \       |
//        |         /     \           /   \      |
//        |   A    /   B   \    C    /  D  \  E  |
//     3  |       /         \       /       \    |
//        |      /           \     /         \   |
//        |     /             \   /           \  |
//        |    /               \ /             \ |
//        +--------------------------------------+
//        |    \               / \             / |
//        |     \             /   \           /  |
//     4  |  F   \     G     /  H  \    I    / J |
//        |       \         /       \       /    |
//        +--------------------------------------+
//     5  |======================================|
//        +--------------------------------------+
//     6  |======================================|
//        +--------------------------------------+
//        |                                      |
//        |                                      |
//     7  |           Stop Stratum               |
//        |                                      |
//        |                                      |
//        +--------------------------------------+
//        
//  
//  Strata & complement mode.
//  
//  The anti-aliased HW rasterizer produces a series of "strata" where
//  each strata can be a complex span rendered using lines (#'s 2,5,6) or
//  a series of trapezoids (#'s 3 & 4.)  In normal mode the trapezoid
//  regions B,D,G,I are filled in.
//  
//  Complement mode complicates things.  Complex spans are relatively easy
//  because we get the whole line's worth of data at once.  Trapezoids are
//  more complex because we get B,D,G and I separately.  We handle this by
//  tracking the current stratum and finishing the last incomplete
//  trapezoid stratum when a new stratum begins.  Regions E & J finish
//  trapezoid strata.  We also need to add rectangles at the beginning and
//  end of the geometry (start and stop) to fill out the complement
//  region.
//  
//  This is implemented like so:
//  
//    1. Strata are generated from top to bottom without gaps.
//    2. Before drawing any lines or trapezoids call
//       PrepareStratum(a, b, fTrapezoid) where a & b are the extent of
//       the current stratum and fTrapezoid is true if you are drawing
//       a trapezoid.  This will take care of creating the start
//       stratum and/or finishing a trapezoid stratum if necessary.
//    3. When completely done call EndBuildingOutside() which will
//       close a pending trapezoid and/or produce the stop stratum.
//  
//-----------------------------------------------------------------------------

#include "precomp.hpp"

MtDefine(CHwTVertexBuffer_Builder, MILRender, "CHwTVertexBuffer<TVertex>::Builder");

ExternTag(tagWireframe);

//+----------------------------------------------------------------------------
//
//  Constants to control when we stop waffling because the tiles are too
//  small to make a difference.
//
//  Future Consideration:  can produce an excessive number of triangles.
//   How we mitigate or handle this could be improved.  Right now we stop
//   waffling if the waffle size is less than a quarter-pixel.
//   Two big improvements that could be made are:
//    - multipacking very small textures (but note that we cannot rely
//      on prefiltering to ensure that small screen space means small texture
//      source)
//    - clipping primitives to approximately the screen size
//
//-----------------------------------------------------------------------------

const float c_rMinWaffleWidthPixels = 0.25f;


const DWORD FLOAT_ZERO = 0x00000000;
const DWORD FLOAT_ONE  = 0x3f800000;

//+----------------------------------------------------------------------------
//
//  Class:     CHwVertexBuffer and CHwTVertexBuffer<class TVertex>
//
//  Synopsis:  This class accumulates geometry data for a primitive
//
//-----------------------------------------------------------------------------

//+----------------------------------------------------------------------------
//
//  Member:    CHwVertexBuffer::AddTriangle
//
//  Synopsis:  Add a triangle using the three indices given to the list
//

MIL_FORCEINLINE
HRESULT
CHwVertexBuffer::AddTriangle(
    WORD i1,         // In: Index of triangle's first vertex
    WORD i2,         // In: Index of triangle's second vertex
    WORD i3          // In: Index of triangle's third vertex
    )
{
    HRESULT hr = S_OK;

    // Asserting indices < max vertex requires a debug only pure virtual method
    // which is too much of a functionality change between retail and debug.
    //
    //
    // Assert(i1 < GetNumTriListVertices());
    // Assert(i2 < GetNumTriListVertices());
    // Assert(i3 < GetNumTriListVertices());

    WORD *pIndices;

    IFC(m_rgIndices.AddMultiple(3, &pIndices));

    pIndices[0] = i1;
    pIndices[1] = i2;
    pIndices[2] = i3;

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::AddTriangle
//
//  Synopsis:  Add a triangle using given three points to the list
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::AddTriangle(
    __in_ecount(1) const PointXYA &v0,
    __in_ecount(1) const PointXYA &v1,
    __in_ecount(1) const PointXYA &v2)
{
    HRESULT hr = S_OK;
    
    TVertex *pVertices;
    hr = AddNonIndexedTriListVertices(3,&pVertices);

    if (hr == E_OUTOFMEMORY)
    {
        DebugBreak ();
    }
    IFC(hr);
    
    pVertices[0].ptPt.X = v0.x;
    pVertices[0].ptPt.Y = v0.y;
    pVertices[0].Diffuse = reinterpret_cast<const DWORD &>(v0.a);
    pVertices[1].ptPt.X = v1.x;
    pVertices[1].ptPt.Y = v1.y;
    pVertices[1].Diffuse = reinterpret_cast<const DWORD &>(v1.a);
    pVertices[2].ptPt.X = v2.x;
    pVertices[2].ptPt.Y = v2.y;
    pVertices[2].Diffuse = reinterpret_cast<const DWORD &>(v2.a);

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::AddLine
//
//  Synopsis:  Add a nominal width line using given two points to the list
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::AddLine(
    __in_ecount(1) const PointXYA &v0,
    __in_ecount(1) const PointXYA &v1
    )
{
    HRESULT hr = S_OK;

    TVertex *pVertices;
    TVertex rgScratchVertices[2];

    Assert(!(v0.y != v1.y));
    
    bool fUseTriangles = (v0.y < m_pBuilder->GetViewportTop() + 1) || FORCE_TRIANGLES;

    if (fUseTriangles)
    {
        pVertices = rgScratchVertices;
    }
    else
    {
        IFC(AddLineListVertices(2, &pVertices));
    }
    
    pVertices[0].ptPt.X = v0.x;
    pVertices[0].ptPt.Y = v0.y;
    pVertices[0].Diffuse = reinterpret_cast<const DWORD &>(v0.a);
    pVertices[1].ptPt.X = v1.x;
    pVertices[1].ptPt.Y = v1.y;
    pVertices[1].Diffuse = reinterpret_cast<const DWORD &>(v1.a);

    if (fUseTriangles)
    {
        IFC(AddLineAsTriangleStrip(pVertices,pVertices+1));
    }
    
  Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::AddTriListVertices
//
//  Synopsis:  Reserve space for consecutive vertices and return start index
//

template <class TVertex>
MIL_FORCEINLINE
HRESULT
CHwTVertexBuffer<TVertex>::AddTriListVertices(
    UINT uDelta,
    __deref_ecount(uDelta) TVertex **ppVertices,
    __out_ecount(1) WORD *pwIndexStart
    )
{
    HRESULT hr = S_OK;

    Assert(ppVertices);

    UINT uCount = static_cast<UINT>(m_rgVerticesTriList.GetCount());
    if (uCount > SHRT_MAX)
    {
        IFC(WGXERR_INVALIDPARAMETER);
    }
    UINT newCount;
    newCount = uDelta + uCount;

    if (newCount > SHRT_MAX)
    {
        IFC(m_pBuilder->FlushReset());
        uCount = 0;
        newCount = uDelta;
    }

    if (newCount > m_rgVerticesTriList.GetCapacity())
    {
        IFC(m_rgVerticesTriList.ReserveSpace(uDelta));
    }

    m_rgVerticesTriList.SetCount(newCount);
    *pwIndexStart = static_cast<WORD>(uCount);
    *ppVertices = &m_rgVerticesTriList[uCount];

  Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::AddTriStripVertices
//
//  Synopsis:  Reserve space for consecutive vertices
//

template <class TVertex>
MIL_FORCEINLINE
HRESULT
CHwTVertexBuffer<TVertex>::AddTriStripVertices(
    UINT uCount,
    __deref_ecount(uCount) TVertex **ppVertices
    )
{
    HRESULT hr = S_OK;

    Assert(ppVertices);
#if DBG
    if (uCount != 6)
    {
        // Make a note that we added a tristrip using other than
        // 6 elements.
        m_fDbgNonLineSegmentTriangleStrip = true;
    }
#endif

    UINT Count = static_cast<UINT>(m_rgVerticesTriStrip.GetCount());
    UINT newCount = Count + uCount;

    if (newCount > m_rgVerticesTriStrip.GetCapacity())
    {
        IFC(m_rgVerticesTriStrip.ReserveSpace(uCount));
    }

    m_rgVerticesTriStrip.SetCount(newCount);
    *ppVertices = &m_rgVerticesTriStrip[Count];

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::AddNonIndexedTriListVertices
//
//  Synopsis:  Reserve space for triangle list vertices.
//

template <class TVertex>
MIL_FORCEINLINE
HRESULT
CHwTVertexBuffer<TVertex>::AddNonIndexedTriListVertices(
    UINT uCount,
    __deref_ecount(uCount) TVertex **ppVertices
    )
{
    HRESULT hr = S_OK;

    UINT Count = static_cast<UINT>(m_rgVerticesNonIndexedTriList.GetCount());
    UINT newCount = Count + uCount;

    if (newCount > m_rgVerticesNonIndexedTriList.GetCapacity())
    {
        IFC(m_rgVerticesNonIndexedTriList.ReserveSpace(uCount));
    }

    m_rgVerticesNonIndexedTriList.SetCount(newCount);
    *ppVertices = &m_rgVerticesNonIndexedTriList[Count];

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::AddLineListVertices
//
//  Synopsis:  Reserve space for consecutive vertices
//

template <class TVertex>
MIL_FORCEINLINE
HRESULT
CHwTVertexBuffer<TVertex>::AddLineListVertices(
    UINT uCount,
    __deref_ecount(uCount) TVertex **ppVertices
    )
{
    HRESULT hr = S_OK;

    Assert(ppVertices);

    UINT Count = static_cast<UINT>(m_rgVerticesLineList.GetCount());
    UINT newCount = Count + uCount;

    if (newCount > m_rgVerticesLineList.GetCapacity())
    {
        IFC(m_rgVerticesLineList.ReserveSpace(uCount));
    }

    m_rgVerticesLineList.SetCount(newCount);
    *ppVertices = &m_rgVerticesLineList[Count];

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Class:     CHwVertexBuffer::Builder
//
//-----------------------------------------------------------------------------

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::GetOutVertexFormat
//
//  Synopsis:  Return MIL vertex format covered by specific builders
//
//-----------------------------------------------------------------------------

template <>
MilVertexFormat
CHwTVertexBuffer<CD3DVertexXYZDUV2>::Builder::GetOutVertexFormat()
{
    return (MILVFAttrXYZ | MILVFAttrDiffuse | MILVFAttrUV2);
}

template <>
MilVertexFormat
CHwTVertexBuffer<CD3DVertexXYZDUV8>::Builder::GetOutVertexFormat()
{
    return (MILVFAttrXYZ | MILVFAttrDiffuse | MILVFAttrUV8);
}

template <>
MilVertexFormat
CHwTVertexBuffer<CD3DVertexXYZDUV6>::Builder::GetOutVertexFormat()
{
    return (MILVFAttrXYZ | MILVFAttrDiffuse | MILVFAttrUV6);
}

template <>
MilVertexFormat
CHwTVertexBuffer<CD3DVertexXYZNDSUV4>::Builder::GetOutVertexFormat()
{
    return (MILVFAttrXYZ |
            MILVFAttrNormal |
            MILVFAttrDiffuse |
            MILVFAttrSpecular |
            MILVFAttrUV4);
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwVertexBuffer::Builder::Create
//
//  Synopsis:  Choose the appropriate final vertex format and instantiate the
//             matching vertex builder
//

HRESULT
CHwVertexBuffer::Builder::Create(
    MilVertexFormat vfIn,
    MilVertexFormat vfOut,
    MilVertexFormatAttribute mvfaAntiAliasScaleLocation,
    __in_ecount_opt(1) CHwPipeline *pPipeline,
    __in_ecount_opt(1) CD3DDeviceLevel1 *pDevice,
    __in_ecount(1) CBufferDispenser *pBufferDispenser,
    __deref_out_ecount(1) CHwVertexBuffer::Builder **ppVertexBufferBuilder
    )
{
    HRESULT hr = S_OK;

    Assert(ppVertexBufferBuilder);

    *ppVertexBufferBuilder = NULL;

    if (!(vfOut & ~CHwTVertexBuffer<CD3DVertexXYZDUV2>::Builder::GetOutVertexFormat()))
    {
        CHwTVertexBuffer<CD3DVertexXYZDUV2> *pVB = pDevice->GetVB_XYZDUV2();
        CHwTVertexBuffer<CD3DVertexXYZDUV2>::Builder *pVBB = NULL;

        IFC(CHwTVertexBuffer<CD3DVertexXYZDUV2>::Builder::Create(
            pVB,
            vfIn,
            vfOut,
            mvfaAntiAliasScaleLocation,
            pBufferDispenser,
            &pVBB
            ));
        
        *ppVertexBufferBuilder = pVBB;
    }
    else if (!(vfOut & ~CHwTVertexBuffer<CD3DVertexXYZDUV8>::Builder::GetOutVertexFormat()))
    {
        CHwTVertexBuffer<CD3DVertexXYZDUV8> *pVB = pDevice->GetVB_XYZRHWDUV8();
        CHwTVertexBuffer<CD3DVertexXYZDUV8>::Builder *pVBB = NULL;

        IFC(CHwTVertexBuffer<CD3DVertexXYZDUV8>::Builder::Create(
            pVB,
            vfIn,
            vfOut,
            mvfaAntiAliasScaleLocation,
            pBufferDispenser,
            &pVBB
            ));

        *ppVertexBufferBuilder = pVBB;
    }
    else
    {
        // NOTE-2004/03/22-chrisra Adding another vertexbuffer type requires updating enum
        //
        // If we add another buffer builder type kMaxVertexBuilderSize enum in hwvertexbuffer.h file
        // needs to be updated to reflect possible changes to the maximum size of buffer builders.
        //
        IFC(E_NOTIMPL);
    }

    // Store the pipeline, if any, which this VBB can use to spill the vertex buffer to if it
    // overflows.
    (**ppVertexBufferBuilder).m_pPipelineNoRef = pPipeline;
    (**ppVertexBufferBuilder).m_pDeviceNoRef = pDevice;


Cleanup:
    RRETURN(hr);
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwVertexBuffer::Builder::TransferUIntIndicesAsWords
//
//  Synopsis:  Move indices from one buffer to another while converting them
//             from UINT to WORD.
//
//-----------------------------------------------------------------------------
void
CHwVertexBuffer::Builder::TransferUIntIndicesAsWords(
    __in_ecount(cIndices) const UINT *rguInputIndices,
    __out_ecount_full(cIndices) WORD *rgwOutputIndices,
    __range(1, UINT_MAX) UINT cIndices
    )
{
    // Align write pointer to at least four bytes
    UINT_PTR uOutAddress =
        reinterpret_cast<UINT_PTR>(static_cast<void *>(rgwOutputIndices));
    Assert( !(uOutAddress & 0x1) );
    if (uOutAddress & 0x2)
    {
        *rgwOutputIndices++ = static_cast<WORD>(*rguInputIndices++);
        cIndices--;
    }

    // Write and many double words as possible
    while (cIndices > 1)
    {
        DWORD dwTwoWordIndices = *rguInputIndices++ & 0xFFFF;
        dwTwoWordIndices |= *rguInputIndices++ << 16;
        *reinterpret_cast<DWORD *>(rgwOutputIndices) = dwTwoWordIndices;
        rgwOutputIndices += 2;
        cIndices -= 2;
    }

    // Write any remaining single index
    if (cIndices)
    {
        *rgwOutputIndices = static_cast<WORD>(*rguInputIndices);
    }
}




//+----------------------------------------------------------------------------
//
//  Class:     THwTVertexMappings<class TVertex>
//
//-----------------------------------------------------------------------------

//+----------------------------------------------------------------------------
//
//  Member:    THwTVertexMappings<TVertex>::THwTVertexMappings
//
//  Synopsis:  ctor
//
//-----------------------------------------------------------------------------

template <class TVertex>
CHwTVertexMappings<TVertex>::CHwTVertexMappings()
 :
    m_mvfMapped(MILVFAttrNone)
{
    for (int i = 0; i < ARRAY_SIZE(m_rgWaffleMode); ++i)
    {
        m_rgWaffleMode[i] = WaffleModeNone;
    }

    m_matPos2DTransform.SetIdentity();
}


//+----------------------------------------------------------------------------
//
//  Member:    THwTVertexMappings<TVertex>::SetPositionTransform
//
//  Synopsis:  Sets the position transform that needs to be applied.
//
//-----------------------------------------------------------------------------
template <class TVertex>
void 
CHwTVertexMappings<TVertex>::SetPositionTransform(
    __in_ecount(1) const MILMatrix3x2 &matPositionTransform
    )
{
    m_matPos2DTransform = matPositionTransform;
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexMappings<TVertex>::SetConstantMapping
//
//  Synopsis:  Remember the static color for the given vertex field
//

template <class TVertex>
HRESULT
CHwTVertexMappings<TVertex>::SetConstantMapping(
    MilVertexFormatAttribute mvfaLocation,
    __in_ecount(1) const CHwConstantColorSource *pConstCS
    )
{
    HRESULT hr = S_OK;

    Assert(!(m_mvfMapped & mvfaLocation));
    pConstCS->GetColor(m_colorStatic);
    m_mvfMapped |= mvfaLocation;    // Remember this field has been mapped

    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Function:  GetMILVFAttributeOfTextureCoord
//
//  Synopsis:  Compute MilVertexFormatAttribute for a texture coordinate index
//

MIL_FORCEINLINE
MilVertexFormat
GetMILVFAttributeOfTextureCoord(
    DWORD dwCoordIndex
    )
{
    return MILVFAttrUV1 << dwCoordIndex;
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexMappings<TVertex>::SetTextureMapping
//
//  Synopsis:  Remember the the transformation for generating texture
//             coordinates at the given index
//

template <class TVertex>
HRESULT
CHwTVertexMappings<TVertex>::SetTextureMapping(
    DWORD dwDestinationCoordIndex,
    DWORD dwSourceCoordIndex,
    __in_ecount(1) const MILMatrix3x2 *pmatDevicePointToTextureUV
    )
{
    HRESULT hr = S_OK;

    // The array size is not accessible to this class.  The assert is left here
    // for anyone debugging this code to check.
//    Assert(dwDestinationCoordIndex < ARRAY_SIZE(m_rgmatPointToUV));

    // Compute single bit of UV location from coord index
    MilVertexFormat mvfLocation =
        GetMILVFAttributeOfTextureCoord(dwDestinationCoordIndex);

    Assert(!(m_mvfMapped & mvfLocation));

    // Only mappings using matrix transforms from the position is supported
    if (dwSourceCoordIndex != MAXDWORD) IFC(E_NOTIMPL);
    if (!pmatDevicePointToTextureUV) IFC(E_NOTIMPL);

    m_rgmatPointToUV[dwDestinationCoordIndex] = *pmatDevicePointToTextureUV;
    
    m_mvfMapped |= mvfLocation;     // Remember this field has been mapped

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexMappings<TVertex>::PointToUV
//
//  Synopsis:  Helper function to populate the texture coordinates at the given
//             index using the given point
//

template <class TVertex>
MIL_FORCEINLINE void
CHwTVertexMappings<TVertex>::PointToUV(
    __in_ecount(1) const MilPoint2F &ptIn,
    __bound UINT uIndex,
    __out_ecount(1) TVertex *pvOut
    )
{
    m_rgmatPointToUV[uIndex].TransformPoint(
        &pvOut->ptTx[uIndex],
        ptIn.X,
        ptIn.Y
        );
}





//+----------------------------------------------------------------------------
//
//  Class:     CHwTVertexBuffer<TVertex>::Builder
//
//-----------------------------------------------------------------------------




//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::Create
//
//  Synopsis:  Instantiate a specific type of vertex builder
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::Create(
    __in_ecount(1) CHwTVertexBuffer<TVertex> *pVertexBuffer,
    MilVertexFormat mvfIn,
    MilVertexFormat mvfOut,
    MilVertexFormatAttribute mvfaAntiAliasScaleLocation,
    __inout_ecount(1) CBufferDispenser *pBufferDispenser,
    __deref_out_ecount(1) typename CHwTVertexBuffer<TVertex>::Builder **ppVertexBufferBuilder
    )
{
    HRESULT hr = S_OK;

    Assert(ppVertexBufferBuilder);

    *ppVertexBufferBuilder = NULL;

    CHwTVertexBuffer<TVertex>::Builder *pVertexBufferBuilder;

    pVertexBufferBuilder =
        new(pBufferDispenser) CHwTVertexBuffer<TVertex>::Builder(pVertexBuffer);
    IFCOOM(pVertexBufferBuilder);

    IFC(pVertexBufferBuilder->SetupConverter(
        mvfIn,
        mvfOut,
        mvfaAntiAliasScaleLocation
        ));

    *ppVertexBufferBuilder = pVertexBufferBuilder;
    pVertexBufferBuilder = NULL;

Cleanup:
    if (pVertexBufferBuilder)
    {
        delete pVertexBufferBuilder;
    }
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::Builder
//
//  Synopsis:  ctor
//
//-----------------------------------------------------------------------------

template <class TVertex>
CHwTVertexBuffer<TVertex>::Builder::Builder(
    __in_ecount(1) CHwTVertexBuffer<TVertex> *pVertexBuffer
    )
{
    Assert(pVertexBuffer);

    m_pVB = pVertexBuffer;


    m_rgoPrecomputedTriListVertices = NULL;
    m_cPrecomputedTriListVertices = 0;

    m_rguPrecomputedTriListIndices = NULL;
    m_cPrecomputedTriListIndices = 0;

    // These two track the Y extent of the shape this builder is producing.
    m_rCurStratumTop = +FLT_MAX;
    m_rCurStratumBottom = -FLT_MAX;
    m_fNeedOutsideGeometry = false;
    m_fNeedInsideGeometry = true;

    m_rLastTrapezoidRight = -FLT_MAX;
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::SetupConverter
//
//  Synopsis:  Choose the appropriate conversion method
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::SetupConverter(
    MilVertexFormat mvfIn,
    MilVertexFormat mvfOut,
    MilVertexFormatAttribute mvfaAntiAliasScaleLocation
    )
{
    HRESULT hr = S_OK;

    m_mvfIn = mvfIn;

#if DBG
    m_mvfDbgOut = mvfOut;
#endif

    m_mvfGenerated = mvfOut & ~m_mvfIn;
    m_mvfaAntiAliasScaleLocation = mvfaAntiAliasScaleLocation;

    Assert(!(m_mvfGenerated & MILVFAttrXY));

Cleanup:
    RRETURN(hr);
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::SetTransformMapping
//
//  Synopsis:  Delegate mapping sets to CHwTVertexMappings
//
//-----------------------------------------------------------------------------

template <class TVertex>
void
CHwTVertexBuffer<TVertex>::Builder::SetTransformMapping(
    __in_ecount(1) const MILMatrix3x2 &mat2DPositionTransform
    )
{
    m_map.SetPositionTransform(mat2DPositionTransform);
}
                                                                    
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::SetConstantMapping(
    MilVertexFormatAttribute mvfaLocation,
    __in_ecount(1) const CHwConstantColorSource *pConstCS
    )
{
    HRESULT hr = S_OK;

    IFC(m_map.SetConstantMapping(mvfaLocation, pConstCS));

Cleanup:
    RRETURN(hr);
}

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::SetTextureMapping(
    DWORD dwDestinationCoordIndex,
    DWORD dwSourceCoordIndex,
    __in_ecount(1) const MILMatrix3x2 *pmatDevicePointToTextureUV
    )
{
    HRESULT hr = S_OK;

    IFC(m_map.SetTextureMapping(
        dwDestinationCoordIndex,
        dwSourceCoordIndex,
        pmatDevicePointToTextureUV
        ));
    
Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::FinalizeMappings
//
//  Synopsis:  Complete setup of vertex mappings
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::FinalizeMappings(
    )
{
    HRESULT hr = S_OK;

    //
    // Set default Z if required.
    //

    if (m_mvfGenerated & MILVFAttrZ)
    {
        if (!(m_map.m_mvfMapped & MILVFAttrZ))
        {
            m_map.m_vStatic.Z = 0.5f;
        }
    }

    //
    // If AA falloff is not going to scale the diffuse color and it is
    // generated then see if the color is constant such that we can do any
    // complex conversions just once here instead of in every iteration of the
    // expansion loop.  If AA falloff is going to scale the diffuse color then
    // we can still optimize for the falloff = 1.0 case by precomputing that
    // color now and checking for 1.0 during generation.  Such a precomputation
    // has shown significant to performance.
    //

    if (m_mvfGenerated & MILVFAttrDiffuse)
    {
        if (m_map.m_mvfMapped & MILVFAttrDiffuse)
        {

            // Assumes diffuse color is constant
            m_map.m_vStatic.Diffuse =
                Convert_MilColorF_scRGB_To_Premultiplied_MilColorB_sRGB(&m_map.m_colorStatic);
        }
        else
        {
            // Set default Diffuse value: White
            m_map.m_vStatic.Diffuse = MIL_COLOR(0xFF,0xFF,0xFF,0xFF);
        }
    }

    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::SetOutsideBounds
//
//
//  Synopsis:  Enables rendering geometry for areas outside the shape but
//             within the bounds.  These areas will be created with
//             zero alpha.
//

template <class TVertex>
void
CHwTVertexBuffer<TVertex>::Builder::SetOutsideBounds(
    __in_ecount_opt(1) const CMILSurfaceRect *prcOutsideBounds,
    bool fNeedInside
    )
{
    // Waffling and outside bounds is not currently implemented.  It's
    // not difficult to do but currently there is no need.
    Assert(!(AreWaffling() && prcOutsideBounds));

    if (prcOutsideBounds)
    {
        m_rcOutsideBounds = *prcOutsideBounds;
        m_fNeedOutsideGeometry = true;
        m_fNeedInsideGeometry = fNeedInside;
    }
    else
    {
        m_fNeedOutsideGeometry = false;
        m_fNeedInsideGeometry = true;
    }
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::BeginBuilding
//
//  Synopsis:  Prepare for a new primitive by resetting the vertex buffer
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::BeginBuilding(
    )
{
    HRESULT hr = S_OK;

    m_fHasFlushed = false;
    m_pVB->Reset(this);

    // We need to know the viewport that this vertex buffer will be applied
    // to because a horizontal line through the first row of the viewport
    // can be incorrectly clipped.
    // This assumes that we've already set the viewport & we won't use
    // the vertex buffer with any other viewport.
    MilPointAndSizeL rcViewport = m_pDeviceNoRef->GetViewport();
    m_iViewportTop = rcViewport.Y;

    RRETURN(hr);
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddVertex
//
//  Synopsis:  Add a vertex to the vertex buffer
//
//             Remember just the given vertex information now and convert later
//             in a single, more optimal pass.
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddVertex(
    __in_ecount(1) const MilPoint2F &ptPosition,
        // Vertex coordinates
    __out_ecount(1) WORD *pIndex
        // The index of the new vertex
    )
{
    HRESULT hr = S_OK;

    Assert(!NeedOutsideGeometry());
    Assert(m_mvfIn == MILVFAttrXY);

    TVertex *pVertex;

    IFC(m_pVB->AddTriListVertices(1, &pVertex, pIndex));

    pVertex->ptPt = ptPosition;

    //  store coverage as a DWORD instead of float

    pVertex->Diffuse = FLOAT_ONE;

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddIndexedVertices, IGeometrySink
//
//  Synopsis:  Add a fully computed, indexed vertex to the vertex buffer
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddIndexedVertices(
    UINT cVertices,
        // In: number of vertices                                                       
    __in_bcount(cVertices*uVertexStride) const void *pVertexBufferNoRef,
        // In: vertex buffer containing the vertices                                    
    UINT uVertexStride,
        // In: size of each vertex                                                      
    MilVertexFormat mvfFormat,
        // In: format of each vertex                                                    
    UINT cIndices,
        // In: Number of indices                                                        
    __in_ecount(cIndices) const UINT *puIndexBuffer
        // In: index buffer                                                             
    )
{
    Assert(m_mvfIn & (MILVFAttrXYZ | MILVFAttrDiffuse | MILVFAttrUV2));
    Assert(mvfFormat == (MILVFAttrXYZ | MILVFAttrDiffuse | MILVFAttrUV2));

    Assert(uVertexStride == sizeof(TVertex));

    m_rgoPrecomputedTriListVertices = reinterpret_cast<const TVertex *>(pVertexBufferNoRef);
    m_cPrecomputedTriListVertices = cVertices;

    m_rguPrecomputedTriListIndices = puIndexBuffer;
    m_cPrecomputedTriListIndices = cIndices;

    return S_OK;
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddTriangle
//
//  Synopsis:  Add a triangle to the vertex buffer
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddTriangle(
    DWORD i1,                    // In: Index of triangle's first vertex
    DWORD i2,                    // In: Index of triangle's second vertex
    DWORD i3                     // In: Index of triangle's third vertex
    )
{
    HRESULT hr = S_OK;

    Assert(!NeedOutsideGeometry());

    if (AreWaffling())
    {
        TVertex *pVertex;
        UINT uNumVertices;
        m_pVB->GetTriListVertices(&pVertex, &uNumVertices);

        Assert(i1 < uNumVertices);
        Assert(i2 < uNumVertices);
        Assert(i3 < uNumVertices);

        PointXYA rgPoints[3];
        rgPoints[0].x = pVertex[i1].ptPt.X;
        rgPoints[0].y = pVertex[i1].ptPt.Y;
        rgPoints[0].a = 1;
        rgPoints[1].x = pVertex[i2].ptPt.X;
        rgPoints[1].y = pVertex[i2].ptPt.Y;
        rgPoints[1].a = 1;
        rgPoints[2].x = pVertex[i3].ptPt.X;
        rgPoints[2].y = pVertex[i3].ptPt.Y;
        rgPoints[2].a = 1;
        
        TriangleWaffler<PointXYA> wafflers[NUM_OF_VERTEX_TEXTURE_COORDS(TVertex) * 2];
        TriangleWaffler<PointXYA>::ISink *pWaffleSinkNoRef = BuildWafflePipeline(wafflers);
        IFC(pWaffleSinkNoRef->AddTriangle(rgPoints[0], rgPoints[1], rgPoints[2]));
    }
    else
    {
        IFC(m_pVB->AddTriangle(
                static_cast<WORD>(i1),
                static_cast<WORD>(i2),
                static_cast<WORD>(i3)
                ));
    }
    
Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::NeedCoverageGeometry
//
//  Synopsis:  Returns true if the coverage value needs to be rendered
//             based on NeedInsideGeometry() and NeedOutsideGeometry()
//
//             Two cases where we don't need to generate geometry:
//              1. NeedInsideGeometry is false, and coverage is c_nShiftSizeSquared.
//              2. NeedOutsideGeometry is false and coverage is 0
//
//-----------------------------------------------------------------------------
template <class TVertex>
MIL_FORCEINLINE bool
CHwTVertexBuffer<TVertex>::Builder::NeedCoverageGeometry(
    INT nCoverage
    ) const
{
    return    (NeedInsideGeometry()  || nCoverage != c_nShiftSizeSquared)
           && (NeedOutsideGeometry() || nCoverage != 0);
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddComplexScan
//
//  Synopsis:  Add a coverage span to the vertex buffer
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddComplexScan(
    INT nPixelY,
        // In: y coordinate in pixel space
    __in_ecount(1) const CCoverageInterval *pIntervalSpanStart
        // In: coverage segments
    )
{
    HRESULT hr = S_OK;
    TVertex *pVertex = NULL;

    IFC(PrepareStratum(static_cast<float>(nPixelY),
                  static_cast<float>(nPixelY+1),
                  false /* Not a trapezoid. */ ));

    float rPixelY;
    rPixelY = float(nPixelY) + 0.5f;

{
    LineWaffler<PointXYA> wafflers[NUM_OF_VERTEX_TEXTURE_COORDS(TVertex) * 2];

    // Use sink for waffling & the first line fix up (aka the complicated cases.)
    ILineSink<PointXYA> *pLineSink = NULL;

    if (AreWaffling())
    {
        bool fWafflersUsed;
        pLineSink = BuildWafflePipeline(wafflers, OUT fWafflersUsed);
        if (!fWafflersUsed)
        {
            pLineSink = NULL;
        }
    }
    
    // Use triangles instead of lines, for lines too close to the top of the viewport
    // because lines are clipped (before rasterization) against a viewport that only
    // includes half of the top pixel row.  Waffling will take care of this separately.
    if (!pLineSink && rPixelY < GetViewportTop() + 1 || FORCE_TRIANGLES)
    {
        pLineSink = m_pVB;
    }

    //
    // Output all segments if creating outside geometry, otherwise only output segments
    // with non-zero coverage.
    //

    if (!pLineSink)
    {
        UINT nSegmentCount = 0;

        for (const CCoverageInterval *pIntervalSpanTemp = pIntervalSpanStart;
             pIntervalSpanTemp->m_nPixelX != INT_MAX;
             pIntervalSpanTemp = pIntervalSpanTemp->m_pNext
             )
        {
            if (NeedCoverageGeometry(pIntervalSpanTemp->m_nCoverage))
            {
                ++nSegmentCount;
            }
        }

        //
        // Add vertices
        //
        if (nSegmentCount)
        {
            IFC(m_pVB->AddLineListVertices(nSegmentCount*2, &pVertex));
        }
    }

    //
    // Having allocated space (if not using sink), now let's actually output the vertices.
    //

    while (pIntervalSpanStart->m_nPixelX != INT_MAX)
    {
        Assert(pIntervalSpanStart->m_pNext != NULL);

        //
        // Output line list segments
        //
        // Note that line segments light pixels by going through the the
        // "diamond" interior of a pixel.  While we could accomplish this
        // by going from left edge to right edge of pixel, D3D10 uses the
        // convention that the LASTPIXEL is never lit.  We respect that now
        // by setting D3DRS_LASTPIXEL to FALSE and use line segments that
        // start in center of first pixel and end in center of one pixel
        // beyond last.
        //
        // Since our top left corner is integer, we add 0.5 to get to the
        // pixel center.
        //
        if (NeedCoverageGeometry(pIntervalSpanStart->m_nCoverage))
        {
            float rCoverage = static_cast<float>(pIntervalSpanStart->m_nCoverage)/static_cast<float>(c_nShiftSizeSquared);
            
            LONG iBegin = pIntervalSpanStart->m_nPixelX;
            LONG iEnd = pIntervalSpanStart->m_pNext->m_nPixelX;
            if (NeedOutsideGeometry())
            {
                // Intersect the interval with the outside bounds to create
                // start and stop lines.  The scan begins (ends) with an
                // interval starting (ending) at -inf (+inf).

                // The given geometry is not guaranteed to be within m_rcOutsideBounds but
                // the additional inner min and max (in that order) produce empty spans
                // for intervals not intersecting m_rcOutsideBounds.
                //
                // We could cull here but that should really be done by the geometry
                // generator.

                iBegin = max(iBegin, min(iEnd, m_rcOutsideBounds.left));
                iEnd = min(iEnd, max(iBegin, m_rcOutsideBounds.right));
            }
            float rPixelXBegin = float(iBegin) + 0.5f;
            float rPixelXEnd = float(iEnd) + 0.5f;

            //
            // Output line (linelist or tristrip) for a pixel
            //

            if (pLineSink)
            {
                PointXYA v0,v1;
                v0.x = rPixelXBegin;
                v0.y = rPixelY;
                v0.a = rCoverage;

                v1.x = rPixelXEnd;
                v1.y = rPixelY;
                v1.a = rCoverage;

                IFC(pLineSink->AddLine(v0,v1));
            }
            else
            {
                DWORD dwDiffuse = ReinterpretFloatAsDWORD(rCoverage);

                pVertex[0].ptPt.X = rPixelXBegin;
                pVertex[0].ptPt.Y = rPixelY;
                pVertex[0].Diffuse = dwDiffuse;

                pVertex[1].ptPt.X = rPixelXEnd;
                pVertex[1].ptPt.Y = rPixelY;
                pVertex[1].Diffuse = dwDiffuse;

                // Advance output vertex pointer
                pVertex += 2;
            }
        }

        //
        // Advance coverage buffer
        //

        pIntervalSpanStart = pIntervalSpanStart->m_pNext;
    }
}

Cleanup:
    RRETURN(hr);
}
//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddLineAsTriangleStrip
//
//  Synopsis:  Adds a horizontal line as a triangle strip to work around
//             issue in D3D9 where horizontal lines with y = 0 may not render.
//
//              Line clipping in D3D9
//             This behavior will change in D3D10 and this work-around will no
//             longer be needed.  (Pixel center conventions will also change.)
//              
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::AddLineAsTriangleStrip(
    __in_ecount(1) const TVertex *pBegin, // Begin
    __in_ecount(1) const TVertex *pEnd    // End
    )
{
    HRESULT hr = S_OK;
    TVertex *pVertex;

    // Collect pertinent data from vertices.
    Assert(pBegin->ptPt.Y == pEnd->ptPt.Y);
    Assert(pBegin->Diffuse == pEnd->Diffuse);

    // Offset begin and end X left by 0.5 because the line starts on the first
    // pixel center and ends on the center of the pixel after the line segment.
    const float x0 = pBegin->ptPt.X - 0.5f;
    const float x1 = pEnd->ptPt.X - 0.5f;
    const float y = pBegin->ptPt.Y;
    const DWORD dwDiffuse = pBegin->Diffuse;

    //
    // Add the vertices
    //

    IFC(AddTriStripVertices(6, &pVertex));

    //
    // Duplicate the first vertex.  Assuming that the previous two
    // vertices in the tristrip are coincident then the first three
    // vertices here create degenerate triangles.  If this is the
    // beginning of the strip the first two vertices fill the pipe,
    // the third creates a degenerate vertex.  In either case the
    // fourth creates the first triangle in our quad.
    // 
    pVertex[0].ptPt.X = x0;
    pVertex[0].ptPt.Y = y  - 0.5f;
    pVertex[0].Diffuse = dwDiffuse;
    
    // Offset two vertices up and two down to form a 1-pixel-high quad.
    // Order is TL-BL-TR-BR.
    pVertex[1].ptPt.X = x0;
    pVertex[1].ptPt.Y = y  - 0.5f;
    pVertex[1].Diffuse = dwDiffuse;
    pVertex[2].ptPt.X = x0;
    pVertex[2].ptPt.Y = y  + 0.5f;
    pVertex[2].Diffuse = dwDiffuse;
    pVertex[3].ptPt.X = x1;
    pVertex[3].ptPt.Y = y  - 0.5f;
    pVertex[3].Diffuse = dwDiffuse;
    pVertex[4].ptPt.X = x1;
    pVertex[4].ptPt.Y = y  + 0.5f;
    pVertex[4].Diffuse = dwDiffuse;
    
    //
    // Duplicate the last vertex. This creates a degenerate triangle
    // and sets up the next tristrip to create three more degenerate
    // triangles.
    // 
    pVertex[5].ptPt.X = x1;
    pVertex[5].ptPt.Y = y  + 0.5f;
    pVertex[5].Diffuse = dwDiffuse;

  Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddParallelogram
//
//  Synopsis:  This function adds the coordinates of a parallelogram to the vertex strip buffer. 
//
//  Parameter: rgPosition contains four coordinates of the parallelogram. Coordinates should have 
//              a winding order
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddParallelogram(
        __in_ecount(4)  const MilPoint2F *rgPosition
        )
{
    HRESULT hr = S_OK;

    if (AreWaffling())
    {
        PointXYA rgPoints[4];
        for (int i = 0; i < 4; ++i)
        {
            rgPoints[i].x = rgPosition[i].X;
            rgPoints[i].y = rgPosition[i].Y;
            rgPoints[i].a = 1;
        }
        TriangleWaffler<PointXYA> wafflers[NUM_OF_VERTEX_TEXTURE_COORDS(TVertex) * 2];
        TriangleWaffler<PointXYA>::ISink *pWaffleSinkNoRef = BuildWafflePipeline(wafflers);
        IFC(pWaffleSinkNoRef->AddTriangle(rgPoints[0], rgPoints[1], rgPoints[3]));
        IFC(pWaffleSinkNoRef->AddTriangle(rgPoints[3], rgPoints[1], rgPoints[2]));
    }
    else
    {
        TVertex *pVertex;
  
        //
        // Add the vertices
        //

        IFC(m_pVB->AddTriStripVertices(6, &pVertex));

        //
        // Duplicate the first vertex. This creates 2 degenerate triangles: one connecting
        // the previous rect to this one and another between vertices 0 and 1.
        //

        pVertex[0].ptPt = rgPosition[0];
        pVertex[0].Diffuse = FLOAT_ONE;

        pVertex[1].ptPt = rgPosition[0];
        pVertex[1].Diffuse = FLOAT_ONE;
    
        pVertex[2].ptPt = rgPosition[1];
        pVertex[2].Diffuse = FLOAT_ONE;

        pVertex[3].ptPt = rgPosition[3];
        pVertex[3].Diffuse = FLOAT_ONE;

        pVertex[4].ptPt = rgPosition[2];
        pVertex[4].Diffuse = FLOAT_ONE;

        //
        // Duplicate the last vertex. This creates 2 degenerate triangles: one
        // between vertices 4 and 5 and one connecting this Rect to the
        // next one.
        //

        pVertex[5].ptPt = rgPosition[2];
        pVertex[5].Diffuse = FLOAT_ONE;
    }
        
  Cleanup:
    RRETURN(hr);
}
    
//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::BuildWafflePipeline<TWaffler>
//
//  Synopsis:  Builds a pipeline of wafflers into the provided array of wafflers.
//             And returns a pointer (not to be deleted) to the input sink
//             of the waffle pipeline.
//             the final result is sinked int m_pVB.
//
//-----------------------------------------------------------------------------

template<class TVertex>
template<class TWaffler>
__out_ecount(1) typename TWaffler::ISink *
CHwTVertexBuffer<TVertex>::Builder::BuildWafflePipeline(
        __out_xcount(NUM_OF_VERTEX_TEXTURE_COORDS(TVertex) * 2) TWaffler *wafflers,
        __out_ecount(1) bool &fWafflersUsed
    ) const
{
    UINT count = 0;

    for (int i = 0; i < NUM_OF_VERTEX_TEXTURE_COORDS(TVertex); ++i)
    {
        if (m_map.m_rgWaffleMode[i] != 0)
        {
            const MILMatrix3x2 &pMatWaffle = m_map.m_rgmatPointToUV[i];

            // Each column ([a,b,c] transpose) of this matrix specifies a waffler that
            // partitions the plane into regions between the lines:
            //                    ax + by + c = k
            // for every integer k.
            //
            // If this partition width is substantially less than a pixel we have
            // serious problems with waffling generating too many triangles for
            // doubtful visual effect so we don't perform a waffling with width less
            // than c_rMinWaffleWidthPixels.  So we need to know the width of the partition
            // regions:
            //
            // Changing c just translates the partition so let's assume c = 0.
            // The line ax + by = 0 goes through the origin and the line ax + by
            // = 1 is adjacent to it in the partition.  The distance between
            // these lines is also the distance from ax + by = 1 to the origin.
            // Using Lagrange multipliers we can determine that this distance
            // is
            //                     1/sqrt(a*a+b*b).
            // We want to avoid waffling if this is less than c_rMinWaffleWidthPixels
            // or equivalently:
            //   1/sqrt(a*a+b*b) < c_rMinWaffleWidthPixels
            //     sqrt(a*a+b*b) > 1/c_rMinWaffleWidthPixels
            //          a*a+b*b  > 1/(c_rMinWaffleWidthPixels*c_rMinWaffleWidthPixels)
            //          

            const float c_rMaxWaffleMagnitude = 1/(c_rMinWaffleWidthPixels*c_rMinWaffleWidthPixels);
            
            float mag0 = pMatWaffle.m_00*pMatWaffle.m_00+pMatWaffle.m_10*pMatWaffle.m_10;
            if (mag0 < c_rMaxWaffleMagnitude)
            {
                wafflers[count].Set(pMatWaffle.m_00, pMatWaffle.m_10, pMatWaffle.m_20, wafflers+count+1);
                ++count;
            }

            float mag1 = pMatWaffle.m_01*pMatWaffle.m_01+pMatWaffle.m_11*pMatWaffle.m_11;
            if (mag1 < c_rMaxWaffleMagnitude)
            {
                wafflers[count].Set(pMatWaffle.m_01, pMatWaffle.m_11, pMatWaffle.m_21, wafflers+count+1);
                ++count;
            }
        }
    }

    if (count)
    {
        fWafflersUsed = true;
        // As the last step in the chain we send the triangles to our vertex buffer.
        wafflers[count-1].SetSink(m_pVB);
        return &wafflers[0];
    }
    else
    {
        fWafflersUsed = false;
        // If we built no wafflers then sink straight into the vertex buffer.
        return m_pVB;
    }
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::IsEmpty
//
//  Synopsis:  Does our VB have any triangles/lines?
//
//-----------------------------------------------------------------------------
template <class TVertex>
BOOL
CHwTVertexBuffer<TVertex>::Builder::IsEmpty()
{
    return m_pVB->IsEmpty();
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddTrapezoid
//
//  Synopsis:  Add a trapezoid to the vertex buffer
//
//
//      left edge       right edge
//      ___+_________________+___      <<< top edge
//     /  +  /             \  +  \
//    /  +  /               \  +  \
//   /  +  /                 \  +  \
//  /__+__/___________________\__+__\  <<< bottom edge
//    + ^^                        +
//      delta
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddTrapezoid(
    float rPixelYTop,              // In: y coordinate of top of trapezoid
    float rPixelXTopLeft,          // In: x coordinate for top left
    float rPixelXTopRight,         // In: x coordinate for top right
    float rPixelYBottom,           // In: y coordinate of bottom of trapezoid
    float rPixelXBottomLeft,       // In: x coordinate for bottom left
    float rPixelXBottomRight,      // In: x coordinate for bottom right
    float rPixelXLeftDelta,        // In: trapezoid expand radius for left edge
    float rPixelXRightDelta        // In: trapezoid expand radius for right edge
    )
{
    HRESULT hr = S_OK;

    if (AreWaffling())
    {
        IFC(AddTrapezoidWaffle(
                rPixelYTop,
                rPixelXTopLeft,
                rPixelXTopRight,
                rPixelYBottom,
                rPixelXBottomLeft,
                rPixelXBottomRight,
                rPixelXLeftDelta,
                rPixelXRightDelta));
    }
    else
    {
        IFC(AddTrapezoidStandard(
                rPixelYTop,
                rPixelXTopLeft,
                rPixelXTopRight,
                rPixelYBottom,
                rPixelXBottomLeft,
                rPixelXBottomRight,
                rPixelXLeftDelta,
                rPixelXRightDelta));
    }

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddTrapezoidStandard
//
//  Synopsis:  See AddTrapezoid.  This doesn't do waffling & uses tri strips.
//

template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddTrapezoidStandard(
    float rPixelYTop,              // In: y coordinate of top of trapezoid
    float rPixelXTopLeft,          // In: x coordinate for top left
    float rPixelXTopRight,         // In: x coordinate for top right
    float rPixelYBottom,           // In: y coordinate of bottom of trapezoid
    float rPixelXBottomLeft,       // In: x coordinate for bottom left
    float rPixelXBottomRight,      // In: x coordinate for bottom right
    float rPixelXLeftDelta,        // In: trapezoid expand radius for left edge
    float rPixelXRightDelta        // In: trapezoid expand radius for right edge
    )
{
    HRESULT hr = S_OK;
    TVertex *pVertex;

    IFC(PrepareStratum(
        rPixelYTop,
        rPixelYBottom,
        true, /* Trapezoid */
        min(rPixelXTopLeft, rPixelXBottomLeft),
        max(rPixelXTopRight, rPixelXBottomRight)
        ));
    
    //
    // Add the vertices
    //

	UINT cVertices;
	bool fNeedOutsideGeometry, fNeedInsideGeometry;
    cVertices = 8;
    fNeedOutsideGeometry = NeedOutsideGeometry();
    fNeedInsideGeometry = NeedInsideGeometry();

    if (!fNeedOutsideGeometry)
    {
        // For duplicates at beginning and end required to skip outside
        // geometry.
        cVertices += 2;
    }
       
    if (!fNeedInsideGeometry)
    {
        // For duplicates in middle required to skip inside geometry.
        cVertices += 2;
    }

    IFC(m_pVB->AddTriStripVertices(cVertices, &pVertex));

    if (!fNeedOutsideGeometry)
    {
        //
        // Duplicate the first vertex. This creates 2 degenerate triangles: one connecting
        // the previous trapezoid to this one and another between vertices 0 and 1.
        //

        pVertex->ptPt.X = rPixelXTopLeft - rPixelXLeftDelta;
        pVertex->ptPt.Y = rPixelYTop;
        pVertex->Diffuse = FLOAT_ZERO;
        ++pVertex;
    }

    //
    // Fill in the strip vertices
    //

    pVertex->ptPt.X = rPixelXTopLeft - rPixelXLeftDelta;
    pVertex->ptPt.Y = rPixelYTop;
    pVertex->Diffuse = FLOAT_ZERO;
    ++pVertex;

    pVertex->ptPt.X = rPixelXBottomLeft - rPixelXLeftDelta;
    pVertex->ptPt.Y = rPixelYBottom;
    pVertex->Diffuse = FLOAT_ZERO;
    ++pVertex;

    pVertex->ptPt.X = rPixelXTopLeft + rPixelXLeftDelta;
    pVertex->ptPt.Y = rPixelYTop;
    pVertex->Diffuse = FLOAT_ONE;
    ++pVertex;

    pVertex->ptPt.X = rPixelXBottomLeft + rPixelXLeftDelta;
    pVertex->ptPt.Y = rPixelYBottom;
    pVertex->Diffuse = FLOAT_ONE;
    ++pVertex;

    if (!fNeedInsideGeometry)
    {
        // Don't create inside geometry.
        pVertex->ptPt.X = rPixelXBottomLeft + rPixelXLeftDelta;
        pVertex->ptPt.Y = rPixelYBottom;
        pVertex->Diffuse = FLOAT_ONE;
        ++pVertex;
        
        pVertex->ptPt.X = rPixelXTopRight - rPixelXRightDelta;
        pVertex->ptPt.Y = rPixelYTop;
        pVertex->Diffuse = FLOAT_ONE;
        ++pVertex;
    }

    pVertex->ptPt.X = rPixelXTopRight - rPixelXRightDelta;
    pVertex->ptPt.Y = rPixelYTop;
    pVertex->Diffuse = FLOAT_ONE;
    ++pVertex;

    pVertex->ptPt.X = rPixelXBottomRight - rPixelXRightDelta;
    pVertex->ptPt.Y = rPixelYBottom;
    pVertex->Diffuse = FLOAT_ONE;
    ++pVertex;

    pVertex->ptPt.X = rPixelXTopRight + rPixelXRightDelta;
    pVertex->ptPt.Y = rPixelYTop;
    pVertex->Diffuse = FLOAT_ZERO;
    ++pVertex;

    pVertex->ptPt.X = rPixelXBottomRight + rPixelXRightDelta;
    pVertex->ptPt.Y = rPixelYBottom;
    pVertex->Diffuse = FLOAT_ZERO;
    ++pVertex;

    if (!fNeedOutsideGeometry)
    {
        //
        // Duplicate the last vertex. This creates 2 degenerate triangles: one
        // between vertices 8 and 9 and one connecting this trapezoid to the
        // next one.
        //

        pVertex->ptPt.X = rPixelXBottomRight + rPixelXRightDelta;
        pVertex->ptPt.Y = rPixelYBottom;
        pVertex->Diffuse = FLOAT_ZERO;
        ++pVertex;
    }

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::AddTrapezoidWaffle
//
//  Synopsis:  See AddTrapezoid.  This adds a waffled trapezoid.
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::AddTrapezoidWaffle(
    float rPixelYTop,              // In: y coordinate of top of trapezoid
    float rPixelXTopLeft,          // In: x coordinate for top left
    float rPixelXTopRight,         // In: x coordinate for top right
    float rPixelYBottom,           // In: y coordinate of bottom of trapezoid
    float rPixelXBottomLeft,       // In: x coordinate for bottom left
    float rPixelXBottomRight,      // In: x coordinate for bottom right
    float rPixelXLeftDelta,        // In: trapezoid expand radius for left edge
    float rPixelXRightDelta        // In: trapezoid expand radius for right edge
    )
{
    HRESULT hr = S_OK;

    // We have 2 (u & v) wafflers per texture coordinate that need waffling.
    TriangleWaffler<PointXYA> wafflers[NUM_OF_VERTEX_TEXTURE_COORDS(TVertex) * 2];
    bool fWafflersUsed = false;

    TriangleWaffler<PointXYA>::ISink *pWaffleSinkNoRef = BuildWafflePipeline(wafflers, OUT fWafflersUsed);

    PointXYA vertices[8];

    //
    // Fill in the strip vertices
    //

    // Nonstandard coverage mapping and waffling are not supported at the same time.
    Assert(!NeedOutsideGeometry());

    vertices[0].x = rPixelXTopLeft - rPixelXLeftDelta;
    vertices[0].y = rPixelYTop;
    vertices[0].a = 0;

    vertices[1].x = rPixelXBottomLeft - rPixelXLeftDelta;
    vertices[1].y = rPixelYBottom;
    vertices[1].a = 0;

    vertices[2].x = rPixelXTopLeft + rPixelXLeftDelta;
    vertices[2].y = rPixelYTop;
    vertices[2].a = 1;

    vertices[3].x = rPixelXBottomLeft + rPixelXLeftDelta;
    vertices[3].y = rPixelYBottom;
    vertices[3].a = 1;

    vertices[4].x = rPixelXTopRight - rPixelXRightDelta;
    vertices[4].y = rPixelYTop;
    vertices[4].a = 1;

    vertices[5].x = rPixelXBottomRight - rPixelXRightDelta;
    vertices[5].y = rPixelYBottom;
    vertices[5].a = 1;

    vertices[6].x = rPixelXTopRight + rPixelXRightDelta;
    vertices[6].y = rPixelYTop;
    vertices[6].a = 0;

    vertices[7].x = rPixelXBottomRight + rPixelXRightDelta;
    vertices[7].y = rPixelYBottom;
    vertices[7].a = 0;

    // Send the triangles in the strip through the waffle pipeline.
    for (int i = 0; i < 6; ++i)
    {
        IFC(pWaffleSinkNoRef->AddTriangle(vertices[i+1], vertices[i], vertices[i+2]));
    }

Cleanup:
    RRETURN(hr);
}
    
//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::PrepareStratumSlow
//
//  Synopsis:  Call before producing a new stratum (complex span or trapezoid.)
//             Handles several tasks:
//               1. Producing between top of complement geometry & the 1st
//                  stratum or when a gap between strata occurs (because
//                  the geometry is not closed and has horizontal gaps.)
//                  Passing in FLT_MAX for rStratumTop and rStratumBottom
//                  Fills the gap between the last stratum and the bottom
//                  of the outside.
//               2. Begins and/or ends the triangle strip corresponding to
//                  a trapezoid row.
//               3. Updates status vars m_rCurStratumTop & m_rCurStratumBottom
//
//  Note:      Call PrepareStratum which inlines the check for NeedOutsideGeometry()
//             If NeedOutsideGeometry is false PrepareStratum() does nothing.
//             This (slow) version asserts NeedOutsideGeometry()
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::PrepareStratumSlow(
    float rStratumTop,
    float rStratumBottom,
    bool fTrapezoid,
    float rTrapezoidLeft,
    float rTrapezoidRight
    )
{
    HRESULT hr = S_OK;
    
    Assert(!(rStratumTop > rStratumBottom));
    Assert(NeedOutsideGeometry());

    // There's only once case where a stratum can go "backwards"
    // and that's when we're done building & calling from
    // EndBuildingOutside
        
    float fEndBuildingOutside = rStratumBottom == OutsideBottom() &&
                                rStratumTop == OutsideBottom();

    if (fEndBuildingOutside)
    {
        Assert(!fTrapezoid);
    }
    else
    {
        Assert(!(rStratumBottom < m_rCurStratumBottom));
    }
    
    if (   fEndBuildingOutside
        || rStratumBottom != m_rCurStratumBottom)
    {
        
        // New stratum starting now.  Two things to do
        //  1. Close out current trapezoid stratum if necessary.
        //  2. Begin new trapezoid stratum if necessary.
        
        if (m_rCurStratumTop != FLT_MAX)
        {
            // End current trapezoid stratum.

            TVertex *pVertex;
            IFC(m_pVB->AddTriStripVertices(3, &pVertex));

            // we do not clip trapezoids so RIGHT boundary
            // of the stratus can be outside of m_rcOutsideBounds.
            
            float rOutsideRight = max(OutsideRight(), m_rLastTrapezoidRight);

            pVertex->ptPt.X = rOutsideRight;
            pVertex->ptPt.Y = m_rCurStratumTop;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            pVertex->ptPt.X = rOutsideRight;
            pVertex->ptPt.Y = m_rCurStratumBottom;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            // Duplicate last vertex in row
            pVertex->ptPt.X = rOutsideRight;
            pVertex->ptPt.Y = m_rCurStratumBottom;
            pVertex->Diffuse = FLOAT_ZERO;
        }
        // Compute the gap between where the last stratum ended and where
        // this one begins.
        float flGap = rStratumTop - m_rCurStratumBottom;

        if (flGap > 0)
        {
            // The "special" case of a gap at the beginning is caught here
            // using the sentinel initial value of m_rCurStratumBottom.

            float flRectTop = m_rCurStratumBottom == -FLT_MAX
                              ? OutsideTop()
                              : m_rCurStratumBottom;
            float flRectBot = static_cast<float>(rStratumTop);

            // Produce rectangular for any horizontal intervals in the
            // outside bounds that have no generated geometry.
            Assert(m_rCurStratumBottom != -FLT_MAX || m_rCurStratumTop == FLT_MAX);

            TVertex *pVertex;
            IFC(m_pVB->AddTriStripVertices(6, &pVertex));
            
            // Duplicate first vertex.
            pVertex->ptPt.X = OutsideLeft();
            pVertex->ptPt.Y = flRectTop;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            pVertex->ptPt.X = OutsideLeft();
            pVertex->ptPt.Y = flRectTop;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            pVertex->ptPt.X = OutsideLeft();
            pVertex->ptPt.Y = flRectBot;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            pVertex->ptPt.X = OutsideRight();
            pVertex->ptPt.Y = flRectTop;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            pVertex->ptPt.X = OutsideRight();
            pVertex->ptPt.Y = flRectBot;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;
        
            pVertex->ptPt.X = OutsideRight();
            pVertex->ptPt.Y = flRectBot;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;
        }

        if (fTrapezoid)
        {
            // Begin new trapezoid stratum.
            
            TVertex *pVertex;
            IFC(m_pVB->AddTriStripVertices(3, &pVertex));

            // we do not clip trapezoids so left boundary
            // of the stratus can be outside of m_rcOutsideBounds.
            
            float rOutsideLeft = min(OutsideLeft(), rTrapezoidLeft);

            // Duplicate first vertex.
            pVertex->ptPt.X = rOutsideLeft;
            pVertex->ptPt.Y = rStratumTop;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;
            
            pVertex->ptPt.X = rOutsideLeft;
            pVertex->ptPt.Y = rStratumTop;
            pVertex->Diffuse = FLOAT_ZERO;
            ++pVertex;

            pVertex->ptPt.X = rOutsideLeft;
            pVertex->ptPt.Y = rStratumBottom;
            pVertex->Diffuse = FLOAT_ZERO;
        }
    }
    
    if (fTrapezoid)
    {
        m_rLastTrapezoidRight = rTrapezoidRight;
    }

    m_rCurStratumTop = fTrapezoid ? rStratumTop : FLT_MAX;
    m_rCurStratumBottom = rStratumBottom;

Cleanup:
    RRETURN(hr);
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::EndBuildingOutside
//
//  Synopsis:  Finish creating outside geometry.
//             1. If no geometry was created then just fill bounds.
//             2. Otherwise:
//                 A. End last trapezoid row
//                 B. Produce stop stratum
// 
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::EndBuildingOutside()
{
    return PrepareStratum(
        OutsideBottom(),
        OutsideBottom(),
        false /* Not a trapezoid. */
        );
}

//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::EndBuilding
//
//  Synopsis:  Expand all vertices to the full required format and return
//             vertex buffer.
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::EndBuilding(
    __deref_opt_out_ecount(1) CHwVertexBuffer **ppVertexBuffer
    )
{
    HRESULT hr = S_OK;

    IFC(EndBuildingOutside());
    
    if (ppVertexBuffer)
    {
        *ppVertexBuffer = m_pVB;
    }

Cleanup:
    RRETURN(hr);
}


//+----------------------------------------------------------------------------
//
//  Member:    CHwTVertexBuffer<TVertex>::Builder::FlushInternal
//
//  Synopsis:  Send any pending state and geometry to the device.
//             If the optional argument is NULL then reset the
//             vertex buffer.
//             If the optional argument is non-NULL AND we have
//             not yet flushed the vertex buffer return the vertex
//             buffer.
//
//             These semantics allow the VB to be re-used for multipass
//             rendering if a single buffer sufficed for all of the geometry.
//             Otherwise multipass has to use a slower algorithm.
//
//-----------------------------------------------------------------------------
template <class TVertex>
HRESULT
CHwTVertexBuffer<TVertex>::Builder::FlushInternal(
    __deref_opt_out_ecount_opt(1) CHwVertexBuffer **ppVertexBuffer
    )
{
    HRESULT hr = S_OK;

    if (m_pPipelineNoRef)
    {
        // We use the pointer to the pipeline to ask it to send
        // the state if it hasn't been sent already.  Therefore after sending
        // we null it.
        IFC(m_pPipelineNoRef->RealizeColorSourcesAndSendState(m_pVB));
        m_pPipelineNoRef = NULL;
    }

    IFC(EndBuilding(NULL));

    if (m_rgoPrecomputedTriListVertices)
    {
        abort();
    }
    else
    {
        IFC(m_pVB->DrawPrimitive(m_pDeviceNoRef));
    }

  Cleanup:
    if (ppVertexBuffer)
    {
        if (!m_fHasFlushed)
        {
            *ppVertexBuffer = m_pVB;
        }
    }
    else
    {
        m_fHasFlushed = true;
        m_pVB->Reset(this);

        m_rgoPrecomputedTriListVertices = NULL;
        m_cPrecomputedTriListVertices = 0;

        m_rguPrecomputedTriListIndices = NULL;
        m_cPrecomputedTriListIndices = 0;
    }
    
    RRETURN(hr);
}


// 4505: unreferenced local function has been removed
//   These will show up as errors in very bizarre way including references to
//   shared\dynarray.h and the particular methods in
//   CHwTVertexBuffer<TVertex>::Builder, but what you won't find is a reference
//   to this line of explicit instantiation.  Proper placing of this was done
//   by trial and error :)
#pragma warning(disable : 4505)

// Explicit template instantiation
template class CHwTVertexBuffer<CD3DVertexXYZDUV2>;
template class CHwTVertexBuffer<CD3DVertexXYZDUV8>;






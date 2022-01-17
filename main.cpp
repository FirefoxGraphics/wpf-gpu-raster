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
        abort();
}

int main() {
}

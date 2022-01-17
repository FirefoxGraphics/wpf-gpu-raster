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
int main() {
}

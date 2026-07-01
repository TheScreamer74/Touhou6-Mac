// No-op stubs for the trimmed D3D methods that kept code (Draw3, LoadAnm) still
// calls. Geometry is captured via the device's SetTransform(WORLD); pixels,
// render-state and the vertex buffer are irrelevant to this oracle.
#include "AnmManager.hpp"
namespace th06 {
ZunResult AnmManager::LoadTexture(i32, char *, i32, D3DCOLOR) { return ZUN_SUCCESS; }
ZunResult AnmManager::CreateEmptyTexture(i32, u32, u32, i32) { return ZUN_SUCCESS; }
ZunResult AnmManager::LoadTextureAlphaChannel(i32, char *, i32, D3DCOLOR) { return ZUN_SUCCESS; }
void AnmManager::ReleaseTexture(i32) {}
void AnmManager::SetRenderStateForVm(AnmVm *) {}
void AnmManager::SetupVertexBuffer() {}
}

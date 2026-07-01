// d3d8 shim for the vertex oracle: like oracle/anm/d3d8.h but with a device that
// captures the world matrix Draw3 hands to SetTransform(D3DTS_WORLD) (the whole
// point of this oracle). Other device calls are no-ops.
#pragma once
#include <cstdint>
#include "d3dx8math.h"
typedef uint32_t D3DCOLOR;
typedef unsigned long DWORD;
typedef int D3DFORMAT;
struct IUnknownStub { long Release() { return 0; } };
struct IDirect3DTexture8 : IUnknownStub {};
struct IDirect3DSurface8 : IUnknownStub {};
struct IDirect3DVertexBuffer8 : IUnknownStub {
    long Lock(unsigned, unsigned, unsigned char**, unsigned long) { return 0; }
    long Unlock() { return 0; }
};

// The device is FakeDev (engine-stub Supervisor.hpp, extended by build.sh to
// capture the world matrix). These are the transform-state / primitive enums the
// decomp Draw3 references.
enum { D3DTS_WORLD = 256, D3DTS_VIEW = 2, D3DTS_PROJECTION = 3, D3DTS_TEXTURE0 = 16 };
typedef IDirect3DTexture8 *LPDIRECT3DTEXTURE8;
enum { D3DRS_ZFUNC = 23, D3DCMP_ALWAYS = 8, D3DCMP_LESSEQUAL = 4 };
enum { GAME_REGION_LEFT = 32, GAME_REGION_TOP = 16, GAME_REGION_WIDTH = 384, GAME_REGION_HEIGHT = 448 };
enum { D3DPT_TRIANGLESTRIP = 5 };
enum { D3DFVF_XYZ = 2, D3DFVF_DIFFUSE = 0x40, D3DFVF_TEX1 = 0x100 };
struct D3DXVECTOR4 { float x, y, z, w; };
struct D3DXIMAGE_INFO { unsigned Width, Height, Depth, MipLevels; };
enum { D3DFMT_UNKNOWN = 0, D3DFMT_A8R8G8B8 = 21, D3DFMT_A1R5G5B5 = 25, D3DFMT_R5G6B5 = 23, D3DFMT_R8G8B8 = 20, D3DFMT_A4R4G4B4 = 26 };

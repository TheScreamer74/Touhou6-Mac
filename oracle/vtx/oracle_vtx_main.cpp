// Vertex oracle: for each drawn bg quad, run the REAL decomp Draw3 to build the
// world matrix, then project the ±128 base quad through a spec-exact
// SetupCamera (view * proj) to screen space -- to diff against the port's
// scene() corner projections. Isolates the transform path no other oracle covers.
//
// Input (a quad list the port emits, so the STD parse is shared, not the suspect):
//   line 1:  F <frame> FACE <fx> <fy> <fz>
//   line N:  <anmScript> <px> <py> <pz> <sx> <sy>   (pos already quad+inst-stage)
// Usage: oracle_vtx <stgNbg.anm> <quadlist.txt>
#include <cstdio>
#include <cstdlib>
#include <cmath>
#include "AnmManager.hpp"
#include "Supervisor.hpp"
using namespace th06;

namespace th06 {
// Captured by FakeDev::SetTransform (see build.sh Supervisor.hpp patch).
D3DXMATRIX g_capturedWorld;
D3DXMATRIX g_capturedTexture;
Supervisor g_Supervisor;
namespace utils {
float AddNormalizeAngle(float a, float b)
{
    a += b;
    while (a > 3.14159265f) a -= 6.28318531f;
    while (a < -3.14159265f) a += 6.28318531f;
    return a;
}
}
namespace FileSystem {
uint8_t *OpenPath(char *path, bool)
{
    FILE *f = fopen(path, "rb");
    if (!f) return nullptr;
    fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
    uint8_t *b = (uint8_t *)malloc(n); fread(b, 1, n, f); fclose(f);
    return b;
}
}
}

static const float PI = 3.14159265358979f;

// Matches GameManager::SetupCamera (viewport = 384x448 arcadeRegion).
static void setupCamera(D3DXMATRIX *view, D3DXMATRIX *proj, float fx, float fy, float fz)
{
    const float W = 384.0f, H = 448.0f;
    const float midW = W / 2.0f, midH = H / 2.0f;
    const float fov = 30.0f * PI / 180.0f;
    const float camDist = midH / tanf(fov / 2.0f);
    D3DXVECTOR3 eye(midW, -midH, -camDist * fz);
    D3DXVECTOR3 at(midW + fx, -midH + fy, 0.0f);
    D3DXVECTOR3 up(0.0f, 1.0f, 0.0f);
    D3DXMatrixLookAtLH(view, &eye, &at, &up);
    D3DXMatrixPerspectiveFovLH(proj, fov, W / H, 100.0f, 10000.0f);
}

// Project a row-vector (lx,ly,0,1) * M to screen (viewport 384x448 at 32,16).
// Returns clip.w (for the near-plane guard).
static float project(const D3DXMATRIX &M, float lx, float ly, float &sx, float &sy)
{
    float v[4] = {lx, ly, 0.0f, 1.0f};
    float c[4];
    for (int j = 0; j < 4; j++)
        c[j] = v[0]*M.m[0][j] + v[1]*M.m[1][j] + v[2]*M.m[2][j] + v[3]*M.m[3][j];
    float ndcx = c[0] / c[3], ndcy = c[1] / c[3];
    sx = 32.0f + (ndcx * 0.5f + 0.5f) * 384.0f;
    sy = 16.0f + (0.5f - ndcy * 0.5f) * 448.0f;
    return c[3];
}

int main(int argc, char **argv)
{
    if (argc < 3) { fprintf(stderr, "usage: oracle_vtx <anm> <quadlist>\n"); return 1; }
    g_AnmManager = new AnmManager();
    if (g_AnmManager->LoadAnm(0, argv[1], 0) != ZUN_SUCCESS) { fprintf(stderr, "LoadAnm failed\n"); return 1; }

    FILE *q = fopen(argv[2], "r");
    if (!q) { fprintf(stderr, "open quadlist failed\n"); return 1; }
    int frame; float fx, fy, fz;
    if (fscanf(q, " F %d FACE %f %f %f", &frame, &fx, &fy, &fz) != 4) { fprintf(stderr, "bad header\n"); return 1; }

    D3DXMATRIX view, proj;
    setupCamera(&view, &proj, fx, fy, fz);

    int anmScript; float px, py, pz, sx_, sy_; int qidx = -1;
    while (fscanf(q, " %d %f %f %f %f %f", &anmScript, &px, &py, &pz, &sx_, &sy_) == 6)
    {
        qidx++;
        AnmVm vm;
        vm.Initialize();
        g_AnmManager->SetAndExecuteScriptIdx(&vm, anmScript);
        for (int i = 0; i < frame; i++) g_AnmManager->ExecuteScript(&vm);
        // RenderObjects (type 0): pos from STD, scale from quad size if set.
        vm.pos.x = px; vm.pos.y = py; vm.pos.z = pz;
        if (sx_ != 0.0f && vm.sprite) vm.scaleX = sx_ / vm.sprite->widthPx;
        if (sy_ != 0.0f && vm.sprite) vm.scaleY = sy_ / vm.sprite->heightPx;
        // Ensure Draw3's early-outs pass (geometry is independent of these).
        vm.flags.isVisible = 1;
        vm.flags.flag1 = 1;
        if (vm.color == 0) vm.color = 0xffffffffu;

        g_AnmManager->Draw3(&vm);

        if (const char *dq = getenv("QUAD_DEBUG"))
            if (atoi(dq) == qidx)
                fprintf(stderr,
                    "quad %d: anm=%d pos=(%.1f %.1f %.1f) size=(%.1f %.1f) "
                    "scaleX=%.4f scaleY=%.4f anchor=%u wpx=%.0f hpx=%.0f\n"
                    "  world row0=%.4f %.4f %.4f  row3(T)=%.2f %.2f %.2f\n",
                    qidx, anmScript, px, py, pz, sx_, sy_, vm.scaleX, vm.scaleY,
                    (unsigned)vm.flags.anchor, vm.sprite ? vm.sprite->widthPx : -1.0f,
                    vm.sprite ? vm.sprite->heightPx : -1.0f,
                    g_capturedWorld.m[0][0], g_capturedWorld.m[0][1], g_capturedWorld.m[0][2],
                    g_capturedWorld.m[3][0], g_capturedWorld.m[3][1], g_capturedWorld.m[3][2]);

        // world (row) * view * proj, then project ±128 corners (tl,tr,br,bl).
        D3DXMATRIX WV, M;
        D3DXMatrixMultiply(&WV, &g_capturedWorld, &view);
        D3DXMatrixMultiply(&M, &WV, &proj);
        float c[8], w[4];
        w[0] = project(M, -128.0f,  128.0f, c[0], c[1]);
        w[1] = project(M,  128.0f,  128.0f, c[2], c[3]);
        w[2] = project(M,  128.0f, -128.0f, c[4], c[5]);
        w[3] = project(M, -128.0f, -128.0f, c[6], c[7]);
        float minw = w[0];
        for (int k = 1; k < 4; k++) if (w[k] < minw) minw = w[k];
        printf("%.2f %.2f %.2f %.2f %.2f %.2f %.2f %.2f %.3f\n",
               c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], minw);
    }
    fclose(q);
    return 0;
}

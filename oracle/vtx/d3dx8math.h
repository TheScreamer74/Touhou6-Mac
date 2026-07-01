// Real D3DX8 matrix math for the vertex oracle — spec-exact (row-vector /
// row-major, LH), the reference ZUN's Draw3 + SetupCamera relied on. Vector
// types/operators mirror ../vm/stub/d3dx8math.h so the decomp headers compile;
// only the matrix functions are made real (the stub's are no-ops).
#pragma once
#include <cmath>
#include <cstdlib>

typedef struct D3DXVECTOR2 {
    float x, y;
    D3DXVECTOR2() : x(0), y(0) {}
    D3DXVECTOR2(float a, float b) : x(a), y(b) {}
} D3DXVECTOR2;

typedef struct D3DXVECTOR3 {
    float x, y, z;
    D3DXVECTOR3() : x(0), y(0), z(0) {}
    D3DXVECTOR3(float a, float b, float c) : x(a), y(b), z(c) {}
    D3DXVECTOR3 operator+(const D3DXVECTOR3& o) const { return D3DXVECTOR3(x+o.x, y+o.y, z+o.z); }
    D3DXVECTOR3 operator-(const D3DXVECTOR3& o) const { return D3DXVECTOR3(x-o.x, y-o.y, z-o.z); }
    D3DXVECTOR3 operator-() const { return D3DXVECTOR3(-x, -y, -z); }
    D3DXVECTOR3 operator*(float s) const { return D3DXVECTOR3(x*s, y*s, z*s); }
    D3DXVECTOR3 operator/(float s) const { return D3DXVECTOR3(x/s, y/s, z/s); }
    D3DXVECTOR3& operator+=(const D3DXVECTOR3& o) { x+=o.x; y+=o.y; z+=o.z; return *this; }
    D3DXVECTOR3& operator-=(const D3DXVECTOR3& o) { x-=o.x; y-=o.y; z-=o.z; return *this; }
    D3DXVECTOR3& operator*=(float s) { x*=s; y*=s; z*=s; return *this; }
    float operator[](int i) const { return (&x)[i]; }
    float& operator[](int i) { return (&x)[i]; }
} D3DXVECTOR3;

inline D3DXVECTOR3 operator*(float s, const D3DXVECTOR3& v) { return v * s; }

typedef struct D3DXMATRIX { float m[4][4]; } D3DXMATRIX;

inline float D3DXVec3Length(const D3DXVECTOR3* v) {
    return std::sqrt(v->x*v->x + v->y*v->y + v->z*v->z);
}
inline D3DXVECTOR3* D3DXVec3Normalize(D3DXVECTOR3* out, const D3DXVECTOR3* v) {
    float len = D3DXVec3Length(v);
    if (len > 0.0f) { out->x = v->x/len; out->y = v->y/len; out->z = v->z/len; }
    else { out->x = out->y = out->z = 0.0f; }
    return out;
}

inline D3DXMATRIX* D3DXMatrixIdentity(D3DXMATRIX* m) {
    for (int i = 0; i < 4; i++)
        for (int j = 0; j < 4; j++)
            m->m[i][j] = (i == j) ? 1.0f : 0.0f;
    return m;
}
// D3DX row-vector rotation matrices (v' = v * M).
inline D3DXMATRIX* D3DXMatrixRotationX(D3DXMATRIX* m, float a) {
    D3DXMatrixIdentity(m);
    float c = cosf(a), s = sinf(a);
    m->m[1][1] = c;  m->m[1][2] = s;
    m->m[2][1] = -s; m->m[2][2] = c;
    return m;
}
inline D3DXMATRIX* D3DXMatrixRotationY(D3DXMATRIX* m, float a) {
    D3DXMatrixIdentity(m);
    float c = cosf(a), s = sinf(a);
    m->m[0][0] = c;  m->m[0][2] = -s;
    m->m[2][0] = s;  m->m[2][2] = c;
    return m;
}
inline D3DXMATRIX* D3DXMatrixRotationZ(D3DXMATRIX* m, float a) {
    D3DXMatrixIdentity(m);
    float c = cosf(a), s = sinf(a);
    m->m[0][0] = c;  m->m[0][1] = s;
    m->m[1][0] = -s; m->m[1][1] = c;
    return m;
}
// out = a * b (row-major). Safe if out aliases a or b.
inline D3DXMATRIX* D3DXMatrixMultiply(D3DXMATRIX* out, const D3DXMATRIX* a, const D3DXMATRIX* b) {
    D3DXMATRIX r;
    for (int i = 0; i < 4; i++)
        for (int j = 0; j < 4; j++) {
            float s = 0.0f;
            for (int k = 0; k < 4; k++) s += a->m[i][k] * b->m[k][j];
            r.m[i][j] = s;
        }
    *out = r;
    return out;
}
// LH view matrix, D3DX convention.
inline D3DXMATRIX* D3DXMatrixLookAtLH(D3DXMATRIX* out, const D3DXVECTOR3* eye,
                                      const D3DXVECTOR3* at, const D3DXVECTOR3* up) {
    D3DXVECTOR3 zaxis = *at - *eye; D3DXVec3Normalize(&zaxis, &zaxis);
    D3DXVECTOR3 xaxis(up->y*zaxis.z - up->z*zaxis.y,
                      up->z*zaxis.x - up->x*zaxis.z,
                      up->x*zaxis.y - up->y*zaxis.x);
    D3DXVec3Normalize(&xaxis, &xaxis);
    D3DXVECTOR3 yaxis(zaxis.y*xaxis.z - zaxis.z*xaxis.y,
                      zaxis.z*xaxis.x - zaxis.x*xaxis.z,
                      zaxis.x*xaxis.y - zaxis.y*xaxis.x);
    auto dot = [](const D3DXVECTOR3& a, const D3DXVECTOR3& b){ return a.x*b.x+a.y*b.y+a.z*b.z; };
    D3DXMatrixIdentity(out);
    out->m[0][0]=xaxis.x; out->m[0][1]=yaxis.x; out->m[0][2]=zaxis.x;
    out->m[1][0]=xaxis.y; out->m[1][1]=yaxis.y; out->m[1][2]=zaxis.y;
    out->m[2][0]=xaxis.z; out->m[2][1]=yaxis.z; out->m[2][2]=zaxis.z;
    out->m[3][0]=-dot(xaxis,*eye); out->m[3][1]=-dot(yaxis,*eye); out->m[3][2]=-dot(zaxis,*eye);
    return out;
}
inline D3DXMATRIX* D3DXMatrixPerspectiveFovLH(D3DXMATRIX* out, float fovy, float aspect,
                                              float zn, float zf) {
    float yScale = 1.0f / tanf(fovy / 2.0f);
    float xScale = yScale / aspect;
    for (int i = 0; i < 4; i++) for (int j = 0; j < 4; j++) out->m[i][j] = 0.0f;
    out->m[0][0] = xScale;
    out->m[1][1] = yScale;
    out->m[2][2] = zf / (zf - zn);
    out->m[2][3] = 1.0f;
    out->m[3][2] = -zn * zf / (zf - zn);
    return out;
}
#define D3DCOLOR_RGBA(r,g,b,a) ((D3DCOLOR)(((a)<<24)|((r)<<16)|((g)<<8)|(b)))

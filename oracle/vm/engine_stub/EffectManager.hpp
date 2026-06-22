#include "Effect.hpp"
#include "inttypes.hpp"
namespace th06 {
#pragma once


struct EffectManager { Effect* SpawnParticles(int, D3DXVECTOR3*, int, u32){ static Effect e; return &e; } };
extern EffectManager g_EffectManager;
}

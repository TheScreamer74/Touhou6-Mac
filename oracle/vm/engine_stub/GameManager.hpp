#include "ZunMath.hpp"
#include "inttypes.hpp"
namespace th06 {
#pragma once


enum Difficulty { EASY=0, NORMAL=1, HARD=2, LUNATIC=3, EXTRA=4 };
struct Catk { char name[64]; u8 nameCsum; i16 numSuccess, numAttempts; i32 captureScore; u8 characterShotType; };
struct GameManager {
    i32 difficulty=NORMAL, rank=16, score=0, character=0, shotType=0;
    i32 isTimeStopped=0, currentPower=0, counat=0, spellcardsCaptured=0;
    bool isInReplay=false;
    Catk catk[64];
    D3DXVECTOR3 arcadeRegionTopLeftPos{32,16,0};
    i32 CharacterShotType(){ return character*2+shotType; }
    bool IsInBounds(f32,f32,f32,f32){ return true; }
    void AddScore(i32 s){ score+=s; }
    D3DXVECTOR3 playerMovementAreaSize{384,448,0};
    i32 livesRemaining=0, currentStage=0;
    void IncreaseSubrank(i32){}
};
extern GameManager g_GameManager;
}

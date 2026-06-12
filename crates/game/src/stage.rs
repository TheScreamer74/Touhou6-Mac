//! Stage gameplay driven by the original ECL scripts.
//!
//! The timeline spawns enemies exactly as the 2002 engine does; enemy
//! behavior runs in ecl_vm. The player, bomb and HUD live here.

use std::collections::HashMap;

use th06_engine::{DrawCmd, Input, Key};
use th06_formats::anm0::{Entry, Instr as AnmInstr, Sprite};
use th06_formats::ecl::Ecl;

use crate::anm_vm::AnmRunner;
use crate::ecl_vm::{Bullet, Enemy, Rng, SpawnReq, World, WorldEvent};

pub const FIELD_W: f32 = 384.0;
pub const FIELD_H: f32 = 448.0;
const FIELD_X: f32 = 32.0;
const FIELD_Y: f32 = 16.0;

// Texture slots, fixed by main.rs.
pub const TEX_PLAYER: usize = 2;
pub const TEX_BULLET: usize = 3;
pub const TEX_FAIRY: usize = 4;
pub const TEX_RUMIA: usize = 5;
pub const TEX_FRONT: usize = 6;
pub const TEX_WHITE: usize = 7;

pub enum Event {
    Sfx(&'static str),
    Bgm(&'static str),
    BackToTitle,
}

#[derive(Clone, Copy)]
struct SpriteRef {
    tex: usize,
    rect: [f32; 4],
}

const fn spr(tex: usize, x: f32, y: f32, w: f32, h: f32) -> SpriteRef {
    SpriteRef { tex, rect: [x, y, w, h] }
}

const REIMU_IDLE: [SpriteRef; 4] = [
    spr(TEX_PLAYER, 1.0, 1.0, 31.0, 47.0),
    spr(TEX_PLAYER, 33.0, 1.0, 31.0, 47.0),
    spr(TEX_PLAYER, 65.0, 1.0, 31.0, 47.0),
    spr(TEX_PLAYER, 97.0, 1.0, 31.0, 47.0),
];
const AMULET: SpriteRef = spr(TEX_PLAYER, 129.0, 1.0, 14.0, 14.0);
const NEEDLE: SpriteRef = spr(TEX_PLAYER, 193.0, 1.0, 14.0, 46.0);
const BOMB_GLOW: SpriteRef = spr(TEX_PLAYER, 1.0, 97.0, 62.0, 62.0);

const HUD_LOGO: SpriteRef = spr(TEX_FRONT, 128.0, 128.0, 128.0, 128.0);
const HUD_PLAYER_LABEL: SpriteRef = spr(TEX_FRONT, 0.0, 208.0, 32.0, 16.0);
const HUD_BOMB_LABEL: SpriteRef = spr(TEX_FRONT, 0.0, 240.0, 32.0, 16.0);
const HUD_STAR_RED: SpriteRef = spr(TEX_FRONT, 32.0, 240.0, 16.0, 16.0);
const HUD_STAR_GREEN: SpriteRef = spr(TEX_FRONT, 48.0, 240.0, 16.0, 16.0);

struct Shot {
    pos: [f32; 2],
    vel: [f32; 2],
    needle: bool,
}

enum PlayerState {
    Alive,
    Dead(u32),
    GameOver(u32),
    Cleared(u32),
}

/// One enemy ANM script with its sprite table and texture slot.
pub struct ScriptRef {
    pub tex: usize,
    pub instrs: Vec<AnmInstr>,
    pub sprites: HashMap<u32, Sprite>,
    pub tex_size: [f32; 2],
}

/// Build the enemy script table keyed by ANM script id (stg1enm uses ids
/// 0..7, stg1enm2 the boss bank at 128+), matching ECL ANMSETMAIN indices.
pub fn build_enemy_scripts(entries: &[(&Entry, usize)]) -> HashMap<i32, ScriptRef> {
    let mut out = HashMap::new();
    for (entry, tex) in entries {
        for (id, instrs) in &entry.scripts {
            out.insert(
                *id as i32,
                ScriptRef {
                    tex: *tex,
                    instrs: instrs.clone(),
                    sprites: entry.sprites.iter().map(|s| (s.index, s.clone())).collect(),
                    tex_size: [entry.width as f32, entry.height as f32],
                },
            );
        }
    }
    out
}

pub struct Stage {
    tick: u32,
    anim: u32,
    ecl: Ecl,
    world: World,
    enemies: Vec<Enemy>,
    anims: Vec<Option<AnmRunner>>,
    timeline_off: u32,
    timeline_time: i32,
    enemy_scripts: HashMap<i32, ScriptRef>,
    bullet_sprites: HashMap<u32, Sprite>,
    bullet_tex_size: [f32; 2],
    // player
    pos: [f32; 2],
    lives: i32,
    bombs: i32,
    invuln: u32,
    bombing: u32,
    fire_cd: u32,
    state: PlayerState,
    shots: Vec<Shot>,
    spell_active: bool,
    boss_bgm_started: bool,
    pub events: Vec<Event>,
}

impl Stage {
    pub fn new(ecl: Ecl, enemy_scripts: HashMap<i32, ScriptRef>, etama: &Entry) -> Self {
        let timeline_off = ecl.timeline_offset;
        Self {
            tick: 0,
            anim: 0,
            ecl,
            world: World {
                rng: Rng::new(0x1234),
                difficulty: 1, // Normal
                rank: 16,
                player_pos: [FIELD_W / 2.0, FIELD_H - 40.0],
                bullets: Vec::new(),
                events: Vec::new(),
                pending_spawns: Vec::new(),
                kill_trash: false,
                boss_present: false,
            },
            enemies: Vec::new(),
            anims: Vec::new(),
            timeline_off,
            timeline_time: 0,
            enemy_scripts,
            bullet_sprites: etama.sprites.iter().map(|s| (s.index, s.clone())).collect(),
            bullet_tex_size: [etama.width as f32, etama.height as f32],
            pos: [FIELD_W / 2.0, FIELD_H - 40.0],
            lives: 2,
            bombs: 3,
            invuln: 0,
            bombing: 0,
            fire_cd: 0,
            state: PlayerState::Alive,
            shots: Vec::new(),
            spell_active: false,
            boss_bgm_started: false,
            events: vec![Event::Bgm("th06_02.wav")],
        }
    }

    pub fn set_lives(&mut self, lives: i32) {
        self.lives = lives;
    }

    pub fn update(&mut self, input: &Input) -> Vec<DrawCmd> {
        self.tick += 1;
        self.anim += 1;

        // Player state machine.
        let mut respawn = false;
        match &mut self.state {
            PlayerState::Alive => {}
            PlayerState::Dead(t) => {
                *t -= 1;
                if *t == 0 {
                    if self.lives < 0 {
                        self.state = PlayerState::GameOver(180);
                    } else {
                        respawn = true;
                    }
                }
            }
            PlayerState::GameOver(t) | PlayerState::Cleared(t) => {
                *t -= 1;
                if *t == 0 {
                    self.events.push(Event::BackToTitle);
                    return self.draw();
                }
            }
        }
        if respawn {
            self.pos = [FIELD_W / 2.0, FIELD_H - 40.0];
            self.invuln = 180;
            self.state = PlayerState::Alive;
        }
        if matches!(self.state, PlayerState::Alive) {
            self.update_player(input);
        }
        self.invuln = self.invuln.saturating_sub(1);
        self.world.player_pos = self.pos;

        self.run_timeline();
        self.update_enemies();
        self.update_shots();
        self.update_bullets();
        self.collide();
        self.drain_world_events();

        // Stage clear: timeline exhausted and field empty.
        if matches!(self.state, PlayerState::Alive)
            && self.timeline_done()
            && self.enemies.is_empty()
            && !self.world.boss_present
        {
            self.state = PlayerState::Cleared(300);
        }

        self.draw()
    }

    fn timeline_done(&self) -> bool {
        self.ecl
            .timeline_at(self.timeline_off)
            .map(|t| t.time < 0)
            .unwrap_or(true)
    }

    /// Port of EnemyManager::RunEclTimeline (dialogue ops are skipped — no
    /// MSG interpreter yet).
    fn run_timeline(&mut self) {
        loop {
            // Copy the instruction out so the borrow of the ECL data ends
            // before any spawning mutates `self`.
            struct T {
                time: i32,
                arg0: i32,
                opcode: i16,
                size: u32,
                pos: [f32; 3],
                life: i16,
                item: i16,
                score: i32,
                a0: i32,
                a1: i32,
            }
            let t = {
                let Some(raw) = self.ecl.timeline_at(self.timeline_off) else { return };
                let has_full = raw.args.len() >= 20;
                T {
                    time: raw.time as i32,
                    arg0: raw.arg0 as i32,
                    opcode: raw.opcode,
                    size: raw.size as u32,
                    pos: if raw.args.len() >= 12 {
                        [raw.arg_f32(0), raw.arg_f32(4), raw.arg_f32(8)]
                    } else {
                        [0.0; 3]
                    },
                    life: if has_full { raw.arg_u16(12) as i16 } else { -1 },
                    item: if has_full { raw.arg_u16(14) as i16 } else { -2 },
                    score: if has_full { raw.arg_i32(16) } else { -1 },
                    a0: if raw.args.len() >= 4 { raw.arg_i32(0) } else { 0 },
                    a1: if raw.args.len() >= 8 { raw.arg_i32(4) } else { 0 },
                }
            };
            if t.time < 0 {
                return;
            }
            if self.timeline_time < t.time {
                break;
            }
            if self.timeline_time == t.time {
                match t.opcode {
                    0..=7 => {
                        if !self.world.boss_present {
                            let mut pos = t.pos;
                            if t.opcode >= 4 {
                                if pos[0] <= -990.0 {
                                    pos[0] = self.world.rng.f32_in_range(368.0);
                                }
                                if pos[1] <= -990.0 {
                                    pos[1] = self.world.rng.f32_in_range(416.0);
                                }
                                if pos[2] <= -990.0 {
                                    pos[2] = self.world.rng.f32_in_range(800.0);
                                }
                            }
                            let with_args = t.opcode % 2 == 0;
                            let (life, item, score) = if with_args {
                                (t.life, t.item, t.score)
                            } else {
                                (-1, -2, -1)
                            };
                            let mirror = matches!(t.opcode, 2 | 3 | 6 | 7);
                            self.spawn(SpawnReq { sub: t.arg0, pos, life, item, score, mirror });
                        }
                    }
                    8 | 9 => {} // dialogue — skipped until the MSG interpreter exists
                    10 => {
                        let interrupt = t.a1;
                        let _ = t.a0;
                        for e in &mut self.enemies {
                            if e.is_boss {
                                e.fire_interrupt(interrupt);
                            }
                        }
                    }
                    11 => {} // set power
                    12 => {
                        // Wait for the boss slot to clear.
                        if self.enemies.iter().any(|e| e.is_boss && e.occupied) {
                            return; // do not advance time
                        }
                    }
                    _ => {}
                }
            }
            self.timeline_off += t.size;
        }
        self.timeline_time += 1;
    }

    fn spawn(&mut self, req: SpawnReq) {
        if std::env::var_os("TH06_TRACE").is_some() {
            eprintln!(
                "[{}] spawn sub={} pos=({:.0},{:.0}) life={}",
                self.timeline_time, req.sub, req.pos[0], req.pos[1], req.life
            );
        }
        if let Some(e) = Enemy::spawn(&self.ecl, &mut self.world, &req) {
            self.enemies.push(e);
            self.anims.push(None);
        }
        self.flush_spawns();
    }

    fn flush_spawns(&mut self) {
        while let Some(req) = self.world.pending_spawns.pop() {
            if let Some(e) = Enemy::spawn(&self.ecl, &mut self.world, &req) {
                self.enemies.push(e);
                self.anims.push(None);
            }
        }
    }

    fn update_enemies(&mut self) {
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            if !e.occupied {
                continue;
            }
            e.frame_move();
            if !e.update_bounds() {
                e.occupied = false;
                continue;
            }
            e.handle_callbacks(&self.ecl, &mut self.world);
            e.run_ecl(&self.ecl, &mut self.world);

            // Refresh the ANM runner when the script changed.
            if e.anm_dirty {
                e.anm_dirty = false;
                self.anims[i] = self
                    .enemy_scripts
                    .get(&e.anm_script)
                    .map(|s| AnmRunner::new(s.instrs.clone()));
            }
            if let Some(anim) = &mut self.anims[i] {
                anim.tick();
            }
        }
        self.flush_spawns();

        if self.world.kill_trash {
            self.world.kill_trash = false;
            for i in 0..self.enemies.len() {
                let e = &mut self.enemies[i];
                if e.occupied && !e.is_boss {
                    e.life = 0;
                    e.on_death(&self.ecl, &mut self.world);
                }
            }
        }

        // Compact dead slots.
        let mut kept_anims = Vec::with_capacity(self.anims.len());
        let mut idx = 0;
        self.enemies.retain(|e| {
            let keep = e.occupied;
            if keep {
                kept_anims.push(self.anims[idx].take());
            }
            idx += 1;
            keep
        });
        self.anims = kept_anims;
    }

    fn update_player(&mut self, input: &Input) {
        let focus = input.held(Key::Focus);
        let speed = if focus { 2.0 } else { 4.0 };
        let mut d = [0.0f32, 0.0f32];
        if input.held(Key::Left) {
            d[0] -= 1.0;
        }
        if input.held(Key::Right) {
            d[0] += 1.0;
        }
        if input.held(Key::Up) {
            d[1] -= 1.0;
        }
        if input.held(Key::Down) {
            d[1] += 1.0;
        }
        if d[0] != 0.0 && d[1] != 0.0 {
            let inv = 1.0 / 2.0f32.sqrt();
            d[0] *= inv;
            d[1] *= inv;
        }
        self.pos[0] = (self.pos[0] + d[0] * speed).clamp(12.0, FIELD_W - 12.0);
        self.pos[1] = (self.pos[1] + d[1] * speed).clamp(20.0, FIELD_H - 20.0);

        self.fire_cd = self.fire_cd.saturating_sub(1);
        if input.held(Key::Shoot) && self.fire_cd == 0 {
            self.fire_cd = 4;
            if focus {
                for dx in [-5.0, 5.0] {
                    self.shots.push(Shot {
                        pos: [self.pos[0] + dx, self.pos[1] - 20.0],
                        vel: [0.0, -14.0],
                        needle: true,
                    });
                }
            } else {
                for (dx, vx) in [(-8.0, -0.6), (8.0, 0.6)] {
                    self.shots.push(Shot {
                        pos: [self.pos[0] + dx, self.pos[1] - 16.0],
                        vel: [vx, -12.0],
                        needle: false,
                    });
                }
            }
            if self.tick % 8 == 0 {
                self.events.push(Event::Sfx("plst00"));
            }
        }

        if self.bombing > 0 {
            self.bombing -= 1;
            self.world.bullets.clear();
            for e in &mut self.enemies {
                if e.occupied && e.interactable && e.damageable {
                    e.life -= 4;
                }
            }
        } else if input.pressed(Key::Bomb) && self.bombs > 0 {
            self.bombs -= 1;
            self.bombing = 120;
            self.invuln = self.invuln.max(180);
            self.events.push(Event::Sfx("power1"));
        }
    }

    fn update_shots(&mut self) {
        for s in &mut self.shots {
            s.pos[0] += s.vel[0];
            s.pos[1] += s.vel[1];
        }
        self.shots.retain(|s| s.pos[1] > -50.0);
    }

    fn update_bullets(&mut self) {
        for b in &mut self.world.bullets {
            let factor = if b.spawn_delay > 0 {
                b.spawn_delay -= 1;
                1.0 / 2.5
            } else {
                1.0
            };
            b.pos[0] += b.angle.cos() * b.speed * factor;
            b.pos[1] += b.angle.sin() * b.speed * factor;
        }
        self.world.bullets.retain(|b| {
            b.pos[0] > -20.0 && b.pos[0] < FIELD_W + 20.0 && b.pos[1] > -20.0 && b.pos[1] < FIELD_H + 20.0
        });
    }

    fn bullet_radius(&self, b: &Bullet) -> f32 {
        match self.bullet_sprites.get(&b.sprite).map(|s| s.height as u32) {
            Some(h) if h <= 8 => 2.0,
            Some(h) if h <= 16 => 3.2,
            Some(_) => 8.0,
            None => 3.0,
        }
    }

    fn collide(&mut self) {
        // Player shots vs enemies.
        let spell_penalty = self.spell_active && self.bombing == 0;
        for s in &mut self.shots {
            let mut dmg = if s.needle { 6 } else { 4 };
            if spell_penalty {
                dmg = (dmg / 7).max(1);
            }
            for e in &mut self.enemies {
                if !e.occupied || !e.interactable {
                    continue;
                }
                let r = (e.hitbox[0].max(e.hitbox[1])) / 2.0 + 6.0;
                let dx = s.pos[0] - e.pos[0];
                let dy = s.pos[1] - e.pos[1];
                if dx * dx + dy * dy < r * r {
                    if e.damageable {
                        e.life -= dmg;
                    }
                    s.pos[1] = -100.0;
                    break;
                }
            }
        }
        self.shots.retain(|s| s.pos[1] > -90.0);

        // Enemy deaths.
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            if e.occupied && e.interactable && e.life <= 0 {
                e.on_death(&self.ecl, &mut self.world);
                self.events.push(Event::Sfx("enep00"));
            }
        }
        self.flush_spawns();

        // Bullets / enemy bodies vs player.
        if !matches!(self.state, PlayerState::Alive) || self.invuln > 0 {
            return;
        }
        let p = self.pos;
        let radii: Vec<f32> = self.world.bullets.iter().map(|b| self.bullet_radius(b)).collect();
        let hit_bullet = self
            .world
            .bullets
            .iter()
            .zip(&radii)
            .any(|(b, r)| {
                let dx = b.pos[0] - p[0];
                let dy = b.pos[1] - p[1];
                let rr = r + 2.0;
                dx * dx + dy * dy < rr * rr
            });
        let hit_body = self.enemies.iter().any(|e| {
            if !e.occupied || !e.collidable || !e.interactable {
                return false;
            }
            let r = (e.hitbox[0].max(e.hitbox[1])) / 1.5 / 2.0 + 2.0;
            let dx = e.pos[0] - p[0];
            let dy = e.pos[1] - p[1];
            dx * dx + dy * dy < r * r
        });
        if hit_bullet || hit_body {
            self.lives -= 1;
            self.bombs = 3;
            self.world.bullets.clear();
            self.state = PlayerState::Dead(60);
            self.events.push(Event::Sfx("pldead00"));
        }
    }

    fn drain_world_events(&mut self) {
        let events: Vec<WorldEvent> = self.world.events.drain(..).collect();
        for ev in events {
            match ev {
                WorldEvent::Sfx(idx) => {
                    // SoundIdx -> our named sfx; a few common ones mapped.
                    let name = match idx {
                        0 => "plst00",
                        1 => "enep00",
                        5 => "power1",
                        16 => "tan00",
                        17 => "tan01",
                        18 => "tan02",
                        22 => "cat00",
                        _ => "tan00",
                    };
                    self.events.push(Event::Sfx(name));
                }
                WorldEvent::SpellcardStart(_name) => {
                    self.spell_active = true;
                    self.world.bullets.clear();
                    self.events.push(Event::Sfx("cat00"));
                }
                WorldEvent::SpellcardEnd => {
                    self.spell_active = false;
                    self.world.bullets.clear();
                }
                WorldEvent::BulletCancel => self.world.bullets.clear(),
                WorldEvent::BossSet(present) => {
                    if present && !self.boss_bgm_started {
                        self.boss_bgm_started = true;
                        self.events.push(Event::Bgm("th06_03.wav"));
                    }
                }
                WorldEvent::EnemyDeath(_pos) => {
                    self.events.push(Event::Sfx("enep00"));
                }
            }
        }
    }

    fn draw(&self) -> Vec<DrawCmd> {
        let mut cmds = Vec::with_capacity(96 + self.world.bullets.len());
        let base = if self.spell_active { 0.02 } else { 0.07 };
        cmds.push(rect(
            [FIELD_X, FIELD_Y, FIELD_W, FIELD_H],
            [base, base * 0.6, base * 0.9, 1.0],
        ));

        // Enemies via their ANM state.
        for (e, anim) in self.enemies.iter().zip(&self.anims) {
            if !e.occupied || e.invisible {
                continue;
            }
            let Some(anim) = anim else { continue };
            let Some(script) = self.enemy_scripts.get(&e.anm_script) else { continue };
            let Some(idx) = anim.sprite else { continue };
            let Some(sp) = script.sprites.get(&idx) else { continue };
            let [tw, th] = script.tex_size;
            let w = sp.width * anim.scale[0].abs();
            let h = sp.height * anim.scale[1].abs();
            // Sprite coordinates can exceed the sheet edge (Rumia); wrap.
            let sx = sp.x % tw;
            let sy = sp.y % th;
            let (mut u0, mut u1) = (sx / tw, (sx + sp.width) / tw);
            if anim.flip_x {
                std::mem::swap(&mut u0, &mut u1);
            }
            cmds.push(DrawCmd {
                tex: script.tex,
                dst: [
                    FIELD_X + e.pos[0] - w / 2.0,
                    FIELD_Y + e.pos[1] - h / 2.0,
                    w,
                    h,
                ],
                src: [u0, sy / th, u1, (sy + sp.height) / th],
                tint: [1.0, 1.0, 1.0, anim.alpha],
            });
        }

        // Boss HP bar.
        if let Some(boss) = self.enemies.iter().find(|e| e.is_boss && e.occupied) {
            let frac = (boss.life.max(0) as f32 / boss.max_life.max(1) as f32).clamp(0.0, 1.0);
            cmds.push(rect(
                [FIELD_X + 8.0, FIELD_Y + 4.0, (FIELD_W - 16.0) * frac, 4.0],
                [0.9, 0.15, 0.15, 0.9],
            ));
        }

        // Player shots.
        for s in &self.shots {
            let sp = if s.needle { NEEDLE } else { AMULET };
            cmds.push(sprite_at(sp, s.pos, 0.85));
        }

        // Player.
        if matches!(self.state, PlayerState::Alive | PlayerState::Cleared(_)) {
            let blink = self.invuln > 0 && (self.anim / 4) % 2 == 0;
            if !blink {
                cmds.push(sprite_at(REIMU_IDLE[(self.anim / 8) as usize % 4], self.pos, 1.0));
            }
        }

        // Bomb orbs.
        if self.bombing > 0 {
            let t = (120 - self.bombing) as f32;
            for i in 0..6 {
                let a = t * 0.08 + i as f32 / 6.0 * std::f32::consts::TAU;
                let r = 30.0 + t * 1.8;
                let pos = [self.pos[0] + a.cos() * r, self.pos[1] + a.sin() * r];
                let mut c = sprite_at(BOMB_GLOW, pos, 0.8);
                c.tint = [1.0, 0.45, 0.45, 0.75];
                cmds.push(c);
            }
        }

        // Bullets from the original scripts.
        let [tw, th] = self.bullet_tex_size;
        for b in &self.world.bullets {
            let Some(sp) = self.bullet_sprites.get(&b.sprite) else { continue };
            cmds.push(DrawCmd {
                tex: TEX_BULLET,
                dst: [
                    FIELD_X + b.pos[0] - sp.width / 2.0,
                    FIELD_Y + b.pos[1] - sp.height / 2.0,
                    sp.width,
                    sp.height,
                ],
                src: [sp.x / tw, sp.y / th, (sp.x + sp.width) / tw, (sp.y + sp.height) / th],
                tint: [1.0, 1.0, 1.0, 1.0],
            });
        }

        self.draw_hud(&mut cmds);
        cmds
    }

    fn draw_hud(&self, cmds: &mut Vec<DrawCmd>) {
        let border = [0.12, 0.05, 0.08, 1.0];
        cmds.push(rect([0.0, 0.0, 640.0, FIELD_Y], border));
        cmds.push(rect([0.0, FIELD_Y + FIELD_H, 640.0, 480.0 - FIELD_Y - FIELD_H], border));
        cmds.push(rect([0.0, 0.0, FIELD_X, 480.0], border));
        cmds.push(rect([FIELD_X + FIELD_W, 0.0, 640.0 - FIELD_X - FIELD_W, 480.0], border));

        let sx = FIELD_X + FIELD_W + 24.0;
        cmds.push(hud_sprite(HUD_PLAYER_LABEL, [sx, 120.0]));
        for i in 0..self.lives.max(0) {
            cmds.push(hud_sprite(HUD_STAR_RED, [sx + 40.0 + i as f32 * 18.0, 120.0]));
        }
        cmds.push(hud_sprite(HUD_BOMB_LABEL, [sx, 144.0]));
        for i in 0..self.bombs.max(0) {
            cmds.push(hud_sprite(HUD_STAR_GREEN, [sx + 40.0 + i as f32 * 18.0, 144.0]));
        }
        let mut logo = hud_sprite(HUD_LOGO, [sx - 4.0, 300.0]);
        logo.tint = [1.0, 1.0, 1.0, 0.85];
        cmds.push(logo);

        match self.state {
            PlayerState::GameOver(_) => {
                cmds.push(rect([FIELD_X, FIELD_Y, FIELD_W, FIELD_H], [0.0, 0.0, 0.0, 0.6]));
            }
            PlayerState::Cleared(_) => {
                cmds.push(rect([FIELD_X, FIELD_Y, FIELD_W, FIELD_H], [1.0, 1.0, 1.0, 0.12]));
            }
            _ => {}
        }
    }
}

fn rect(dst: [f32; 4], tint: [f32; 4]) -> DrawCmd {
    DrawCmd { tex: TEX_WHITE, dst, src: [0.25, 0.25, 0.75, 0.75], tint }
}

fn sprite_at(s: SpriteRef, field_pos: [f32; 2], alpha: f32) -> DrawCmd {
    let [x, y, w, h] = s.rect;
    DrawCmd {
        tex: s.tex,
        dst: [FIELD_X + field_pos[0] - w / 2.0, FIELD_Y + field_pos[1] - h / 2.0, w, h],
        src: [x / 256.0, y / 256.0, (x + w) / 256.0, (y + h) / 256.0],
        tint: [1.0, 1.0, 1.0, alpha],
    }
}

fn hud_sprite(s: SpriteRef, screen_pos: [f32; 2]) -> DrawCmd {
    let [x, y, w, h] = s.rect;
    DrawCmd {
        tex: s.tex,
        dst: [screen_pos[0], screen_pos[1], w, h],
        src: [x / 256.0, y / 256.0, (x + w) / 256.0, (y + h) / 256.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    }
}

//! Stage 1 gameplay: Reimu, fairy waves, Rumia, lives/bombs, HUD.
//!
//! Patterns are a hand-scripted approximation of the original stage; the
//! real ECL interpreter replaces this in a later milestone. Coordinates
//! are playfield-relative (384x448), drawn at screen offset (32, 16).

use th06_engine::{DrawCmd, Input, Key};

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
    // Pixel rect in the 256x256 sheet.
    rect: [f32; 4],
}

const fn spr(tex: usize, x: f32, y: f32, w: f32, h: f32) -> SpriteRef {
    SpriteRef { tex, rect: [x, y, w, h] }
}

// Reimu (player00.anm): idle frames, amulet, needle, bomb glow.
const REIMU_IDLE: [SpriteRef; 4] = [
    spr(TEX_PLAYER, 1.0, 1.0, 31.0, 47.0),
    spr(TEX_PLAYER, 33.0, 1.0, 31.0, 47.0),
    spr(TEX_PLAYER, 65.0, 1.0, 31.0, 47.0),
    spr(TEX_PLAYER, 97.0, 1.0, 31.0, 47.0),
];
const AMULET: SpriteRef = spr(TEX_PLAYER, 129.0, 1.0, 14.0, 14.0);
const NEEDLE: SpriteRef = spr(TEX_PLAYER, 193.0, 1.0, 14.0, 46.0);
const BOMB_GLOW: SpriteRef = spr(TEX_PLAYER, 1.0, 97.0, 62.0, 62.0);

// Fairies (stg1enm.anm): 30x30 frames, row per color.
const FAIRY_BLUE: [SpriteRef; 4] = [
    spr(TEX_FAIRY, 1.0, 1.0, 30.0, 30.0),
    spr(TEX_FAIRY, 33.0, 1.0, 30.0, 30.0),
    spr(TEX_FAIRY, 65.0, 1.0, 30.0, 30.0),
    spr(TEX_FAIRY, 97.0, 1.0, 30.0, 30.0),
];
const FAIRY_PINK: [SpriteRef; 4] = [
    spr(TEX_FAIRY, 1.0, 33.0, 30.0, 30.0),
    spr(TEX_FAIRY, 33.0, 33.0, 30.0, 30.0),
    spr(TEX_FAIRY, 65.0, 33.0, 30.0, 30.0),
    spr(TEX_FAIRY, 97.0, 33.0, 30.0, 30.0),
];

// Rumia (stg1enm2.anm); sprite x coordinates wrap at the 256px sheet edge.
const RUMIA_IDLE: [SpriteRef; 4] = [
    spr(TEX_RUMIA, 0.0, 0.0, 32.0, 48.0),
    spr(TEX_RUMIA, 32.0, 0.0, 32.0, 48.0),
    spr(TEX_RUMIA, 64.0, 0.0, 32.0, 48.0),
    spr(TEX_RUMIA, 96.0, 0.0, 32.0, 48.0),
];
const RUMIA_CAST: SpriteRef = spr(TEX_RUMIA, 0.0, 48.0, 32.0, 48.0);

// Bullets (etama3.anm). Color picked via 16px row offsets.
const PELLET_RED: SpriteRef = spr(TEX_BULLET, 136.0, 208.0, 8.0, 8.0);
const BALL_RED: SpriteRef = spr(TEX_BULLET, 16.0, 32.0, 16.0, 16.0);
const BALL_BLUE: SpriteRef = spr(TEX_BULLET, 96.0, 32.0, 16.0, 16.0);
const RICE_RED: SpriteRef = spr(TEX_BULLET, 17.0, 64.0, 14.0, 16.0);

// HUD (front.anm).
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

struct Bullet {
    pos: [f32; 2],
    vel: [f32; 2],
    sprite: SpriteRef,
    radius: f32,
}

#[derive(PartialEq)]
enum FairyKind {
    Blue,
    Pink,
}

struct Enemy {
    pos: [f32; 2],
    vel: [f32; 2],
    hp: i32,
    kind: FairyKind,
    age: u32,
    fire_at: u32,
}

enum BossPhase {
    Entering,
    Normal,
    Spell,
    Dying(u32),
}

struct Boss {
    pos: [f32; 2],
    hp: i32,
    phase: BossPhase,
    age: u32,
    spiral: f32,
}

enum PlayerState {
    Alive,
    Dead(u32),
    GameOver(u32),
    Cleared(u32),
}

/// Tiny LCG; determinism is fine, fidelity comes later with the decomp RNG.
struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0
    }
    fn f32(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16777216.0
    }
}

pub struct Stage {
    tick: u32,
    rng: Rng,
    pos: [f32; 2],
    anim: u32,
    lives: i32,
    bombs: i32,
    invuln: u32,
    bombing: u32,
    fire_cd: u32,
    state: PlayerState,
    shots: Vec<Shot>,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    boss: Option<Boss>,
    boss_started: bool,
    pub events: Vec<Event>,
}

impl Stage {
    /// Debug aid for headless runs.
    pub fn set_lives(&mut self, lives: i32) {
        self.lives = lives;
    }

    pub fn new() -> Self {
        Self {
            tick: 0,
            rng: Rng(0x6a09e667),
            pos: [FIELD_W / 2.0, FIELD_H - 40.0],
            anim: 0,
            lives: 2,
            bombs: 3,
            invuln: 0,
            bombing: 0,
            fire_cd: 0,
            state: PlayerState::Alive,
            shots: Vec::new(),
            bullets: Vec::new(),
            enemies: Vec::new(),
            boss: None,
            boss_started: false,
            events: vec![Event::Bgm("th06_02.wav")],
        }
    }

    pub fn update(&mut self, input: &Input) -> Vec<DrawCmd> {
        self.tick += 1;
        self.anim += 1;

        let mut next_state: Option<PlayerState> = None;
        match &mut self.state {
            PlayerState::Alive => {}
            PlayerState::Dead(t) => {
                *t -= 1;
                if *t == 0 {
                    next_state = Some(if self.lives < 0 {
                        PlayerState::GameOver(180)
                    } else {
                        PlayerState::Alive
                    });
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
        if let Some(s) = next_state {
            if matches!(s, PlayerState::Alive) {
                self.pos = [FIELD_W / 2.0, FIELD_H - 40.0];
                self.invuln = 180;
            }
            self.state = s;
        }
        if self.alive() {
            self.update_player(input);
        }

        self.invuln = self.invuln.saturating_sub(1);
        self.run_script();
        self.update_enemies();
        self.update_boss();
        self.update_shots();
        self.update_bullets();
        self.collide();
        self.draw()
    }

    fn alive(&self) -> bool {
        matches!(self.state, PlayerState::Alive)
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

        // Shooting.
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

        // Bomb: Fantasy Seal.
        if self.bombing > 0 {
            self.bombing -= 1;
            self.bullets.clear();
            let dmg = 4;
            for e in &mut self.enemies {
                e.hp -= dmg;
            }
            if let Some(b) = &mut self.boss {
                if !matches!(b.phase, BossPhase::Entering | BossPhase::Dying(_)) {
                    b.hp -= dmg;
                }
            }
        } else if input.pressed(Key::Bomb) && self.bombs > 0 {
            self.bombs -= 1;
            self.bombing = 120;
            self.invuln = self.invuln.max(180);
            self.events.push(Event::Sfx("power1"));
        }
    }

    /// Timed wave script for the pre-boss section, then the boss.
    fn run_script(&mut self) {
        let t = self.tick;
        // Waves of blue fairies swooping in from the top corners.
        if (120..=480).contains(&t) && t % 40 == 0 {
            let from_left = (t / 40) % 2 == 0;
            let x = if from_left { 40.0 } else { FIELD_W - 40.0 };
            let vx = if from_left { 1.4 } else { -1.4 };
            self.spawn_fairy([x, -16.0], [vx, 2.2], FairyKind::Blue, 70);
        }
        // A line of pink fairies sweeping across.
        if (700..=940).contains(&t) && t % 40 == 0 {
            self.spawn_fairy([-16.0, 60.0], [3.0, 0.6], FairyKind::Pink, 50);
        }
        // Symmetric streams.
        if (1100..=1500).contains(&t) && t % 30 == 0 {
            let phase = ((t / 30) % 5) as f32;
            self.spawn_fairy([40.0 + phase * 70.0, -16.0], [0.0, 2.6], FairyKind::Blue, 60);
        }
        // Pre-boss rush.
        if (1700..=1940).contains(&t) && t % 24 == 0 {
            let x = 30.0 + self.rng.f32() * (FIELD_W - 60.0);
            self.spawn_fairy([x, -16.0], [0.0, 3.2], FairyKind::Pink, 45);
        }
        // Rumia.
        if t == 2300 && !self.boss_started {
            self.boss_started = true;
            self.boss = Some(Boss {
                pos: [FIELD_W / 2.0, -40.0],
                hp: 1800,
                phase: BossPhase::Entering,
                age: 0,
                spiral: 0.0,
            });
            self.events.push(Event::Bgm("th06_03.wav"));
        }
    }

    fn spawn_fairy(&mut self, pos: [f32; 2], vel: [f32; 2], kind: FairyKind, fire_at: u32) {
        self.enemies.push(Enemy { pos, vel, hp: 32, kind, age: 0, fire_at });
    }

    fn update_enemies(&mut self) {
        let mut fired: Vec<Bullet> = Vec::new();
        let mut killed = 0;
        let player = self.pos;
        let player_alive = self.alive();
        for e in &mut self.enemies {
            e.age += 1;
            e.pos[0] += e.vel[0];
            e.pos[1] += e.vel[1];
            // Blue fairies curve away after a while.
            if e.kind == FairyKind::Blue && e.age > 90 {
                e.vel[1] -= 0.05;
            }
            if e.age == e.fire_at && player_alive {
                let dx = player[0] - e.pos[0];
                let dy = player[1] - e.pos[1];
                let len = (dx * dx + dy * dy).sqrt().max(0.001);
                let v = [dx / len * 2.4, dy / len * 2.4];
                let sprite = if e.kind == FairyKind::Blue { PELLET_RED } else { RICE_RED };
                fired.push(Bullet { pos: e.pos, vel: v, sprite, radius: 3.0 });
            }
        }
        if !fired.is_empty() {
            self.events.push(Event::Sfx("tan00"));
        }
        self.bullets.append(&mut fired);
        self.enemies.retain(|e| {
            if e.hp <= 0 {
                killed += 1;
                return false;
            }
            e.pos[1] < FIELD_H + 24.0 && e.pos[1] > -60.0 && e.pos[0] > -24.0 && e.pos[0] < FIELD_W + 24.0
        });
        for _ in 0..killed {
            self.events.push(Event::Sfx("enep00"));
        }
    }

    fn update_boss(&mut self) {
        let Some(boss) = &mut self.boss else { return };
        boss.age += 1;
        let mut volley: Vec<Bullet> = Vec::new();
        let mut defeated = false;
        let player = self.pos;
        let aim = |from: [f32; 2], speed: f32| -> [f32; 2] {
            let dx = player[0] - from[0];
            let dy = player[1] - from[1];
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            [dx / len * speed, dy / len * speed]
        };

        match boss.phase {
            BossPhase::Entering => {
                boss.pos[1] += 1.5;
                if boss.pos[1] >= 96.0 {
                    boss.phase = BossPhase::Normal;
                    boss.age = 0;
                }
            }
            BossPhase::Normal => {
                boss.pos[0] = FIELD_W / 2.0 + (boss.age as f32 * 0.015).sin() * 90.0;
                // Ring of red balls.
                if boss.age % 90 == 20 {
                    for i in 0..24 {
                        let a = i as f32 / 24.0 * std::f32::consts::TAU;
                        volley.push(Bullet {
                            pos: boss.pos,
                            vel: [a.cos() * 1.6, a.sin() * 1.6],
                            sprite: BALL_RED,
                            radius: 4.5,
                        });
                    }
                    self.events.push(Event::Sfx("tan02"));
                }
                // Aimed rice fan.
                if boss.age % 50 == 0 {
                    let base = aim(boss.pos, 2.6);
                    for spread in [-0.35f32, 0.0, 0.35] {
                        let (s, c) = spread.sin_cos();
                        volley.push(Bullet {
                            pos: boss.pos,
                            vel: [base[0] * c - base[1] * s, base[0] * s + base[1] * c],
                            sprite: RICE_RED,
                            radius: 3.0,
                        });
                    }
                    self.events.push(Event::Sfx("tan01"));
                }
                if boss.hp <= 900 {
                    boss.phase = BossPhase::Spell;
                    boss.age = 0;
                    self.bullets.clear();
                    self.events.push(Event::Sfx("cat00"));
                }
            }
            BossPhase::Spell => {
                // "Darkness sign"-style rotating spiral.
                boss.pos[0] = FIELD_W / 2.0 + (boss.age as f32 * 0.01).sin() * 60.0;
                boss.pos[1] = 96.0 + (boss.age as f32 * 0.02).cos() * 24.0;
                if boss.age % 5 == 0 {
                    boss.spiral += 0.55;
                    for arm in 0..4 {
                        let a = boss.spiral + arm as f32 * std::f32::consts::FRAC_PI_2;
                        volley.push(Bullet {
                            pos: boss.pos,
                            vel: [a.cos() * 1.9, a.sin() * 1.9],
                            sprite: BALL_BLUE,
                            radius: 4.5,
                        });
                    }
                }
                if boss.age % 110 == 60 {
                    volley.push(Bullet {
                        pos: boss.pos,
                        vel: aim(boss.pos, 2.2),
                        sprite: BALL_RED,
                        radius: 4.5,
                    });
                    self.events.push(Event::Sfx("tan00"));
                }
                if boss.hp <= 0 {
                    boss.phase = BossPhase::Dying(60);
                    self.bullets.clear();
                    self.events.push(Event::Sfx("enep01"));
                }
            }
            BossPhase::Dying(ref mut t) => {
                *t -= 1;
                if *t == 0 {
                    defeated = true;
                }
            }
        }
        self.bullets.append(&mut volley);
        if defeated {
            self.boss = None;
            self.state = PlayerState::Cleared(300);
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
        for b in &mut self.bullets {
            b.pos[0] += b.vel[0];
            b.pos[1] += b.vel[1];
        }
        self.bullets.retain(|b| {
            b.pos[0] > -20.0 && b.pos[0] < FIELD_W + 20.0 && b.pos[1] > -20.0 && b.pos[1] < FIELD_H + 20.0
        });
    }

    fn collide(&mut self) {
        // Player shots vs enemies and boss.
        let mut hit_sfx = false;
        for s in &mut self.shots {
            let dmg = if s.needle { 6 } else { 4 };
            for e in &mut self.enemies {
                let dx = s.pos[0] - e.pos[0];
                let dy = s.pos[1] - e.pos[1];
                if dx * dx + dy * dy < 18.0 * 18.0 && e.hp > 0 {
                    e.hp -= dmg;
                    s.pos[1] = -100.0; // consume
                    hit_sfx = true;
                    break;
                }
            }
            if let Some(b) = &mut self.boss {
                if !matches!(b.phase, BossPhase::Entering | BossPhase::Dying(_)) {
                    let dx = s.pos[0] - b.pos[0];
                    let dy = s.pos[1] - b.pos[1];
                    if dx * dx + dy * dy < 28.0 * 28.0 {
                        b.hp -= dmg;
                        s.pos[1] = -100.0;
                        hit_sfx = true;
                    }
                }
            }
        }
        if hit_sfx && self.tick % 4 == 0 {
            self.events.push(Event::Sfx("damage00"));
        }
        self.shots.retain(|s| s.pos[1] > -90.0);

        // Bullets and enemy bodies vs player.
        if !self.alive() || self.invuln > 0 {
            return;
        }
        let p = self.pos;
        let hit_bullet = self
            .bullets
            .iter()
            .any(|b| {
                let dx = b.pos[0] - p[0];
                let dy = b.pos[1] - p[1];
                let r = b.radius + 2.0;
                dx * dx + dy * dy < r * r
            });
        let hit_body = self.enemies.iter().any(|e| {
            let dx = e.pos[0] - p[0];
            let dy = e.pos[1] - p[1];
            dx * dx + dy * dy < 18.0 * 18.0
        });
        if hit_bullet || hit_body {
            self.lives -= 1;
            self.bombs = 3;
            self.bullets.clear();
            self.state = PlayerState::Dead(60);
            self.events.push(Event::Sfx("pldead00"));
        }
    }

    fn draw(&self) -> Vec<DrawCmd> {
        let mut cmds = Vec::with_capacity(64 + self.bullets.len());

        // Playfield backdrop: near-black with a faint red night tint.
        let spell_dark = matches!(self.boss.as_ref().map(|b| &b.phase), Some(BossPhase::Spell));
        let base = if spell_dark { 0.02 } else { 0.07 };
        cmds.push(rect(
            [FIELD_X, FIELD_Y, FIELD_W, FIELD_H],
            [base, base * 0.6, base * 0.9, 1.0],
        ));

        // Enemies.
        for e in &self.enemies {
            let frames = match e.kind {
                FairyKind::Blue => &FAIRY_BLUE,
                FairyKind::Pink => &FAIRY_PINK,
            };
            let f = frames[(self.anim / 8) as usize % 4];
            cmds.push(sprite_at(f, e.pos, 1.0));
        }

        // Boss.
        if let Some(b) = &self.boss {
            let s = match b.phase {
                BossPhase::Spell => RUMIA_CAST,
                _ => RUMIA_IDLE[(self.anim / 10) as usize % 4],
            };
            cmds.push(sprite_at(s, b.pos, 1.0));
            // HP bar.
            let max = 1800.0;
            let frac = (b.hp.max(0) as f32 / max).clamp(0.0, 1.0);
            cmds.push(rect([FIELD_X + 8.0, FIELD_Y + 4.0, (FIELD_W - 16.0) * frac, 4.0], [0.9, 0.15, 0.15, 0.9]));
        }

        // Player shots.
        for s in &self.shots {
            let spr = if s.needle { NEEDLE } else { AMULET };
            cmds.push(sprite_at(spr, s.pos, 0.85));
        }

        // Player.
        if self.alive() || matches!(self.state, PlayerState::Cleared(_)) {
            let blink = self.invuln > 0 && (self.anim / 4) % 2 == 0;
            if !blink {
                cmds.push(sprite_at(REIMU_IDLE[(self.anim / 8) as usize % 4], self.pos, 1.0));
            }
        }

        // Bomb effect: expanding orbs orbiting the player.
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

        // Enemy bullets on top.
        for b in &self.bullets {
            cmds.push(sprite_at(b.sprite, b.pos, 1.0));
        }

        self.draw_hud(&mut cmds);
        cmds
    }

    fn draw_hud(&self, cmds: &mut Vec<DrawCmd>) {
        let border = [0.12, 0.05, 0.08, 1.0];
        // Opaque borders mask sprites leaving the playfield.
        cmds.push(rect([0.0, 0.0, 640.0, FIELD_Y], border));
        cmds.push(rect([0.0, FIELD_Y + FIELD_H, 640.0, 480.0 - FIELD_Y - FIELD_H], border));
        cmds.push(rect([0.0, 0.0, FIELD_X, 480.0], border));
        cmds.push(rect([FIELD_X + FIELD_W, 0.0, 640.0 - FIELD_X - FIELD_W, 480.0], border));

        let sx = FIELD_X + FIELD_W + 24.0;
        // Lives.
        cmds.push(hud_sprite(HUD_PLAYER_LABEL, [sx, 120.0]));
        for i in 0..self.lives.max(0) {
            cmds.push(hud_sprite(HUD_STAR_RED, [sx + 40.0 + i as f32 * 18.0, 120.0]));
        }
        // Bombs.
        cmds.push(hud_sprite(HUD_BOMB_LABEL, [sx, 144.0]));
        for i in 0..self.bombs.max(0) {
            cmds.push(hud_sprite(HUD_STAR_GREEN, [sx + 40.0 + i as f32 * 18.0, 144.0]));
        }
        // Emblem.
        let mut logo = hud_sprite(HUD_LOGO, [sx - 4.0, 300.0]);
        logo.tint = [1.0, 1.0, 1.0, 0.85];
        cmds.push(logo);

        // End-state overlays: dim the field.
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

/// Solid-color rectangle in screen pixels (white texture x tint).
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

use crate::config::RainConfig;

/// Simple xorshift64 PRNG — no external dependency needed
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xDEAD_BEEF_CAFE_BABE } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Returns a float in [0.0, 1.0)
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
    }

    /// Returns a float in [lo, hi)
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

/// A single raindrop particle
pub struct Raindrop {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub length: f32,
    pub width: f32,
    pub alpha: f32,
    pub fade_in: f32, // 0..1, increases to 1 quickly after spawn
}

/// A splash particle spawned when a raindrop hits the bottom
pub struct Splash {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub max_radius: f32,
    pub alpha: f32,
    pub life: f32, // 0..1, decreases over time
}

/// Manages all rain particles
pub struct ParticleSystem {
    pub drops: Vec<Raindrop>,
    pub splashes: Vec<Splash>,
    rng: Rng,

    // Cached config values
    screen_w: f32,
    screen_h: f32,
    spawn_rate: f32,     // drops per second
    base_speed: f32,     // pixels per second (vertical)
    angle_rad: f32,      // radians
    drop_length: f32,
    drop_width: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    color_a: f32,
    opacity: f32,
    splash_enabled: bool,
    splash_intensity: f32,
    enabled: bool,
    spawn_accumulator: f32,
}

impl ParticleSystem {
    pub fn new(screen_w: f32, screen_h: f32, config: &RainConfig) -> Self {
        let mut sys = Self {
            drops: Vec::with_capacity(4096),
            splashes: Vec::with_capacity(1024),
            rng: Rng::new(0xDEAD_DE5C_0000_0001_u64.wrapping_mul(42)),
            screen_w,
            screen_h,
            spawn_rate: 0.0,
            base_speed: 0.0,
            angle_rad: 0.0,
            drop_length: 0.0,
            drop_width: 0.0,
            color_r: 0.0,
            color_g: 0.0,
            color_b: 0.0,
            color_a: 0.0,
            opacity: 0.0,
            splash_enabled: false,
            splash_intensity: 0.0,
            enabled: false,
            spawn_accumulator: 0.0,
        };
        sys.update_config(config);
        sys
    }

    /// Resize the particle system when the screen dimensions change
    pub fn resize(&mut self, w: f32, h: f32) {
        self.screen_w = w;
        self.screen_h = h;
    }

    /// Update cached config values from a RainConfig
    pub fn update_config(&mut self, config: &RainConfig) {
        self.enabled = config.enabled;
        // intensity 0→50/s, 1→3000/s (exponential feel)
        self.spawn_rate = 50.0 + config.intensity * config.intensity * 2950.0;
        self.base_speed = 400.0 * config.speed;
        self.angle_rad = config.angle.to_radians();
        self.drop_length = config.drop_length;
        self.drop_width = config.drop_width;
        self.color_r = config.color.r as f32 / 255.0;
        self.color_g = config.color.g as f32 / 255.0;
        self.color_b = config.color.b as f32 / 255.0;
        self.color_a = config.color.a as f32 / 255.0;
        self.opacity = config.opacity;
        self.splash_enabled = config.splash_enabled;
        self.splash_intensity = config.splash_intensity;
    }

    /// Advance the simulation by `dt` seconds
    pub fn update(&mut self, dt: f32) {
        if !self.enabled {
            // Fade out existing drops quickly
            self.drops.retain_mut(|d| {
                d.alpha -= dt * 3.0;
                d.alpha > 0.0
            });
            self.splashes.retain_mut(|s| {
                s.life -= dt * 3.0;
                s.life > 0.0
            });
            return;
        }

        // Spawn new drops
        self.spawn_accumulator += self.spawn_rate * dt;
        let to_spawn = self.spawn_accumulator as u32;
        self.spawn_accumulator -= to_spawn as f32;

        for _ in 0..to_spawn {
            self.spawn_drop();
        }

        // Update drops
        let screen_h = self.screen_h;
        let screen_w = self.screen_w;
        let splash_enabled = self.splash_enabled;
        let splash_intensity = self.splash_intensity;

        // Collect splashes to spawn (avoid borrow issues)
        let mut new_splashes = Vec::new();

        self.drops.retain_mut(|drop| {
            // Fade in
            if drop.fade_in < 1.0 {
                drop.fade_in = (drop.fade_in + dt * 8.0).min(1.0);
            }

            // Move
            drop.x += drop.vx * dt;
            drop.y += drop.vy * dt;

            // Check if off-screen
            let off_bottom = drop.y > screen_h + 10.0;
            let off_sides = drop.x < -50.0 || drop.x > screen_w + 50.0;

            if off_bottom {
                // Maybe spawn a splash
                if splash_enabled && new_splashes.len() < 200 {
                    // Probabilistic splash based on splash_intensity
                    // Use a simple hash of the position for determinism
                    let hash = ((drop.x as u32).wrapping_mul(2654435761)) & 0xFF;
                    if (hash as f32 / 255.0) < splash_intensity {
                        new_splashes.push(Splash {
                            x: drop.x,
                            y: screen_h - 2.0,
                            radius: 0.0,
                            max_radius: drop.width * 3.0 + 2.0,
                            alpha: 0.8,
                            life: 1.0,
                        });
                    }
                }
                return false;
            }

            !off_sides
        });

        self.splashes.extend(new_splashes);

        // Update splashes
        self.splashes.retain_mut(|splash| {
            splash.life -= dt * 3.0;
            splash.radius += (splash.max_radius - splash.radius) * dt * 8.0;
            splash.alpha = splash.life * 0.6;
            splash.life > 0.0
        });
    }

    fn spawn_drop(&mut self) {
        let speed = self.base_speed * self.rng.range(0.7, 1.3);
        let vx = -self.angle_rad.sin() * speed;
        let vy = self.angle_rad.cos() * speed;

        // Spawn across top edge, offset horizontally based on angle
        let extra_w = (self.angle_rad.sin().abs() * self.screen_h).min(self.screen_w * 0.5);
        let x = self.rng.range(-extra_w, self.screen_w + extra_w);
        let y = self.rng.range(-self.drop_length * 2.0, -self.drop_length);

        let length = self.drop_length * self.rng.range(0.6, 1.4);
        let width = self.drop_width * self.rng.range(0.7, 1.3);

        self.drops.push(Raindrop {
            x,
            y,
            vx,
            vy,
            length,
            width,
            alpha: self.opacity * self.color_a,
            fade_in: 0.0,
        });
    }

    /// Get the RGBA color as [f32; 4]
    pub fn color(&self) -> [f32; 4] {
        [self.color_r, self.color_g, self.color_b, self.color_a * self.opacity]
    }
}

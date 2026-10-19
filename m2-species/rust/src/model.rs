use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default)]
pub struct V {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl V {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn mix(self, b: Self, t: f32) -> Self {
        Self::new(
            mix(self.x, b.x, t),
            mix(self.y, b.y, t),
            mix(self.z, b.z, t),
        )
    }
    pub fn distance(self, b: Self) -> f32 {
        ((self.x - b.x).powi(2) + (self.y - b.y).powi(2) + (self.z - b.z).powi(2)).sqrt()
    }
}
pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn smooth(v: f32) -> f32 {
    let t = v.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
#[derive(Clone, Copy)]
pub struct Habitat {
    pub p: V,
    pub kind: u32,
}
pub const HABITATS: [Habitat; 4] = [
    Habitat {
        p: V::new(-230., -110., 95.),
        kind: 0,
    },
    Habitat {
        p: V::new(80., -100., 60.),
        kind: 0,
    },
    Habitat {
        p: V::new(-95., -72., -195.),
        kind: 1,
    },
    Habitat {
        p: V::new(245., -117., -170.),
        kind: 2,
    },
];
pub const CRACKS: [V; 4] = [
    V::new(-265., -1., 236.),
    V::new(105., -1., 215.),
    V::new(-50., -1., -38.),
    V::new(275., -1., 12.),
];
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Soil,
    Emerging,
    Hook,
    Drifting,
    Attaching,
    Juvenile,
    Adult,
    Reproduce,
    Withering,
    Falling,
    Dead,
}
impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Soil => "soil",
            Self::Emerging => "emerging",
            Self::Hook => "hook",
            Self::Drifting => "drifting",
            Self::Attaching => "attaching",
            Self::Juvenile => "juvenile",
            Self::Adult => "adult",
            Self::Reproduce => "reproduce",
            Self::Withering => "withering",
            Self::Falling => "falling",
            Self::Dead => "dead",
        }
    }
}
#[derive(Clone)]
pub struct Seed {
    pub id: u32,
    pub generation: u32,
    pub cycle: u32,
    pub base: f32,
    pub phase: Phase,
    pub p: V,
    pub start: V,
    pub crack: V,
    pub anchor: V,
    pub habitat: Option<usize>,
    pub timer: f32,
    pub travel: f32,
    pub nights: u32,
    pub days: u32,
    pub maturity: f32,
    pub growth: f32,
    pub wilt: f32,
    pub hooks: [f32; 3],
    pub wings: f32,
    pub spin: f32,
    pub armed: bool,
    pub seen_day: bool,
    pub released: bool,
    pub offspring: u32,
}
impl Seed {
    pub fn new(id: u32, generation: u32, base: f32, crack: V) -> Self {
        Self {
            id,
            generation,
            cycle: 1,
            base,
            phase: Phase::Soil,
            p: crack,
            start: crack,
            crack,
            anchor: crack,
            habitat: None,
            timer: 0.,
            travel: 0.,
            nights: 0,
            days: 0,
            maturity: 0.,
            growth: 0.,
            wilt: 0.,
            hooks: [0.; 3],
            wings: 0.,
            spin: 0.,
            armed: false,
            seen_day: false,
            released: false,
            offspring: 0,
        }
    }
    pub fn size(&self) -> f32 {
        self.base * 0.90_f32.powi((self.cycle - 1) as i32)
    }
    pub fn body_size(&self) -> f32 {
        self.size()
            * if matches!(
                self.phase,
                Phase::Soil | Phase::Emerging | Phase::Hook | Phase::Drifting | Phase::Attaching
            ) {
                0.38
            } else if self.phase == Phase::Juvenile {
                mix(0.38, 1., self.maturity)
            } else {
                1.
            }
    }
    fn enter(&mut self, phase: Phase) {
        self.phase = phase;
        self.timer = 0.;
        self.start = self.p;
        if phase == Phase::Drifting {
            self.travel = 0.;
        }
    }
    fn night(&mut self, light: f32) -> bool {
        if light < 0.18 && self.seen_day {
            self.armed = true;
        }
        if light > 0.45 {
            self.seen_day = true;
            if self.armed {
                self.armed = false;
                return true;
            }
        }
        false
    }
    pub fn floating_position(&self, wind: f32, time: f32) -> V {
        V::new(
            self.anchor.x + (time * 1.5 + self.id as f32).sin() * wind * 2.,
            self.anchor.y - 12. - smooth(wind) * 18.,
            self.anchor.z + 35. + smooth(wind) * 58.,
        )
    }
    pub fn update(&mut self, dt: f32, light: f32, wind: f32, time: f32) -> bool {
        self.timer += dt;
        self.spin += dt * wind * 0.9;
        match self.phase {
            Phase::Soil => {
                if self.timer >= 3. && light >= 0.45 {
                    self.enter(Phase::Emerging);
                }
            }
            Phase::Emerging => {
                if light >= 0.45 {
                    self.p = self.crack.mix(
                        V::new(self.crack.x, -4., self.crack.z),
                        smooth(self.timer / 5.),
                    );
                } else {
                    self.timer = (self.timer - dt).max(0.);
                }
                if self.timer >= 5. && light >= 0.45 {
                    self.enter(Phase::Hook);
                }
            }
            Phase::Hook => {
                self.hooks[0] = (self.timer / 4.).min(1.);
                if self.hooks[0] >= 1. && wind > 0.20 {
                    self.enter(Phase::Drifting);
                }
            }
            Phase::Drifting => {
                let speed = 0.02 + wind * 0.13;
                if wind < 0.12 {
                    self.timer = (self.timer - dt).max(0.);
                    return false;
                }
                self.travel = (self.travel + dt * speed).min(1.);
                let t = self.travel;
                let dest = if self.habitat.is_some() {
                    self.floating_position(wind, time)
                } else {
                    V::new(self.start.x + 950., -130., self.start.z + 150.)
                };
                self.p = self.start.mix(dest, smooth(t));
                self.p.y -= (t * std::f32::consts::PI).sin() * 55.;
                if t >= 1. {
                    if self.habitat.is_some() {
                        self.enter(Phase::Attaching);
                    } else {
                        self.enter(Phase::Dead);
                    }
                }
            }
            Phase::Attaching => {
                self.p = self.floating_position(wind, time);
                if self.timer >= 2. {
                    self.enter(Phase::Juvenile);
                    self.seen_day = light > 0.45;
                    self.armed = false;
                }
            }
            Phase::Juvenile => {
                self.p = self.floating_position(wind, time);
                if self.night(light) {
                    self.nights = (self.nights + 1).min(3);
                }
                self.maturity = (self.maturity + dt * 0.12).min(self.nights as f32 / 3.);
                self.hooks = [
                    1.,
                    (self.maturity * 3.).min(1.),
                    (self.maturity * 3.).min(1.),
                ];
                self.wings = ((self.maturity - 1. / 3.) * 3.).clamp(0., 1.);
                self.growth = 0.;
                self.wilt = 0.;
                if self.nights == 3 && self.maturity >= 1. {
                    self.enter(Phase::Adult);
                    self.armed = false;
                    self.seen_day = light > 0.45;
                }
            }
            Phase::Adult => {
                self.p = self.floating_position(wind, time);
                if light >= 0.45 {
                    self.growth = (self.growth + dt * (0.08 + light * 0.08)).min(1.);
                    self.wilt = (self.wilt - dt * 0.8).max(0.);
                } else {
                    self.growth = (self.growth - dt * 0.18).max(0.);
                    self.wilt = (self.wilt + dt * 0.5).min(1.);
                }
                if self.night(light) {
                    self.days = (self.days + 1).min(7);
                }
                if self.days == 7 {
                    self.enter(Phase::Reproduce);
                }
            }
            Phase::Reproduce => {
                self.p = self.floating_position(wind, time);
                if light < 0.45 {
                    self.growth = (self.growth - dt * 0.18).max(0.);
                    self.wilt = (self.wilt + dt * 0.5).min(1.);
                }
                if wind >= 0.25 && !self.released {
                    self.released = true;
                    self.hooks = [1., 0., 0.];
                    return true;
                }
                if self.released && self.timer > 5. {
                    self.enter(Phase::Withering);
                }
            }
            Phase::Withering => {
                self.growth = (self.growth - dt * 0.25).max(0.);
                self.wilt = (self.wilt + dt * 0.35).min(1.);
                self.wings = (self.wings - dt * 0.15).max(0.);
                if self.timer >= 5. {
                    self.habitat = None;
                    self.enter(Phase::Falling);
                }
            }
            Phase::Falling => {
                self.hooks = self.hooks.map(|h| (h - dt).max(0.));
                self.p = self.start.mix(self.crack, smooth(self.timer / 4.));
                if self.timer >= 4. {
                    if self.cycle >= 7 {
                        self.enter(Phase::Dead);
                    } else {
                        self.cycle += 1;
                        self.enter(Phase::Soil);
                        self.nights = 0;
                        self.days = 0;
                        self.maturity = 0.;
                        self.wings = 0.;
                        self.growth = 0.;
                        self.wilt = 0.;
                        self.hooks = [0.; 3];
                        self.armed = false;
                        self.seen_day = false;
                        self.released = false;
                    }
                }
            }
            Phase::Dead => {}
        }
        false
    }
}
pub struct World {
    pub seeds: Vec<Seed>,
    pub next_id: u32,
    pub time: f32,
    pub wind: f32,
    pub light: f32,
    pub fan: f32,
}
impl Default for World {
    fn default() -> Self {
        Self {
            seeds: vec![Seed::new(1, 0, 1., CRACKS[0])],
            next_id: 2,
            time: 0.,
            wind: 0.35,
            light: 0.72,
            fan: 0.,
        }
    }
}
impl World {
    pub fn find_anchor(&self, seed: &Seed) -> Option<(usize, V)> {
        let mut best = None;
        let mut score = f32::MAX;
        for (index, h) in HABITATS.iter().enumerate() {
            // Search actual frame perimeter. Clearance reserves the fully grown wing envelope.
            for n in 0..120 {
                let t = n as f32 / 120.;
                let a = anchor_at(*h, t);
                let radius = 22. * seed.size() + 3.;
                if self.seeds.iter().any(|s| {
                    s.id != seed.id
                        && s.habitat == Some(index)
                        && s.phase != Phase::Dead
                        && s.anchor.distance(a) < radius + 22. * s.size() + 3.
                }) {
                    continue;
                }
                let candidate = seed.p.distance(a);
                if candidate < score {
                    score = candidate;
                    best = Some((index, a));
                }
            }
        }
        best
    }
    pub fn step(&mut self, dt: f32) {
        self.time += dt;
        self.fan += dt * self.wind * 9.;
        let mut births = Vec::new();
        for s in &mut self.seeds {
            if s.update(dt, self.light, self.wind, self.time) {
                for i in 0..2 {
                    let mut child =
                        Seed::new(self.next_id, s.generation + 1, s.size() * 0.78, s.crack);
                    self.next_id += 1;
                    child.phase = Phase::Drifting;
                    child.p = V::new(s.p.x + (i as f32 - 0.5) * 8., s.p.y, s.p.z);
                    child.start = child.p;
                    child.hooks = [1., 0., 0.];
                    births.push(child);
                }
                s.offspring += 2;
            }
        }
        self.seeds.extend(births);
        for i in 0..self.seeds.len() {
            if self.seeds[i].phase == Phase::Drifting
                && self.seeds[i].habitat.is_none()
                && self.seeds[i].timer < 0.1
            {
                if let Some((h, a)) = self.find_anchor(&self.seeds[i]) {
                    self.seeds[i].habitat = Some(h);
                    self.seeds[i].anchor = a;
                    self.seeds[i].crack = CRACKS[h];
                }
            }
        }
        // Departed seeds no longer occupy a frame or consume update/render work.
        if self.seeds.len() > 512 {
            self.seeds.retain(|s| s.phase != Phase::Dead || s.id == 1);
        }
    }
}
pub fn anchor_at(h: Habitat, t: f32) -> V {
    if h.kind == 0 {
        return V::new(
            h.p.x + (t * TAU).cos() * 76.,
            h.p.y + (t * TAU).sin() * 76.,
            h.p.z + 14.,
        );
    }
    let (w, height, z) = if h.kind == 1 {
        (74., 50., h.p.z + 33.)
    } else {
        (56., 21., h.p.z + 65.)
    };
    let p = t * 4.;
    let (x, y) = if p < 1. {
        (mix(-w, w, p), -height)
    } else if p < 2. {
        (w, mix(-height, height, p - 1.))
    } else if p < 3. {
        (mix(w, -w, p - 2.), height)
    } else {
        (-w, mix(height, -height, p - 3.))
    };
    V::new(h.p.x + x, h.p.y + y, z)
}
pub fn preview(cycle: u32, stage: u32, day: u32) -> Seed {
    let mut s = Seed::new(999, 0, 1., CRACKS[0]);
    s.cycle = cycle.clamp(1, 7);
    s.habitat = Some(0);
    s.anchor = anchor_at(HABITATS[0], 0.62);
    s.phase = Phase::Adult;
    s.maturity = 1.;
    s.nights = 3;
    s.wings = 1.;
    s.hooks = [1.; 3];
    s.days = day.clamp(1, 7) - 1;
    s.p = s.floating_position(0.5, 0.);
    if stage < 8 {
        s.days = 0;
        s.nights = 0;
    }
    match stage {
        0 => {
            s.phase = Phase::Soil;
            s.p = s.crack;
            s.habitat = None;
            s.hooks = [0.; 3];
            s.wings = 0.;
        }
        1 => {
            s.phase = Phase::Emerging;
            s.p = V::new(s.crack.x, -4., s.crack.z);
            s.habitat = None;
            s.hooks = [0.; 3];
            s.wings = 0.;
        }
        2 => {
            s.phase = Phase::Hook;
            s.p = V::new(s.crack.x, -4., s.crack.z);
            s.habitat = None;
            s.hooks = [1., 0., 0.];
            s.wings = 0.;
        }
        3 => {
            s.phase = Phase::Drifting;
            s.p = V::new(-275., -65., 210.);
            s.hooks = [1., 0., 0.];
            s.wings = 0.;
        }
        4 => {
            s.phase = Phase::Attaching;
            s.hooks = [1., 0., 0.];
            s.wings = 0.;
        }
        5..=7 => {
            s.phase = Phase::Juvenile;
            s.nights = stage - 5;
            s.maturity = (stage - 4) as f32 / 3. - 0.05;
            s.wings = ((s.maturity - 1. / 3.) * 3.).clamp(0., 1.);
            s.hooks = [1., (s.maturity * 3.).min(1.), (s.maturity * 3.).min(1.)];
        }
        8 => s.growth = 0.28,
        9 => s.growth = 0.55,
        10 => s.growth = 1.,
        11 => {
            s.growth = 0.40;
            s.wilt = 0.85;
        }
        12 => {
            s.phase = Phase::Reproduce;
            s.days = 7;
            s.growth = 0.8;
            s.hooks = [1., 0., 0.];
        }
        13 => {
            s.phase = Phase::Withering;
            s.days = 7;
            s.growth = 0.2;
            s.wilt = 1.;
            s.wings = 0.25;
        }
        14 => {
            s.phase = Phase::Falling;
            s.days = 7;
            s.p = s.p.mix(s.crack, 0.65);
            s.wilt = 1.;
            s.wings = 0.1;
            s.hooks = [0.; 3];
        }
        _ => {
            s.phase = if cycle >= 7 { Phase::Dead } else { Phase::Soil };
            if cycle < 7 {
                s.cycle += 1;
            }
            s.days = 0;
            s.nights = 0;
            s.p = s.crack;
            s.wings = 0.;
            s.hooks = [0.; 3];
            s.habitat = None;
        }
    }
    s
}

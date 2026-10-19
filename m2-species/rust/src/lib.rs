mod model;
mod render;
use model::*;
use render::{camera, Draw};
use std::cell::RefCell;
use std::fmt::Write;
struct App {
    world: World,
    draw: Draw,
    json: String,
    yaw: f32,
    pitch: f32,
    demo: Option<Seed>,
    demo_time: f32,
    selected: u32,
    eye: V,
}
impl Default for App {
    fn default() -> Self {
        Self {
            world: World::default(),
            draw: Draw::new(),
            json: String::new(),
            yaw: -0.32,
            pitch: -0.40,
            demo: None,
            demo_time: 0.,
            selected: 1,
            eye: V::default(),
        }
    }
}
thread_local! {static APP:RefCell<App>=RefCell::new(App::default());}

#[no_mangle]
pub extern "C" fn reset() {
    APP.with(|a| *a.borrow_mut() = App::default());
}
#[no_mangle]
pub extern "C" fn orbit(dx: f32, dy: f32) {
    APP.with(|a| {
        let mut a = a.borrow_mut();
        a.yaw -= dx * 0.008;
        a.pitch = (a.pitch + dy * 0.006).clamp(-1.15, -0.10);
    });
}
#[no_mangle]
pub extern "C" fn set_preview(cycle: u32, stage: i32, day: u32) {
    APP.with(|a| {
        let mut a = a.borrow_mut();
        a.demo = if stage < 0 {
            None
        } else {
            Some(model::preview(cycle, stage as u32, day))
        };
        a.demo_time = 0.;
    });
}
#[no_mangle]
pub extern "C" fn frame(
    dt: f32,
    width: f32,
    height: f32,
    wind: f32,
    light: f32,
    auto_wind: u32,
    auto_sun: u32,
    selected: u32,
    focus: u32,
) {
    APP.with(|cell| {
        let mut app = cell.borrow_mut();
        let a = &mut *app;
        let dt = dt.clamp(0., 0.05);
        a.selected = selected;
        if a.demo.is_none() {
            let t = a.world.time;
            let wind = if auto_wind != 0 {
                0.48 + (t * 0.3).sin() * 0.4
            } else {
                wind.clamp(0., 1.)
            };
            let light = if auto_sun != 0 {
                0.5 + 0.5 * (t * std::f32::consts::TAU / 40.).cos()
            } else {
                light.clamp(0., 1.)
            };
            a.world.wind = mix(a.world.wind, wind, 1. - (-dt * 3.).exp());
            a.world.light = mix(a.world.light, light, 1. - (-dt * 3.).exp());
            a.world.step(dt);
        } else {
            a.demo_time += dt;
        }
        let seed = a.demo.as_ref().unwrap_or_else(|| {
            a.world
                .seeds
                .iter()
                .find(|s| s.id == selected)
                .unwrap_or(&a.world.seeds[0])
        });
        let target = if focus != 0 {
            V::new(seed.p.x, seed.p.y.min(-5.), seed.p.z)
        } else {
            V::new(0., -70., -25.)
        };
        let aspect = (width / height.max(1.)).max(0.1);
        let distance = if focus != 0 {
            (72. * seed.body_size()).max(26.) * (1. / aspect).max(1.)
        } else {
            970_f32.max(1100. / aspect)
        };
        a.eye = camera(target, a.yaw, a.pitch, distance);
        let l = a.world.light;
        a.draw.data.clear();
        a.draw.cmd(
            30,
            &[mix(31., 204., l), mix(42., 225., l), mix(53., 228., l)],
        );
        a.draw.cmd(
            31,
            &[
                std::f32::consts::PI / 3.1,
                aspect,
                if focus != 0 { 1. } else { 3. },
                12000.,
            ],
        );
        a.draw.cmd(
            32,
            &[
                a.eye.x, a.eye.y, a.eye.z, target.x, target.y, target.z, 0., 1., 0.,
            ],
        );
        a.draw.cmd(
            33,
            &[mix(80., 150., l), mix(92., 151., l), mix(110., 151., l)],
        );
        a.draw.cmd(
            34,
            &[
                mix(95., 240., l),
                mix(112., 225., l),
                mix(140., 200., l),
                -0.5,
                0.8,
                -0.4,
            ],
        );
        a.draw.scene(&a.world);
        if let Some(s) = &a.demo {
            a.draw.seed(s, 0.5, a.demo_time);
            if s.phase == Phase::Reproduce {
                for i in 0..2 {
                    let mut child = Seed::new(1000 + i, 1, s.size() * 0.78, s.crack);
                    child.phase = Phase::Drifting;
                    child.hooks = [1., 0., 0.];
                    child.p = V::new(
                        s.p.x + 12. + i as f32 * 16.,
                        s.p.y - 12. - i as f32 * 8.,
                        s.p.z + 10.,
                    );
                    a.draw.seed(&child, 0.5, a.demo_time);
                }
            }
        } else {
            for s in &a.world.seeds {
                a.draw.seed(s, a.world.wind, a.world.time);
            }
        }
        a.snapshot();
    });
}
impl App {
    fn snapshot(&mut self) {
        self.json.clear();
        let _=write!(self.json,"{{\"rust\":true,\"preview\":{},\"time\":{},\"wind\":{},\"sunlight\":{},\"eyeY\":{},\"pitch\":{},\"population\":{},\"seeds\":[",
            self.demo.is_some(),self.world.time,self.world.wind,self.world.light,self.eye.y,self.pitch,
            self.world.seeds.iter().filter(|s|s.phase!=Phase::Dead).count());
        for (i, s) in self.world.seeds.iter().enumerate() {
            if i > 0 {
                self.json.push(',');
            }
            seed_json(&mut self.json, s);
        }
        self.json.push_str("],\"observed\":");
        let s = self.demo.as_ref().unwrap_or_else(|| {
            self.world
                .seeds
                .iter()
                .find(|s| s.id == self.selected)
                .unwrap_or(&self.world.seeds[0])
        });
        seed_json(&mut self.json, s);
        self.json.push('}');
    }
}
fn seed_json(json: &mut String, s: &Seed) {
    let habitat = s.habitat.map(|i| i as i32).unwrap_or(-1);
    let _=write!(json,"{{\"id\":{},\"phase\":\"{}\",\"cycle\":{},\"days\":{},\"nights\":{},\"size\":{},\"growth\":{},\"wilt\":{},\"wings\":{},\"generation\":{},\"habitat\":{},\"offspring\":{},\"x\":{},\"y\":{},\"z\":{}}}",
        s.id,s.phase.name(),s.cycle,s.days,s.nights,s.size(),s.growth,s.wilt,s.wings,s.generation,habitat,s.offspring,s.p.x,s.p.y,s.p.z);
}
#[no_mangle]
pub extern "C" fn draw_ptr() -> *const f32 {
    APP.with(|a| a.borrow().draw.data.as_ptr())
}
#[no_mangle]
pub extern "C" fn draw_len() -> usize {
    APP.with(|a| a.borrow().draw.data.len())
}
#[no_mangle]
pub extern "C" fn state_ptr() -> *const u8 {
    APP.with(|a| a.borrow().json.as_ptr())
}
#[no_mangle]
pub extern "C" fn state_len() -> usize {
    APP.with(|a| a.borrow().json.len())
}
#[no_mangle]
pub extern "C" fn test_step(seconds: f32, light: f32, wind: f32) {
    APP.with(|a| {
        let mut a = a.borrow_mut();
        a.world.light = light;
        a.world.wind = wind;
        for _ in 0..(seconds.max(0.).min(1000.) * 20.) as u32 {
            a.world.step(0.05);
        }
        a.snapshot();
    });
}

fn advance(seed: &mut Seed, seconds: f32, light: f32, wind: f32) -> u32 {
    let mut releases = 0;
    for i in 0..(seconds * 20.) as u32 {
        if seed.update(0.05, light, wind, i as f32 * 0.05) {
            releases += 1;
        }
    }
    releases
}
/// Executed against the compiled WASM by tests/check-wasm.cjs.
#[no_mangle]
pub extern "C" fn check_model() -> u32 {
    let mut s = Seed::new(1, 0, 1., CRACKS[0]);
    advance(&mut s, 4., 1., 0.);
    assert_eq!(s.phase, Phase::Emerging);
    advance(&mut s, 6., 1., 0.);
    advance(&mut s, 5., 1., 0.);
    assert_eq!(s.phase, Phase::Hook);
    advance(&mut s, 0.1, 1., 0.8);
    assert_eq!(s.phase, Phase::Drifting);
    let mut total = 0;
    for cycle in 1..=7 {
        s.cycle = cycle;
        s.habitat = Some(0);
        s.anchor = anchor_at(HABITATS[0], 0.6);
        s.phase = Phase::Attaching;
        s.timer = 0.;
        advance(&mut s, 2.1, 1., 0.8);
        assert_eq!(s.phase, Phase::Juvenile);
        for night in 1..=3 {
            advance(&mut s, 8., 0., 0.8);
            assert_eq!(s.nights, night - 1);
            assert_eq!(s.growth, 0.);
            assert_eq!(s.wilt, 0.);
            advance(&mut s, 3.1, 1., 0.8);
        }
        assert_eq!(s.phase, Phase::Adult);
        for day in 1..=7 {
            total += advance(&mut s, 8., 1., 0.8);
            total += advance(&mut s, 8., 0., 0.8);
            assert_eq!(s.days, day - 1);
            assert_eq!(total, (cycle - 1) as u32, "No early reproduction");
            s.update(0.05, 1., 0.8, 0.);
            assert_eq!(s.days, day);
        }
        assert_eq!(s.phase, Phase::Reproduce);
        total += advance(&mut s, 5.2, 1., 0.8);
        assert_eq!(total, cycle as u32);
        advance(&mut s, 5.2, 0., 0.8);
        assert_eq!(s.phase, Phase::Falling);
        advance(&mut s, 4.2, 0., 0.8);
        if cycle < 7 {
            assert_eq!(s.phase, Phase::Soil);
            assert_eq!(s.cycle, cycle + 1);
        } else {
            assert_eq!(s.phase, Phase::Dead);
        }
    }
    // Capacity derives from body clearance, rather than a two-per-outlet limit.
    let mut w = World::default();
    w.seeds.clear();
    for id in 1..=12 {
        let mut seed = Seed::new(id, 1, 0.65, CRACKS[0]);
        seed.p = HABITATS[0].p;
        let (h, a) = w.find_anchor(&seed).expect("available perimeter");
        seed.habitat = Some(h);
        seed.anchor = a;
        seed.phase = Phase::Juvenile;
        w.seeds.push(seed);
    }
    assert!(w.seeds.iter().filter(|s| s.habitat == Some(0)).count() > 2);
    for a in &w.seeds {
        for b in &w.seeds {
            if a.id != b.id && a.habitat == b.habitat {
                assert!(a.anchor.distance(b.anchor) >= 22. * (a.size() + b.size()) + 6.);
            }
        }
    }
    for cycle in 1..=7 {
        for stage in 0..16 {
            let p = model::preview(cycle, stage, 7);
            for yaw in [-3., -1., 0., 1., 3.] {
                for pitch in [-2., 0., 2.] {
                    let eye = camera(p.p, yaw, pitch, 125.);
                    assert!(eye.y <= -18.);
                }
            }
        }
    }
    7
}

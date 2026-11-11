use crate::model::*;
use std::f32::consts::{PI, TAU};
pub struct Draw {
    pub data: Vec<f32>,
}
impl Draw {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(60000),
        }
    }
    pub fn cmd(&mut self, op: u32, args: &[f32]) {
        self.data.push(op as f32);
        self.data.push(args.len() as f32);
        self.data.extend_from_slice(args);
    }
    fn push(&mut self) {
        self.cmd(0, &[]);
    }
    fn pop(&mut self) {
        self.cmd(1, &[]);
    }
    fn at(&mut self, p: V) {
        self.cmd(2, &[p.x, p.y, p.z]);
    }
    fn rx(&mut self, a: f32) {
        self.cmd(3, &[a]);
    }
    fn rz(&mut self, a: f32) {
        self.cmd(5, &[a]);
    }
    fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.cmd(6, &[x, y, z]);
    }
    fn material(&mut self, c: [f32; 3]) {
        self.cmd(7, &c);
        self.cmd(15, &[]);
    }
    fn line(&mut self, a: V, b: V, color: [f32; 3], weight: f32) {
        self.cmd(12, &[color[0], color[1], color[2], 220.]);
        self.cmd(13, &[weight]);
        self.cmd(14, &[a.x, a.y, a.z, b.x, b.y, b.z]);
    }
    fn box_at(&mut self, p: V, size: V, c: [f32; 3]) {
        self.push();
        self.at(p);
        self.material(c);
        self.cmd(8, &[size.x, size.y, size.z]);
        self.pop();
    }
    fn ball(&mut self, p: V, r: f32, shape: V, c: [f32; 3]) {
        self.push();
        self.at(p);
        self.scale(shape.x, shape.y, shape.z);
        self.material(c);
        self.cmd(9, &[r, 18., 12.]);
        self.pop();
    }
    fn rod(&mut self, a: V, b: V, r: f32, c: [f32; 3]) {
        let d = V::new(b.x - a.x, b.y - a.y, b.z - a.z);
        let len = a.distance(b);
        if len < 0.001 {
            return;
        }
        self.push();
        self.at(a.mix(b, 0.5));
        let axis = if d.x.abs() + d.z.abs() < 0.001 {
            V::new(1., 0., 0.)
        } else {
            V::new(d.z, 0., -d.x)
        };
        self.cmd(
            20,
            &[(d.y / len).clamp(-1., 1.).acos(), axis.x, axis.y, axis.z],
        );
        self.material(c);
        self.cmd(10, &[r, len, 8., 1.]);
        self.pop();
    }
    fn torus(&mut self, r: f32, t: f32, c: [f32; 3]) {
        self.material(c);
        self.cmd(11, &[r, t, 40., 8.]);
    }
    pub fn scene(&mut self, w: &World) {
        let l = w.light;
        for i in 0..10 {
            let x = (i as f32 - 4.5) * 1000.;
            let tall = 1800. + (i * 317 % 1600) as f32;
            let z = -5200. - (i % 3) as f32 * 600.;
            let c = [mix(45., 151., l), mix(57., 175., l), mix(68., 183., l)];
            self.box_at(V::new(x, 1800. - tall / 2., z), V::new(700., tall, 550.), c);
            for row in 0..(tall / 300.) as i32 - 1 {
                for col in -1..=1 {
                    self.box_at(
                        V::new(x + col as f32 * 160., 1600. - row as f32 * 300., z + 286.),
                        V::new(80., 130., 4.),
                        [c[0] * 0.8, c[1] * 0.84, c[2] * 0.88],
                    );
                }
            }
        }
        self.box_at(
            V::new(0., 12., 0.),
            V::new(800., 24., 650.),
            [153., 163., 162.],
        );
        self.box_at(
            V::new(0., -27., -318.),
            V::new(800., 54., 14.),
            [186., 195., 192.],
        );
        for x in [-393., 393.] {
            self.box_at(
                V::new(x, -27., 0.),
                V::new(14., 54., 650.),
                [186., 195., 192.],
            );
        }
        for x in (-300..=300).step_by(100) {
            self.line(
                V::new(x as f32, -0.5, -310.),
                V::new(x as f32, -0.5, 320.),
                [116., 133., 136.],
                0.65,
            );
        }
        for z in (-220..=280).step_by(100) {
            self.line(
                V::new(-380., -0.5, z as f32),
                V::new(380., -0.5, z as f32),
                [116., 133., 136.],
                0.65,
            );
        }
        for (i, c) in CRACKS.iter().enumerate() {
            for j in -4..4 {
                let a = V::new(c.x + j as f32 * 9., -0.8, c.z + (j as f32 * 2.7).sin() * 4.);
                let b = V::new(
                    c.x + (j + 1) as f32 * 9.,
                    -0.8,
                    c.z + ((j + 1) as f32 * 2.7).sin() * 4.,
                );
                self.line(a, b, [66., 76., 72.], 2.);
                if j % 2 == 0 {
                    self.line(a, V::new(a.x + 3., -0.9, a.z + 12.), [85., 94., 86.], 1.);
                    self.rod(
                        V::new(a.x, -1., a.z),
                        V::new(a.x + 1., -4. - (i % 2) as f32, a.z),
                        0.15,
                        [83., 120., 88.],
                    );
                }
            }
        }
        self.panel(-275., -220.);
        self.panel(-275., -90.);
        for h in HABITATS {
            self.line(
                V::new(h.p.x, -2., h.p.z - 25.),
                V::new(h.p.x, -2., -285.),
                [77., 91., 93.],
                2.,
            );
            self.line(
                V::new(h.p.x, -2., -285.),
                V::new(-275., -2., -285.),
                [77., 91., 93.],
                2.,
            );
            self.habitat(h, w.fan);
        }
        if w.wind > 0.08 {
            for h in HABITATS {
                for i in 0..7 {
                    let z = h.p.z + 40. + ((w.time * (20. + w.wind * 80.) + i as f32 * 27.) % 160.);
                    let x = h.p.x + (i as f32 * 4.7).sin() * 50.;
                    let y = h.p.y + (i as f32 * 2.3).cos() * 30.;
                    self.line(
                        V::new(x, y, z),
                        V::new(x, y, z + 8. + w.wind * 12.),
                        [173., 217., 215.],
                        0.5,
                    );
                }
            }
        }
    }
    fn panel(&mut self, x: f32, z: f32) {
        for sx in [-38., 38.] {
            for sz in [-60., 60.] {
                let y = -53. - 0.32_f32.sin() * sz;
                self.box_at(
                    V::new(x + sx, y / 2., z + sz * 0.32_f32.cos()),
                    V::new(4., -y, 4.),
                    [151., 177., 175.],
                );
            }
        }
        self.push();
        self.at(V::new(x, -53., z));
        self.rx(0.32);
        self.box_at(V::default(), V::new(104., 5., 164.), [151., 177., 175.]);
        self.box_at(V::new(0., -3., 0.), V::new(98., 2., 158.), [48., 83., 111.]);
        for a in (-32..=32).step_by(16) {
            self.line(
                V::new(a as f32, -4., -78.),
                V::new(a as f32, -4., 78.),
                [117., 155., 176.],
                0.7,
            );
        }
        for b in (-60..=60).step_by(20) {
            self.line(
                V::new(-48., -4., b as f32),
                V::new(48., -4., b as f32),
                [117., 155., 176.],
                0.7,
            );
        }
        self.pop();
    }
    fn habitat(&mut self, h: Habitat, angle: f32) {
        let x = h.p.x;
        let z = h.p.z;
        let metal = [61., 113., 113.];
        let steel = [151., 177., 175.];
        let dark = [43., 64., 68.];
        self.box_at(
            V::new(x, -10., z - 15.),
            V::new(180., 20., 100.),
            [186., 195., 192.],
        );
        if h.kind == 0 {
            self.box_at(
                V::new(x, h.p.y + 9., z - 25.),
                V::new(164., 174., 54.),
                metal,
            );
            self.push();
            self.at(V::new(x, h.p.y, z + 4.));
            self.push();
            self.rx(PI / 2.);
            self.material(dark);
            self.cmd(10, &[74., 12., 32., 1.]);
            self.pop();
            self.torus(76., 5., steel);
            self.push();
            self.rz(angle);
            for i in 0..7 {
                self.push();
                self.rz(i as f32 * TAU / 7.);
                self.at(V::new(34., 0., 10.));
                self.rz(-0.4);
                self.box_at(V::default(), V::new(57., 18., 3.), [115., 151., 149.]);
                self.pop();
            }
            self.ball(V::default(), 14., V::new(1., 1., 1.), dark);
            self.pop();
            self.at(V::new(0., 0., 15.));
            for r in [30., 52., 69.] {
                self.torus(r, 0.6, steel);
            }
            self.pop();
        } else if h.kind == 1 {
            self.box_at(V::new(x, -65., z - 12.), V::new(156., 130., 64.), metal);
            self.box_at(V::new(x, -72., z + 22.), V::new(138., 104., 3.), dark);
            for row in 0..6 {
                self.push();
                self.at(V::new(x, -116. + row as f32 * 18., z + 27.));
                self.rx(-0.3);
                self.box_at(V::default(), V::new(140., 5., 18.), steel);
                self.pop();
            }
        } else {
            self.box_at(V::new(x, -58., z - 28.), V::new(100., 116., 92.), steel);
            self.box_at(V::new(x, -118., z - 6.), V::new(120., 56., 126.), metal);
            self.box_at(V::new(x, -117., z + 59.), V::new(103., 40., 3.), dark);
            for col in (-40..=40).step_by(16) {
                self.box_at(
                    V::new(x + col as f32, -117., z + 62.),
                    V::new(3., 38., 3.),
                    steel,
                );
            }
        }
        self.box_at(
            V::new(x + 58., -24., z + 28.),
            V::new(18., 8., 2.),
            [222., 177., 104.],
        );
    }
    pub fn seed(&mut self, s: &Seed, wind: f32, time: f32) {
        if s.phase == Phase::Dead {
            return;
        }
        let size = s.body_size();
        let attached = matches!(
            s.phase,
            Phase::Attaching | Phase::Juvenile | Phase::Adult | Phase::Reproduce | Phase::Withering
        );
        for i in 0..3 {
            let length = s.hooks[i];
            if length < 0.01 {
                continue;
            }
            let a = V::new(s.p.x + (i as f32 - 1.) * 1.2 * size, s.p.y + size, s.p.z);
            let dest = if attached {
                V::new(s.anchor.x + (i as f32 - 1.) * 2., s.anchor.y, s.anchor.z)
            } else {
                V::new(a.x + 6. * size, a.y - 10. * size, a.z + 2. * size)
            };
            let end = a.mix(dest, length);
            let mut prev = a;
            for k in 1..=14 {
                let t = k as f32 / 14.;
                let mut p = a.mix(end, t);
                p.y += (t * PI).sin() * if attached { 8. } else { -3. * size };
                self.line(prev, p, [86., 123., 101.], 0.6);
                prev = p;
            }
            self.line(
                end,
                V::new(end.x + 1.5 * size, end.y - 1.5 * size, end.z),
                [86., 123., 101.],
                0.6,
            );
        }
        self.push();
        self.at(s.p);
        let burial = if s.phase == Phase::Soil { 0.35 } else { 1. };
        self.rz(
            if matches!(s.phase, Phase::Soil | Phase::Emerging | Phase::Hook) {
                -0.15
            } else {
                s.spin
            },
        );
        self.scale(6. * size * burial, 6. * size * burial, 6. * size * burial);
        let body = [
            mix(95., 113., s.wilt),
            mix(157., 107., s.wilt),
            mix(113., 79., s.wilt),
        ];
        self.ball(V::default(), 1., V::new(1., 0.52, 0.72), body);
        if s.phase == Phase::Drifting {
            for i in 0..9 {
                let a = i as f32 * TAU / 9.;
                self.line(
                    V::new(0., -0.4, 0.),
                    V::new(a.cos() * 1.4, -1.1 + a.sin() * 0.45, -0.2),
                    [209., 224., 198.],
                    0.45,
                );
            }
        }
        // Model-space radii keep stem, leaves and hooks proportional at any zoom.
        for ridge in -2..=2 {
            let z = ridge as f32 * 0.18;
            let mut previous = V::new(-0.88, 0., z);
            for j in 1..=12 {
                let t = j as f32 / 12.;
                let p = V::new(mix(-0.88, 0.88, t), -0.46 * (t * PI).sin(), z);
                self.line(previous, p, [168., 197., 145.], 0.45);
                previous = p;
            }
        }
        if s.wings > 0.01 {
            for i in 0..3 {
                self.push();
                self.rz(i as f32 * TAU / 3.);
                self.at(V::new(0., 0., (i as f32 - 1.) * 0.25));
                let open = smooth(wind);
                self.rx(mix(1.0, 0.10, open));
                self.scale(s.wings, s.wings, s.wings);
                self.cmd(15, &[]);
                self.cmd(16, &[163., 209., 181., 145.]);
                self.cmd(17, &[]);
                for j in 0..=14 {
                    let u = j as f32 / 14.;
                    let reach = mix(1.2, 2.8, open);
                    let x = mix(0.65, reach, u);
                    let breadth = (u * PI).sin() * mix(0.10, 0.65, open);
                    let z =
                        (u * PI).sin() * 0.18 + (time * 2. + u * 3. + i as f32).sin() * wind * 0.02;
                    self.cmd(18, &[x, -breadth, z]);
                    self.cmd(18, &[x, breadth, z]);
                }
                self.cmd(19, &[]);
                self.line(
                    V::new(0.65, 0., 0.),
                    V::new(mix(1.2, 2.8, open), 0., 0.),
                    [192., 225., 197.],
                    0.55,
                );
                self.pop();
            }
        }
        if s.growth > 0.02 {
            self.push();
            self.at(V::new(0., -0.43, 0.32));
            self.rz(s.wilt * 0.75);
            let stem = 1.3 * smooth(s.growth / 0.3);
            self.rod(
                V::default(),
                V::new(0.06, -stem, 0.),
                0.022,
                [86., 123., 72.],
            );
            let leaves = smooth((s.growth - 0.12) / 0.25);
            if leaves > 0.01 {
                for side in [-1., 1.] {
                    self.push();
                    self.at(V::new(side * 0.18 * leaves, -stem * 0.6, 0.));
                    self.rz(side * 0.6);
                    self.ball(
                        V::default(),
                        0.25 * leaves,
                        V::new(1., 0.42, 0.14),
                        [103., 156., 96.],
                    );
                    self.pop();
                }
            }
            let bud = smooth((s.growth - 0.35) / 0.25);
            if bud > 0.01 {
                self.ball(
                    V::new(0.06, -stem, 0.),
                    0.13 * bud,
                    V::new(1., 1.3, 0.8),
                    [191., 131., 123.],
                );
                let flower = smooth((s.growth - 0.62) / 0.38);
                for i in 0..5 {
                    if flower < 0.01 {
                        break;
                    }
                    self.push();
                    self.at(V::new(0.06, -stem, 0.));
                    self.rz(i as f32 * TAU / 5.);
                    self.ball(
                        V::new(0., -0.20 * flower, 0.),
                        0.17 * flower,
                        V::new(0.65, 1., 0.28),
                        [
                            mix(225., 112., s.wilt),
                            mix(148., 99., s.wilt),
                            mix(144., 78., s.wilt),
                        ],
                    );
                    self.pop();
                }
                self.ball(
                    V::new(0.06, -stem, 0.035),
                    0.07,
                    V::new(1., 1., 0.7),
                    [224., 189., 96.],
                );
            }
            self.pop();
        }
        self.pop();
    }
}

pub fn camera(target: V, yaw: f32, pitch: f32, distance: f32) -> V {
    let pitch = pitch.clamp(-1.15, -0.10);
    let mut eye = V::new(
        target.x + yaw.sin() * pitch.cos() * distance,
        (target.y + pitch.sin() * distance).min(-18.),
        target.z + yaw.cos() * pitch.cos() * distance,
    );
    // Camera stays above the roof and outside equipment solids, including close-ups.
    for _ in 0..6 {
        let mut changed = false;
        for h in HABITATS {
            let top = if h.kind == 0 {
                h.p.y - 85.
            } else if h.kind == 1 {
                -135.
            } else {
                -150.
            };
            if (eye.x - h.p.x).abs() < 102.
                && eye.z > h.p.z - 88.
                && eye.z < h.p.z + 82.
                && eye.y > top - 14.
            {
                eye.y = top - 14.;
                changed = true;
            }
        }
        if eye.x.abs() > 377. && eye.x.abs() < 420. && eye.z.abs() < 340. && eye.y > -70. {
            eye.y = -70.;
            changed = true;
        }
        if eye.z < -298. && eye.z > -340. && eye.x.abs() < 420. && eye.y > -70. {
            eye.y = -70.;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    eye
}

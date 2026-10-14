//! Scripture-led Tabernacle scene and exploration pages.
//!
//! The drawing uses the Anatomy kit's isometric camera equations and the same
//! painter's-order idea for extruded faces, implemented here in Rust so the
//! exact scene renders in both Axum and the WebAssembly reader.

use super::{esc, parse_ref, ref_label, reference_url, route_path, verses, Ref, RouteResponse};
use std::f64::consts::PI;

const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 440.0;
const PARTS: [Part; 8] = [
    Part {
        id: "courtyard",
        name: "Courtyard",
        aria: "Explore the courtyard",
        references: &["Exodus 27:13-18"],
        depth: -120.0,
        note: "Exodus specifies the court's linen hangings, east side, and embroidered gate curtain (Exodus 27:13–18).",
    },
    Part {
        id: "altar",
        name: "Altar",
        aria: "Explore the altar",
        references: &["Exodus 27:1-2", "Exodus 40:6"],
        depth: 43.0,
        note: "The square shittim-wood altar had horns and a brass overlay, and stood before the tabernacle door (Exodus 27:1–2; 40:6).",
    },
    Part {
        id: "basin",
        name: "Laver",
        aria: "Explore the laver",
        references: &["Exodus 30:18-21"],
        depth: 28.0,
        note: "The brass laver and its foot were placed between the tabernacle and the altar for washing.",
    },
    Part {
        id: "tent",
        name: "Tabernacle tent",
        aria: "Explore the tabernacle tent",
        references: &["Exodus 26:1", "Exodus 26:7"],
        depth: 39.0,
        note: "The curtains used fine twined linen with blue, purple, and scarlet, with a goats’ hair covering also specified (Exodus 26:1, 7).",
    },
    Part {
        id: "table",
        name: "Table of shewbread",
        aria: "Explore the table of shewbread",
        references: &["Exodus 25:23-24", "Exodus 25:30", "Exodus 26:35"],
        depth: -3.0,
        note: "The gold-overlaid shittim table held shewbread and stood on the north side of the tabernacle (Exodus 25:23–24, 30; 26:35).",
    },
    Part {
        id: "lamp",
        name: "Candlestick",
        aria: "Explore the candlestick",
        references: &["Exodus 25:31-32", "Exodus 25:37", "Exodus 26:35"],
        depth: 4.0,
        note: "The pure-gold candlestick had six branches and seven lamps, and stood opposite the table on the south side (Exodus 25:31–32, 37; 26:35).",
    },
    Part {
        id: "veil",
        name: "Veil",
        aria: "Explore the veil",
        references: &["Exodus 26:31-33"],
        depth: -17.0,
        note: "The veil used blue, purple, scarlet, and fine twined linen, and divided the holy place from the most holy.",
    },
    Part {
        id: "ark",
        name: "Ark of the testimony",
        aria: "Explore the ark of the testimony",
        references: &[
            "Exodus 25:10-11",
            "Exodus 25:16",
            "Exodus 25:21-22",
            "Exodus 26:33",
        ],
        depth: -24.0,
        note: "The gold-overlaid shittim ark held the testimony and was placed within the veil (Exodus 25:10–11, 16, 21–22; 26:33).",
    },
];

struct Part {
    id: &'static str,
    name: &'static str,
    aria: &'static str,
    references: &'static [&'static str],
    depth: f64,
    note: &'static str,
}

#[derive(Clone, Copy)]
struct Point3(f64, f64, f64);

#[derive(Clone, Copy)]
struct Point2(f64, f64);

struct Projection {
    k: f64,
    origin: Point2,
    sin_a: f64,
    cos_a: f64,
    sin_e: f64,
    cos_e: f64,
}

impl Projection {
    fn fitted() -> Self {
        let azimuth = 45.0_f64.to_radians();
        let elevation = 30.0_f64.to_radians();
        let (sin_a, cos_a) = azimuth.sin_cos();
        let (sin_e, cos_e) = elevation.sin_cos();
        let corners = [
            Point3(-105.0, -70.0, -2.0),
            Point3(-105.0, 70.0, -2.0),
            Point3(105.0, -70.0, -2.0),
            Point3(105.0, 70.0, -2.0),
            Point3(-105.0, -70.0, 48.0),
            Point3(-105.0, 70.0, 48.0),
            Point3(105.0, -70.0, 48.0),
            Point3(105.0, 70.0, 48.0),
        ];
        let raw = corners.map(|p| {
            Point2(
                p.0 * sin_a - p.1 * cos_a,
                (p.0 * cos_a + p.1 * sin_a) * sin_e - p.2 * cos_e,
            )
        });
        let min_x = raw.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let max_x = raw.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let min_y = raw.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let max_y = raw.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let k = ((WIDTH - 48.0) / (max_x - min_x)).min((HEIGHT - 48.0) / (max_y - min_y));
        let origin = Point2(
            WIDTH / 2.0 - (min_x + max_x) * k / 2.0,
            HEIGHT / 2.0 - (min_y + max_y) * k / 2.0,
        );
        Self {
            k,
            origin,
            sin_a,
            cos_a,
            sin_e,
            cos_e,
        }
    }

    // Anatomy kit iso(): x'=(x sin A-y cos A)k; y'=((x cos A+y sin A)sin E-z cos E)k.
    fn project(&self, p: Point3) -> Point2 {
        Point2(
            self.origin.0 + (p.0 * self.sin_a - p.1 * self.cos_a) * self.k,
            self.origin.1
                + ((p.0 * self.cos_a + p.1 * self.sin_a) * self.sin_e - p.2 * self.cos_e) * self.k,
        )
    }

    // Anatomy kit depthOf(): camera-facing surfaces are drawn after farther surfaces.
    fn depth(&self, p: Point3) -> f64 {
        (p.0 * self.cos_a + p.1 * self.sin_a) * self.cos_e + p.2 * self.sin_e
    }
}

struct Face {
    points: Vec<Point3>,
    class: &'static str,
    material: &'static str,
    depth: f64,
    opacity: Option<f64>,
}

#[derive(Default)]
struct SceneBuilder {
    faces: Vec<Face>,
    lines: Vec<(f64, String)>,
}

impl SceneBuilder {
    fn prism(&mut self, ring: &[(f64, f64)], z: f64, height: f64, material: &'static str) {
        self.prism_opacity(ring, z, height, material, None);
    }

    fn prism_opacity(
        &mut self,
        ring: &[(f64, f64)],
        z: f64,
        height: f64,
        material: &'static str,
        opacity: Option<f64>,
    ) {
        if ring.len() < 3 {
            return;
        }
        let top = ring
            .iter()
            .map(|(x, y)| Point3(*x, *y, z + height))
            .collect::<Vec<_>>();
        self.add_face(top.clone(), "face-top", material, opacity);

        let signed_area = ring
            .iter()
            .enumerate()
            .map(|(i, (x1, y1))| {
                let (x2, y2) = ring[(i + 1) % ring.len()];
                x1 * y2 - x2 * y1
            })
            .sum::<f64>();
        let counter_clockwise = signed_area > 0.0;
        let projection = Projection::fitted();
        for index in 0..ring.len() {
            let (ax, ay) = ring[index];
            let (bx, by) = ring[(index + 1) % ring.len()];
            let dx = bx - ax;
            let dy = by - ay;
            let length = (dx * dx + dy * dy).sqrt().max(1.0);
            let normal = if counter_clockwise {
                (dy / length, -dx / length)
            } else {
                (-dy / length, dx / length)
            };
            if normal.0 * projection.cos_a + normal.1 * projection.sin_a <= 0.0 {
                continue;
            }
            let next = (index + 1) % ring.len();
            let low_a = Point3(ax, ay, z);
            let low_b = Point3(bx, by, z);
            self.add_face(
                vec![top[index], top[next], low_b, low_a],
                if normal.0 > normal.1 {
                    "face-side-a"
                } else {
                    "face-side-b"
                },
                material,
                opacity,
            );
        }
    }

    fn box_solid(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        width: f64,
        depth: f64,
        height: f64,
        material: &'static str,
    ) {
        self.prism(
            &[
                (x, y),
                (x + width, y),
                (x + width, y + depth),
                (x, y + depth),
            ],
            z,
            height,
            material,
        );
    }

    fn box_opacity(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        width: f64,
        depth: f64,
        height: f64,
        material: &'static str,
        opacity: f64,
    ) {
        self.prism_opacity(
            &[
                (x, y),
                (x + width, y),
                (x + width, y + depth),
                (x, y + depth),
            ],
            z,
            height,
            material,
            Some(opacity),
        );
    }

    fn round_solid(
        &mut self,
        cx: f64,
        cy: f64,
        radius: f64,
        z: f64,
        height: f64,
        material: &'static str,
    ) {
        let ring = (0..12)
            .map(|step| {
                let angle = step as f64 * (2.0 * PI / 12.0);
                (cx + radius * angle.cos(), cy + radius * angle.sin())
            })
            .collect::<Vec<_>>();
        self.prism(&ring, z, height, material);
    }

    fn add_face(
        &mut self,
        points: Vec<Point3>,
        class: &'static str,
        material: &'static str,
        opacity: Option<f64>,
    ) {
        let projection = Projection::fitted();
        let depth = points.iter().map(|p| projection.depth(*p)).sum::<f64>() / points.len() as f64;
        self.faces.push(Face {
            points,
            class,
            material,
            depth,
            opacity,
        });
    }

    fn line(&mut self, points: &[Point3], class: &'static str, width: f64, close: bool) {
        if points.len() < 2 {
            return;
        }
        let projection = Projection::fitted();
        let depth = points.iter().map(|p| projection.depth(*p)).sum::<f64>() / points.len() as f64;
        let mut d = String::new();
        for (index, point) in points.iter().enumerate() {
            let screen = projection.project(*point);
            d.push_str(if index == 0 { "M" } else { "L" });
            d.push_str(&format!("{} {}", num(screen.0), num(screen.1)));
        }
        if close {
            d.push('Z');
        }
        self.lines.push((
            depth,
            format!(
                "<path class=\"scene-line {class}\" d=\"{d}\" style=\"stroke-width:{width}px\"/>"
            ),
        ));
    }

    fn markup(mut self) -> String {
        self.faces.sort_by(|a, b| a.depth.total_cmp(&b.depth));
        self.lines.sort_by(|a, b| a.0.total_cmp(&b.0));
        let projection = Projection::fitted();
        let mut markup = String::new();
        for face in self.faces {
            let points = face
                .points
                .iter()
                .map(|point| {
                    let p = projection.project(*point);
                    format!("{},{}", num(p.0), num(p.1))
                })
                .collect::<Vec<_>>()
                .join(" ");
            let fill = face_fill(face.material, face.class);
            let opacity = face
                .opacity
                .map(|value| format!(" opacity=\"{}\"", num(value)))
                .unwrap_or_default();
            markup.push_str(&format!(
                "<polygon class=\"scene-face {} material-{}\" points=\"{}\" fill=\"{}\" stroke=\"var(--scene-outline,#76664f)\" stroke-width=\".45\" vector-effect=\"non-scaling-stroke\"{}/>",
                face.class, face.material, points, fill, opacity
            ));
        }
        for (_, line) in self.lines {
            markup.push_str(&line);
        }
        markup
    }
}

fn face_fill(material: &str, class: &str) -> &'static str {
    match (material, class) {
        ("ground", "face-top") => "var(--scene-ground-top,#d9c8a8)",
        ("ground", _) => "var(--scene-ground-side,#a99270)",
        ("wood", "face-top") => "var(--scene-wood-top,#bd9564)",
        ("wood", "face-side-a") => "var(--scene-wood-a,#8d6845)",
        ("wood", _) => "var(--scene-wood-b,#a97e54)",
        ("brass", "face-top") => "var(--scene-brass-top,#d6a65a)",
        ("brass", "face-side-a") => "var(--scene-brass-a,#9c6d31)",
        ("brass", _) => "var(--scene-brass-b,#bc8841)",
        ("gold", "face-top") => "var(--scene-gold-top,#efcf79)",
        ("gold", "face-side-a") => "var(--scene-gold-a,#b18436)",
        ("gold", _) => "var(--scene-gold-b,#d1a749)",
        ("silver", "face-top") => "var(--scene-silver-top,#e4ddcb)",
        ("silver", "face-side-a") => "var(--scene-silver-a,#aaa18f)",
        ("silver", _) => "var(--scene-silver-b,#c9c1ae)",
        ("blue", "face-top") => "var(--scene-blue-top,#788d9b)",
        ("blue", "face-side-a") => "var(--scene-blue-a,#526876)",
        ("blue", _) => "var(--scene-blue-b,#667d8b)",
        ("purple", "face-top") => "var(--scene-purple-top,#927b91)",
        ("purple", "face-side-a") => "var(--scene-purple-a,#68556a)",
        ("purple", _) => "var(--scene-purple-b,#806b82)",
        ("scarlet", "face-top") => "var(--scene-scarlet-top,#ba786c)",
        ("scarlet", "face-side-a") => "var(--scene-scarlet-a,#87544c)",
        ("scarlet", _) => "var(--scene-scarlet-b,#a4665d)",
        ("linen", "face-top") => "var(--scene-linen-top,#f0e9d9)",
        ("linen", "face-side-a") => "var(--scene-linen-a,#c6baa3)",
        ("linen", _) => "var(--scene-linen-b,#ddd2be)",
        ("dark", "face-top") => "var(--scene-dark-top,#50483a)",
        ("dark", "face-side-a") => "var(--scene-dark-a,#312c24)",
        ("dark", _) => "var(--scene-dark-b,#40392f)",
        _ => "var(--scene-wood-top,#bd9564)",
    }
}

fn num(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.abs() < 0.05 {
        "0".to_string()
    } else {
        format!("{rounded:.1}").trim_end_matches(".0").to_string()
    }
}

fn ring_rect(x: f64, y: f64, w: f64, d: f64) -> [(f64, f64); 4] {
    [(x, y), (x + w, y), (x + w, y + d), (x, y + d)]
}

fn build_courtyard() -> String {
    let mut scene = SceneBuilder::default();
    for x in (-96..=96).step_by(24) {
        for y in [-62.0, 62.0] {
            scene.box_solid(x as f64 - 1.2, y - 1.2, 0.0, 2.4, 2.4, 12.0, "wood");
            scene.box_solid(x as f64 - 1.7, y - 1.7, 11.0, 3.4, 3.4, 1.2, "silver");
        }
    }
    for x in [-96.0, 96.0] {
        for y in [-48.0, -24.0, 24.0, 48.0] {
            scene.box_solid(x - 1.2, y - 1.2, 0.0, 2.4, 2.4, 12.0, "wood");
            scene.box_solid(x - 1.7, y - 1.7, 11.0, 3.4, 3.4, 1.2, "silver");
        }
    }
    for x in [-84.0, -60.0, -36.0, -12.0, 12.0, 36.0, 60.0, 84.0] {
        scene.box_opacity(x, -62.3, 2.0, 22.0, 0.6, 8.0, "linen", 0.5);
        scene.box_opacity(x, 61.7, 2.0, 22.0, 0.6, 8.0, "linen", 0.5);
    }
    for y in [-54.0, -36.0, 36.0, 54.0] {
        scene.box_opacity(-96.3, y, 2.0, 0.6, 18.0, 8.0, "linen", 0.5);
        scene.box_opacity(95.7, y, 2.0, 0.6, 18.0, 8.0, "linen", 0.5);
    }
    // The east gate is left open in the model, with its four support posts and
    // a small embroidered panel at the threshold.
    scene.box_opacity(94.0, -14.0, 3.0, 0.8, 6.0, 8.0, "blue", 0.82);
    scene.box_opacity(94.0, 8.0, 3.0, 0.8, 6.0, 8.0, "scarlet", 0.82);
    scene.markup()
}

fn build_altar() -> String {
    let mut scene = SceneBuilder::default();
    scene.box_solid(53.0, -7.0, 0.0, 16.0, 16.0, 9.0, "brass");
    scene.box_solid(54.0, -6.0, 8.0, 14.0, 14.0, 1.2, "wood");
    for (x, y) in [(53.0, -7.0), (66.0, -7.0), (66.0, 6.0), (53.0, 6.0)] {
        scene.box_solid(x, y, 9.0, 3.0, 3.0, 5.0, "brass");
    }
    // The grate and its carrying rings are visible details of the altar's body.
    for offset in [2.0, 5.0, 8.0, 11.0] {
        scene.line(
            &[
                Point3(54.0 + offset, -5.0, 8.7),
                Point3(54.0 + offset, 7.0, 8.7),
            ],
            "line-brass",
            0.55,
            false,
        );
        scene.line(
            &[
                Point3(54.0, -6.0 + offset, 8.7),
                Point3(68.0, -6.0 + offset, 8.7),
            ],
            "line-brass",
            0.55,
            false,
        );
    }
    for y in [-7.0, 7.0] {
        scene.round_solid(60.0, y, 1.3, 2.0, 2.0, "brass");
    }
    scene.markup()
}

fn build_basin() -> String {
    let mut scene = SceneBuilder::default();
    scene.round_solid(38.0, 0.0, 4.2, 0.0, 2.4, "brass");
    scene.round_solid(38.0, 0.0, 2.7, 2.4, 4.2, "brass");
    scene.round_solid(38.0, 0.0, 7.1, 6.2, 2.0, "brass");
    scene.round_solid(38.0, 0.0, 5.3, 6.0, 1.0, "dark");
    scene.round_solid(38.0, 0.0, 2.3, 6.5, 0.5, "brass");
    scene.markup()
}

fn build_tent() -> (String, String, String) {
    let mut backdrop = SceneBuilder::default();
    backdrop.box_solid(-61.0, -27.0, -1.0, 90.0, 54.0, 1.8, "linen");
    // Rear and north-side structure belongs behind the furnishings and veil.
    for (x, y) in [(-59.0, -25.0), (-59.0, 23.0), (26.0, -25.0)] {
        backdrop.box_solid(x, y, 0.0, 2.2, 2.2, 37.0, "wood");
        backdrop.box_solid(x - 0.5, y - 0.5, 36.0, 3.2, 3.2, 1.0, "gold");
    }
    backdrop.box_opacity(-60.0, -25.0, 2.0, 86.0, 0.8, 34.0, "linen", 0.56);
    backdrop.box_opacity(-61.0, -25.0, 2.0, 0.8, 50.0, 34.0, "linen", 0.48);
    for x in [-48.0, -28.0, -8.0, 12.0] {
        backdrop.box_solid(x, -25.6, 2.0, 0.5, 1.2, 34.0, "blue");
    }
    // The south side stays open in this cutaway. The east near-side post
    // remains a small, real hit target; its far partner is in the backdrop.
    let mut foreground = SceneBuilder::default();
    foreground.box_solid(26.0, 23.0, 0.0, 2.2, 2.2, 37.0, "wood");
    foreground.box_solid(25.5, 22.5, 36.0, 3.2, 3.2, 1.0, "gold");
    foreground.box_solid(26.0, -26.0, 36.0, 2.2, 50.0, 1.0, "wood");
    // The roof is a distinct moving group; CSS moves it aside in cutaway mode.
    let mut roof = SceneBuilder::default();
    roof.box_solid(-62.0, -28.0, 40.0, 92.0, 56.0, 2.0, "linen");
    for x in [-56.0, -36.0, -16.0, 4.0, 24.0] {
        roof.box_solid(x, -27.5, 42.0, 1.1, 55.0, 1.0, "purple");
    }
    (backdrop.markup(), foreground.markup(), roof.markup())
}

fn build_table() -> String {
    let mut scene = SceneBuilder::default();
    // North side of the holy place, west of the entrance.
    let x = -15.0;
    let y = -19.0;
    for px in [x, x + 15.0] {
        for py in [y, y + 9.0] {
            scene.box_solid(px, py, 0.0, 1.6, 1.6, 13.0, "wood");
            scene.round_solid(px + 0.8, py + 0.8, 1.2, 1.2, 1.0, "gold");
        }
    }
    scene.box_solid(x - 1.0, y - 1.0, 13.0, 18.0, 12.0, 1.7, "gold");
    scene.box_solid(x - 1.5, y - 1.5, 14.7, 19.0, 13.0, 1.0, "gold");
    scene.box_solid(x - 1.0, y - 1.0, 15.7, 18.0, 0.8, 1.0, "gold");
    scene.box_solid(x - 1.0, y + 10.2, 15.7, 18.0, 0.8, 1.0, "gold");
    // Two small flat loaves suggest the shewbread without asserting a count.
    scene.box_solid(x + 3.0, y + 2.0, 16.8, 4.8, 3.4, 0.7, "linen");
    scene.box_solid(x + 9.0, y + 2.0, 16.8, 4.8, 3.4, 0.7, "linen");
    for px in [x + 2.0, x + 13.0] {
        scene.round_solid(px, y - 1.3, 0.9, 1.0, 1.2, "gold");
    }
    scene.markup()
}

fn build_lamp() -> String {
    let mut scene = SceneBuilder::default();
    let cx = -12.0;
    let cy = 16.0;
    scene.box_solid(cx - 4.0, cy - 3.2, 0.0, 8.0, 6.4, 1.2, "gold");
    scene.box_solid(cx - 2.5, cy - 2.0, 1.2, 5.0, 4.0, 1.4, "gold");
    scene.box_solid(cx - 1.0, cy - 0.9, 2.6, 2.0, 1.8, 27.0, "gold");
    scene.round_solid(cx, cy, 1.8, 28.0, 1.5, "gold");
    for (side, levels) in [(-1.0, [8.0, 16.0, 24.0]), (1.0, [8.0, 16.0, 24.0])] {
        for level in levels {
            let end_x = cx + side * 9.0;
            scene.line(
                &[
                    Point3(cx + side * 0.8, cy, level),
                    Point3(cx + side * 3.5, cy, level + 1.2),
                    Point3(cx + side * 6.2, cy, level + 3.4),
                    Point3(end_x, cy, level + 5.5),
                ],
                "line-gold",
                1.8,
                false,
            );
            scene.round_solid(end_x, cy, 1.8, level + 5.0, 1.2, "gold");
            scene.round_solid(cx + side * 3.7, cy, 0.8, level + 1.2, 1.1, "gold");
        }
    }
    scene.markup()
}

fn build_veil() -> String {
    let mut cloth = SceneBuilder::default();
    let mut scene = SceneBuilder::default();
    let x = -28.0;
    for index in 0..8 {
        let y = -24.0 + index as f64 * 6.0;
        let material = match index % 4 {
            0 => "blue",
            1 => "purple",
            2 => "scarlet",
            _ => "linen",
        };
        cloth.box_opacity(x, y, 2.0, 0.8, 6.0, 33.0, material, 0.82);
    }
    for y in [-23.0, -8.0, 8.0, 23.0] {
        scene.box_solid(x - 1.0, y - 1.0, 0.0, 2.8, 2.8, 35.0, "wood");
        scene.round_solid(x, y, 1.8, 34.8, 1.0, "gold");
        scene.round_solid(x, y, 2.0, 0.0, 1.2, "silver");
    }
    format!(
        "<g class=\"veil-cloth\">{}</g>{}",
        cloth.markup(),
        scene.markup()
    )
}

fn build_ark() -> String {
    let mut scene = SceneBuilder::default();
    let x = -51.0;
    let y = -5.0;
    scene.box_solid(x, y, 2.0, 13.0, 10.0, 7.0, "wood");
    scene.box_solid(x - 0.3, y - 0.3, 8.0, 13.6, 10.6, 1.4, "gold");
    scene.box_solid(x - 0.8, y - 0.8, 9.4, 14.6, 11.6, 1.0, "gold");
    for side_y in [y - 0.5, y + 9.5] {
        scene.box_solid(x - 8.0, side_y, 4.5, 29.0, 0.8, 1.4, "gold");
        for px in [x + 1.0, x + 10.0] {
            scene.box_solid(px, side_y - 0.2, 5.0, 1.7, 1.2, 2.4, "gold");
        }
    }
    scene.markup()
}

fn part_scene(part: &Part, selected: &str, client: bool, content: &str) -> String {
    let href = route_path(&format!("/explore/tabernacle?part={}", part.id), client);
    let selected_class = if part.id == selected {
        " is-selected"
    } else {
        ""
    };
    // Keep depth on the named group so the whole object remains a single
    // keyboard target; all of its physical faces are painter-sorted inside.
    let group = format!(
        "<a class=\"scene-link\" href=\"{}\" aria-label=\"{}\"><g class=\"scene-object{}\" data-scene-part=\"{}\" data-scene-depth=\"{}\" pointer-events=\"visiblePainted\"><title>{}</title>{}</g></a>",
        esc(&href),
        esc(part.aria),
        selected_class,
        part.id,
        num(part.depth),
        esc(part.name),
        content
    );
    group
}

fn render_scene(selected: &str, client: bool) -> String {
    let mut parts = PARTS.iter().collect::<Vec<_>>();
    parts.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    let mut contents = String::new();
    let (tent_backdrop, tent_foreground, roof_markup) = build_tent();
    for part in parts {
        let content = match part.id {
            "courtyard" => build_courtyard(),
            "altar" => build_altar(),
            "basin" => build_basin(),
            "tent" => tent_foreground.clone(),
            "table" => build_table(),
            "lamp" => build_lamp(),
            "veil" => build_veil(),
            "ark" => build_ark(),
            _ => String::new(),
        };
        contents.push_str(&part_scene(part, selected, client, &content));
    }
    let roof_selected = if selected == "tent" {
        " is-selected"
    } else {
        ""
    };
    format!(
        "<svg class=\"tabernacle-model\" viewBox=\"0 0 640 440\" role=\"group\" aria-labelledby=\"tabernacle-title tabernacle-desc\"><title id=\"tabernacle-title\">Isometric Tabernacle reconstruction</title><desc id=\"tabernacle-desc\">A schematic cutaway of the courtyard and tabernacle furnishings. Choose a named object to open its Scripture and explanation.</desc><g class=\"scene-ground\" pointer-events=\"none\">{}{}</g>{contents}<g class=\"scene-object scene-roof{}\" data-scene-part=\"tent\" aria-hidden=\"true\" pointer-events=\"none\">{roof_markup}</g></svg>",
        build_ground(),
        tent_backdrop,
        roof_selected
    )
}

fn build_ground() -> String {
    let mut scene = SceneBuilder::default();
    scene.prism(&ring_rect(-104.0, -65.0, 208.0, 130.0), -3.0, 1.0, "ground");
    scene.markup()
}

fn part_for(id: &str) -> Option<&'static Part> {
    PARTS.iter().find(|part| part.id == id)
}

fn picker(selected: &str, client: bool) -> String {
    let mut list = String::from("<ul class=\"part-picker\" aria-label=\"Tabernacle parts\">");
    for part in &PARTS {
        let href = route_path(&format!("/explore/tabernacle?part={}", part.id), client);
        let current = if part.id == selected {
            " aria-current=\"page\""
        } else {
            ""
        };
        list.push_str(&format!(
            "<li><a href=\"{}\"{}>{}</a></li>",
            esc(&href),
            current,
            esc(part.name)
        ));
    }
    list.push_str("</ul>");
    list
}

fn passage_blocks(part: &Part, client: bool) -> String {
    let mut output = String::new();
    for text_reference in part.references {
        let reference = parse_ref(text_reference).expect("Tabernacle source reference is valid");
        let canonical = ref_label(&reference);
        output.push_str(&format!(
            "<div class=\"passage-excerpt\"><h4>{}</h4><div class=\"scripture\" data-reference=\"{}\" aria-label=\"{} · King James Version\">",
            esc(&canonical),
            esc(&canonical),
            esc(&canonical)
        ));
        for (verse, text) in verses(&reference) {
            output.push_str(&format!(
                "<span class=\"v\" data-reference=\"{}\" data-verse=\"{}\"><sup>{}</sup> {}</span>",
                esc(&canonical),
                verse,
                verse,
                esc(text)
            ));
        }
        output.push_str("</div></div>");
    }
    let chapter = parse_ref(part.references[0]).expect("Tabernacle source reference is valid");
    let chapter_link = reference_url(
        &Ref {
            v1: None,
            v2: None,
            ..chapter
        },
        client,
    );
    output.push_str(&format!(
        "<p class=\"chapter-link\"><a href=\"{}\">Read {} {} →</a></p>",
        esc(&chapter_link),
        esc(super::NAMES[chapter.book]),
        chapter.ch
    ));
    output
}

pub(super) fn listen_controls(reference: &str) -> String {
    format!(
        "<div class=\"listen-controls\" data-read-aloud data-reference=\"{}\"><div class=\"listen-actions\"><button type=\"button\" data-audio-action=\"play\">Play</button><button type=\"button\" data-audio-action=\"pause\">Pause</button><button type=\"button\" data-audio-action=\"stop\">Stop</button></div><label>Speed <select data-audio-rate aria-label=\"Speech speed\"><option value=\"0.75\">0.75×</option><option value=\"1\" selected>1×</option><option value=\"1.25\">1.25×</option><option value=\"1.5\">1.5×</option></select></label><label>Voice <select data-audio-voice aria-label=\"Voice\"><option value=\"\">Default voice</option></select></label><span data-audio-status role=\"status\" aria-live=\"polite\">Ready to read Scripture.</span><small>Synthetic voice. Availability depends on your browser and device.</small></div>",
        esc(reference)
    )
}

fn explore_body(selected: &str, client: bool) -> String {
    let part = part_for(selected).expect("selected Tabernacle part is known");
    let mut body = String::from(
        "<section id=\"explore\" class=\"explore-page\"><div class=\"explore-heading\"><p class=\"eyebrow\">Read · Explore</p><h1>Explore the Tabernacle</h1><p>Choose an object in the scene or use the part list. Each selection pairs a reconstruction with its King James passage.</p></div><div class=\"explore-layout\"><section class=\"scene-column\" aria-label=\"Interactive Tabernacle scene\"><div class=\"scene-card is-cutaway\"><div class=\"scene-controls\"><button type=\"button\" data-scene-toggle aria-pressed=\"true\">Show roof</button><span>Cutaway view</span></div>",
    );
    body.push_str(&render_scene(selected, client));
    body.push_str("</div>");
    body.push_str(&picker(selected, client));
    body.push_str("</section><article class=\"passage-card\"><p class=\"eyebrow\">Scripture · King James Version</p><h2>");
    body.push_str(&esc(part.name));
    body.push_str("</h2>");
    body.push_str(&listen_controls(&part.references.join("; ")));
    body.push_str(&passage_blocks(part, client));
    body.push_str("<section class=\"context-note\"><h3>Explanation</h3><p>Illustrated reconstruction. The placement and appearance of objects are schematic.</p><p>");
    body.push_str(&esc(part.note));
    body.push_str("</p></section></article></div></section>");
    body
}

pub(super) fn explore_page(client: bool) -> RouteResponse {
    response("Explore the Tabernacle", "courtyard", client)
}

pub(super) fn selected_page(selected: Option<&str>, client: bool) -> RouteResponse {
    let selected = selected.unwrap_or("courtyard");
    if part_for(selected).is_none() {
        let content = format!(
            "<section class=\"explore-page\"><h1>Tabernacle part not found</h1><p>Choose one of the named parts to continue.</p><p><a href=\"{}\">Explore the Tabernacle</a></p></section>",
            esc(&route_path("/explore", client))
        );
        let body = super::page_body("", &content, client);
        return RouteResponse {
            status: 404,
            content_type: "text/html; charset=utf-8".into(),
            location: None,
            title: Some("Tabernacle part not found".into()),
            query: None,
            anchor: None,
            body: if client {
                body
            } else {
                super::document("Tabernacle part not found", &body)
            },
        };
    }
    response("Explore the Tabernacle", selected, client)
}

fn response(title: &str, selected: &str, client: bool) -> RouteResponse {
    let content = explore_body(selected, client);
    let body = super::page_body("", &content, client);
    RouteResponse {
        status: 200,
        content_type: "text/html; charset=utf-8".into(),
        location: None,
        title: Some(title.to_string()),
        query: None,
        anchor: None,
        body: if client {
            body
        } else {
            super::document(title, &body)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{part_for, PARTS};
    use crate::{parse_ref, route, verses};

    #[test]
    fn explore_routes_render_all_eight_named_parts_and_keyboard_links() {
        let landing = route("/explore", false);
        assert_eq!(landing.status, 200);
        for part in &PARTS {
            assert!(landing
                .body
                .contains(&format!("data-scene-part=\"{}\"", part.id)));
            assert!(landing
                .body
                .contains(&format!("aria-label=\"{}\"", part.aria)));
            assert!(landing.body.contains(&format!("?part={}", part.id)));
        }
        assert!(landing.body.contains("class=\"part-picker\""));
        assert!(landing.body.contains("data-scene-toggle"));
        assert!(landing.body.matches("<polygon").count() >= 40);
    }

    #[test]
    fn each_part_uses_canonical_kjv_excerpts_and_its_matching_route_state() {
        for part in &PARTS {
            let route_path = format!("/explore/tabernacle?part={}", part.id);
            let response = route(&route_path, false);
            assert_eq!(response.status, 200, "{route_path}");
            assert!(response.body.contains("class=\"scripture\""));
            assert!(response.body.contains("class=\"context-note\""));
            assert!(response
                .body
                .contains(&format!("data-scene-part=\"{}\"", part.id)));
            for source in part.references {
                let reference = parse_ref(source).expect("part source parses");
                let canonical = crate::ref_label(&reference);
                assert!(response
                    .body
                    .contains(&format!("data-reference=\"{canonical}\"")));
                for (number, text) in verses(&reference) {
                    assert!(response.body.contains(&format!("data-verse=\"{number}\"")));
                    assert!(response.body.contains(&crate::esc(text)));
                }
            }
            assert_eq!(part_for(part.id).unwrap().id, part.id);
        }
    }

    #[test]
    fn invalid_part_is_a_real_not_found_response() {
        assert_eq!(route("/explore/tabernacle?part=made-up", false).status, 404);
    }

    #[test]
    fn chapter_and_passage_views_share_the_kjv_read_aloud_controls() {
        for (chapter_uri, passage_uri) in [
            ("/book/Exodus/25", "/go?q=Exodus%2025%3A31-32"),
            ("#/book/Exodus/25", "#/go?q=Exodus%2025%3A31-32"),
        ] {
            for uri in [chapter_uri, passage_uri] {
                let response = route(uri, uri.starts_with('#'));
                assert_eq!(response.status, 200);
                assert!(response.body.contains("data-read-aloud"));
                assert!(response.body.contains("data-audio-action=\"play\""));
                assert!(response.body.contains("data-audio-rate"));
                assert!(response.body.contains("data-audio-voice"));
                assert!(response.body.contains("data-audio-status"));
                assert!(response.body.contains("class=\"v\""));
            }
        }
    }
}

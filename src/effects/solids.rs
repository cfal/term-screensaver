use std::f64::consts::TAU;

use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, ramp, seeded_rng},
    math::{Vec3, Vec4, draw_projected_line, fitting_scale, project},
};

const DONUT_BOUNDING_RADIUS: f64 = 1.75;
const TESSERACT_BOUNDING_RADIUS: f64 = 1.1;
const TESSERACT_W_DISTANCE: f64 = 2.6;

pub struct Donut {
    samples: Vec<(Vec3, Vec3)>,
    speed_x: f64,
    speed_z: f64,
    palette: [Rgb; 2],
}

impl Donut {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let mut samples = Vec::with_capacity(72 * 28);
        for major_step in 0..72 {
            let major = major_step as f64 / 72.0 * TAU;
            for minor_step in 0..28 {
                let minor = minor_step as f64 / 28.0 * TAU;
                let radial = 1.2 + 0.48 * minor.cos();
                samples.push((
                    Vec3::new(
                        radial * major.cos(),
                        radial * major.sin(),
                        0.48 * minor.sin(),
                    ),
                    Vec3::new(
                        minor.cos() * major.cos(),
                        minor.cos() * major.sin(),
                        minor.sin(),
                    ),
                ));
            }
        }
        Self {
            samples,
            speed_x: rng.gen_range(0.28..0.55),
            speed_z: rng.gen_range(0.16..0.38),
            palette: [Rgb::new(22, 126, 198), Rgb::new(255, 221, 102)],
        }
    }
}

impl Animation for Donut {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        const ASCII: &[char] = &['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];
        const UNICODE: &[char] = &['·', '·', ':', '•', '+', '*', 'o', 'O', '0', '●', '▓', '@'];
        let glyphs = if frame.ascii { ASCII } else { UNICODE };
        let scale = fitting_scale(canvas.width(), canvas.height(), DONUT_BOUNDING_RADIUS, 0.88);
        let angle_x = frame.scene_seconds * self.speed_x + 0.7;
        let angle_z = frame.scene_seconds * self.speed_z;
        let light = Vec3::new(-0.35, 0.45, 0.82).normalized();

        for &(point, normal) in &self.samples {
            let point = point.rotate_x(angle_x).rotate_z(angle_z);
            let normal = normal.rotate_x(angle_x).rotate_z(angle_z);
            let luminance = (normal.dot(light) * 0.5 + 0.5).clamp(0.0, 1.0);
            let Some(projected) = project(point, canvas.width(), canvas.height(), scale) else {
                continue;
            };
            canvas.plot(
                projected.x,
                projected.y,
                projected.inverse_depth,
                ramp(luminance, glyphs),
                Style {
                    foreground: frame.color(lerp_color(
                        self.palette[0],
                        self.palette[1],
                        luminance,
                    )),
                    bold: luminance > 0.72,
                    dim: luminance < 0.2,
                },
            );
        }
    }
}

#[derive(Clone, Copy)]
enum Shape {
    Cube,
    Octahedron,
}

pub struct Wireframe {
    shape: Shape,
    speed: Vec3,
    phase: Vec3,
    colors: [Rgb; 2],
}

impl Wireframe {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        Self {
            shape: if rng.gen_bool(0.5) {
                Shape::Cube
            } else {
                Shape::Octahedron
            },
            speed: Vec3::new(
                rng.gen_range(0.22..0.48),
                rng.gen_range(0.26..0.52),
                rng.gen_range(-0.3..0.3),
            ),
            phase: Vec3::new(rng.r#gen(), rng.r#gen(), rng.r#gen()),
            colors: [Rgb::new(251, 92, 106), Rgb::new(88, 226, 204)],
        }
    }

    fn geometry(&self) -> (&'static [Vec3], &'static [(usize, usize)]) {
        const CUBE_VERTICES: &[Vec3] = &[
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];
        const CUBE_EDGES: &[(usize, usize)] = &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        const OCTAHEDRON_VERTICES: &[Vec3] = &[
            Vec3::new(1.35, 0.0, 0.0),
            Vec3::new(-1.35, 0.0, 0.0),
            Vec3::new(0.0, 1.35, 0.0),
            Vec3::new(0.0, -1.35, 0.0),
            Vec3::new(0.0, 0.0, 1.35),
            Vec3::new(0.0, 0.0, -1.35),
        ];
        const OCTAHEDRON_EDGES: &[(usize, usize)] = &[
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 4),
            (4, 3),
            (3, 5),
            (5, 2),
        ];
        match self.shape {
            Shape::Cube => (CUBE_VERTICES, CUBE_EDGES),
            Shape::Octahedron => (OCTAHEDRON_VERTICES, OCTAHEDRON_EDGES),
        }
    }

    fn bounding_radius(&self) -> f64 {
        match self.shape {
            Shape::Cube => 3.0_f64.sqrt(),
            Shape::Octahedron => 1.35,
        }
    }
}

impl Animation for Wireframe {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        let (vertices, edges) = self.geometry();
        let angles = Vec3::new(
            frame.scene_seconds * self.speed.x + self.phase.x,
            frame.scene_seconds * self.speed.y + self.phase.y,
            frame.scene_seconds * self.speed.z + self.phase.z,
        );
        let scale = fitting_scale(
            canvas.width(),
            canvas.height(),
            self.bounding_radius(),
            0.84,
        );
        let projected: Vec<_> = vertices
            .iter()
            .map(|vertex| {
                project(
                    vertex.rotate(angles.x, angles.y, angles.z),
                    canvas.width(),
                    canvas.height(),
                    scale,
                )
            })
            .collect();

        for (edge_index, &(start, end)) in edges.iter().enumerate() {
            let (Some(start), Some(end)) = (projected[start], projected[end]) else {
                continue;
            };
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let glyph = if frame.ascii {
                if dy.abs() * 2 < dx.abs() {
                    '-'
                } else if dx.abs() < dy.abs() {
                    '|'
                } else if dx.signum() == dy.signum() {
                    '\\'
                } else {
                    '/'
                }
            } else {
                '•'
            };
            let amount = edge_index as f64 / edges.len() as f64;
            draw_projected_line(
                canvas,
                start,
                end,
                glyph,
                Style {
                    foreground: frame.color(lerp_color(self.colors[0], self.colors[1], amount)),
                    bold: edge_index % 3 == 0,
                    dim: false,
                },
            );
        }

        for vertex in projected.into_iter().flatten() {
            canvas.plot(
                vertex.x,
                vertex.y,
                vertex.inverse_depth + 0.001,
                if frame.ascii { '+' } else { '●' },
                Style {
                    foreground: frame.color(self.colors[1]),
                    bold: true,
                    dim: false,
                },
            );
        }
    }
}

pub struct Tesseract {
    vertices: Vec<Vec4>,
    edges: Vec<(usize, usize)>,
    speed_xw: f64,
    speed_zw: f64,
    speed_y: f64,
    colors: [Rgb; 2],
}

impl Tesseract {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let vertices = (0..16)
            .map(|index| {
                Vec4::new(
                    if index & 1 == 0 { -1.0 } else { 1.0 },
                    if index & 2 == 0 { -1.0 } else { 1.0 },
                    if index & 4 == 0 { -1.0 } else { 1.0 },
                    if index & 8 == 0 { -1.0 } else { 1.0 },
                )
            })
            .collect();
        let mut edges: Vec<(usize, usize)> = Vec::with_capacity(32);
        for a in 0..16usize {
            for b in (a + 1)..16 {
                if (a ^ b).count_ones() == 1 {
                    edges.push((a, b));
                }
            }
        }
        Self {
            vertices,
            edges,
            speed_xw: rng.gen_range(0.3..0.55),
            speed_zw: rng.gen_range(0.2..0.45),
            speed_y: rng.gen_range(0.1..0.25),
            colors: [Rgb::new(148, 111, 255), Rgb::new(87, 226, 255)],
        }
    }
}

impl Animation for Tesseract {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        let angle_xw = frame.scene_seconds * self.speed_xw + 0.6;
        let angle_zw = frame.scene_seconds * self.speed_zw;
        let angle_y = frame.scene_seconds * self.speed_y + 0.4;
        let scale = fitting_scale(
            canvas.width(),
            canvas.height(),
            TESSERACT_BOUNDING_RADIUS,
            0.85,
        );
        let projected: Vec<_> = self
            .vertices
            .iter()
            .map(|vertex| {
                let rotated = vertex.rotate_xw(angle_xw).rotate_zw(angle_zw);
                let point = rotated.perspective(TESSERACT_W_DISTANCE).rotate_y(angle_y);
                (
                    project(point, canvas.width(), canvas.height(), scale),
                    rotated.w,
                )
            })
            .collect();

        for (edge_index, &(start, end)) in self.edges.iter().enumerate() {
            let (Some(start_point), Some(end_point)) = (projected[start].0, projected[end].0)
            else {
                continue;
            };
            let dx = end_point.x - start_point.x;
            let dy = end_point.y - start_point.y;
            let glyph = if frame.ascii {
                if dy.abs() * 2 < dx.abs() {
                    '-'
                } else if dx.abs() < dy.abs() {
                    '|'
                } else if dx.signum() == dy.signum() {
                    '\\'
                } else {
                    '/'
                }
            } else {
                '•'
            };
            let near = ((projected[start].1 + projected[end].1) * 0.5).clamp(-1.0, 1.0);
            let amount = edge_index as f64 / self.edges.len() as f64;
            draw_projected_line(
                canvas,
                start_point,
                end_point,
                glyph,
                Style {
                    foreground: frame.color(lerp_color(self.colors[0], self.colors[1], amount)),
                    bold: near < -0.2,
                    dim: near > 0.2,
                },
            );
        }

        for (vertex, _) in projected
            .into_iter()
            .filter_map(|(point, w)| point.map(|p| (p, w)))
        {
            canvas.plot(
                vertex.x,
                vertex.y,
                vertex.inverse_depth + 0.001,
                if frame.ascii { '+' } else { '●' },
                Style {
                    foreground: frame.color(self.colors[1]),
                    bold: true,
                    dim: false,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_on_screen(point: Vec3, width: u16, height: u16, scale: f64) {
        let projected = project(point, width, height, scale).expect("point crossed the camera");
        assert!((0..i32::from(width)).contains(&projected.x));
        assert!((0..i32::from(height)).contains(&projected.y));
    }

    #[test]
    fn tesseract_vertices_fit_common_terminal_sizes() {
        let tesseract = Tesseract::new(91);
        for (width, height) in [(79, 24), (119, 30), (39, 12)] {
            let scale = fitting_scale(width, height, TESSERACT_BOUNDING_RADIUS, 0.85);
            for seconds in [0.0, 2.5, 8.884, 31.0] {
                let angle_xw = seconds * tesseract.speed_xw + 0.6;
                let angle_zw = seconds * tesseract.speed_zw;
                let angle_y = seconds * tesseract.speed_y + 0.4;
                for &vertex in &tesseract.vertices {
                    let rotated = vertex.rotate_xw(angle_xw).rotate_zw(angle_zw);
                    let point = rotated.perspective(TESSERACT_W_DISTANCE).rotate_y(angle_y);
                    assert_on_screen(point, width, height, scale);
                }
            }
        }
    }

    #[test]
    fn donut_samples_fit_common_terminal_sizes() {
        let donut = Donut::new(91);
        for (width, height) in [(79, 24), (119, 30), (39, 12)] {
            let scale = fitting_scale(width, height, DONUT_BOUNDING_RADIUS, 0.88);
            for seconds in [0.0, 2.5, 8.884, 31.0] {
                let angle_x = seconds * donut.speed_x + 0.7;
                let angle_z = seconds * donut.speed_z;
                for &(point, _) in &donut.samples {
                    assert_on_screen(
                        point.rotate_x(angle_x).rotate_z(angle_z),
                        width,
                        height,
                        scale,
                    );
                }
            }
        }
    }

    #[test]
    fn wireframe_vertices_fit_common_terminal_sizes() {
        for shape in [Shape::Cube, Shape::Octahedron] {
            let wireframe = Wireframe {
                shape,
                speed: Vec3::new(0.31, 0.43, -0.17),
                phase: Vec3::new(0.2, 0.7, 0.4),
                colors: [Rgb::default(), Rgb::default()],
            };
            let (vertices, _) = wireframe.geometry();
            for (width, height) in [(79, 24), (119, 30), (39, 12)] {
                let scale = fitting_scale(width, height, wireframe.bounding_radius(), 0.84);
                for seconds in [0.0, 2.5, 8.884, 31.0] {
                    let angles = wireframe.speed * seconds + wireframe.phase;
                    for &vertex in vertices {
                        assert_on_screen(
                            vertex.rotate(angles.x, angles.y, angles.z),
                            width,
                            height,
                            scale,
                        );
                    }
                }
            }
        }
    }
}

use std::ops::*;
use skia_safe::*;

#[derive(Clone, Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Sub for &Vertex {
    type Output = Vertex;

    fn sub(self, rhs: Self) -> Self::Output {
        Vertex { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Div<f32> for &Vertex {
    type Output = Vertex;

    fn div(self, rhs: f32) -> Self::Output {
        Vertex { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl Mul for Vertex {
    type Output = Vertex;

    fn mul(self, rhs: Self) -> Self::Output {
        Vertex {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl Into<Point> for &Vertex {
    fn into(self) -> Point {
        Point::new(self.x as scalar, self.y as scalar)
    }
}

impl Vertex {
    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalise(&self) -> Self {
        self / self.length()
    }
}

#[derive(Clone, Debug)]
struct Triangle {
    vertices: [Vertex; 3],
}

impl Triangle {
    fn centre(&self) -> Vertex {
        let mut x = 0f32;
        let mut y = 0f32;
        let mut z = 0f32;
        for vertex in &self.vertices {
            x += vertex.x;
            y += vertex.y;
            z += vertex.z;
        }

        return Vertex {
            x: x / 3f32,
            y: y / 3f32,
            z: z / 3f32,
        };
    }

    fn normal(&self) -> Vertex {
        let v1 = &self.vertices[0] - &self.vertices[1];
        let v2 = &self.vertices[0] - &self.vertices[2];
        let v3 = v1 * v2;

        return v3.normalise();
    }
}

#[derive(Clone, Debug)]
pub struct LowPoly {
    pub variance: f32,
    pub cell_size: usize,
    pub depth: u8,
    pub dither: u8,
    pub seed: u64,
}

impl LowPoly {
    fn random(&mut self) -> f32 {
        self.seed = self.seed * 16807 % 2147483647;
        if self.seed <= 0 {
            self.seed += 2147483646;
        }
        (self.seed - 1) as f32 / 2147483646.0
    }

    fn generate_points(&mut self, columns: usize, rows: usize) -> Vec<Vertex> {
        let mut vec = Vec::with_capacity(rows * columns);
        let cell_size = self.cell_size as f32;
        let variance = self.variance as f32;
        let depth = self.depth as f32;

        for row in 0..rows {
            for column in 0..columns {
                let x = if row % 2 == 0 {
                    column as f32 * cell_size - cell_size
                } else {
                    column as f32 * cell_size - cell_size / 2.0
                } + (self.random() - 0.5) * variance * cell_size * 2f32;

                let y = row as f32 * cell_size * 0.866 - cell_size +
                    (self.random() - 0.5) * variance * cell_size * 2f32;

                let z = self.random() * depth * cell_size * 50f32;

                let vertex = Vertex { x, y, z };

                vec.push(vertex);
            }
        }

        vec
    }

    fn generate_triangles(&self, columns: usize, rows: usize, points: &Vec<Vertex>) -> Vec<Triangle> {
        let mut vec = Vec::with_capacity(rows * columns);
        let mut i = 0;

        for row in 0..(rows - 1) {
            for _ in 0..(columns - 1) {
                if row % 2 == 0 {
                    vec.push(Triangle {
                        vertices: [
                            points.get(i).cloned().unwrap(),
                            points.get(i + 1).cloned().unwrap(),
                            points.get(i + columns).cloned().unwrap(),
                        ]
                    });
                    vec.push(Triangle {
                        vertices: [
                            points.get(i + 1).cloned().unwrap(),
                            points.get(i + columns + 1).cloned().unwrap(),
                            points.get(i + columns).cloned().unwrap(),
                        ]
                    });
                } else {
                    vec.push(Triangle {
                        vertices: [
                            points.get(i).cloned().unwrap(),
                            points.get(i + columns + 1).cloned().unwrap(),
                            points.get(i + columns).cloned().unwrap(),
                        ]
                    });
                    vec.push(Triangle {
                        vertices: [
                            points.get(i).cloned().unwrap(),
                            points.get(i + 1).cloned().unwrap(),
                            points.get(i + columns + 1).cloned().unwrap(),
                        ]
                    });
                }

                i += 1;
            }
            i += 1;
        }

        vec
    }

    fn draw_poly(&mut self, bitmap: &Bitmap, canvas: &mut Canvas, triangle: &Triangle) {
        let dither = self.dither as f32;
        let mut centre = triangle.centre();

        let dither_x = (dither / 200f32) * bitmap.width() as f32;
        let dither_y = (dither / 200f32) * bitmap.height() as f32;

        centre.x += self.random() * dither_x - dither_x / 2f32;
        centre.y += self.random() * dither_y - dither_y / 2f32;

        centre.x = centre.x.clamp(0f32, (bitmap.width() - 1) as f32);
        centre.y = centre.y.clamp(0f32, (bitmap.height() - 1) as f32);

        let point = IPoint::new(centre.x as _, centre.y as _);
        let color = bitmap.get_color(point);
        let mut color4f = Color4f::from(color);

        let normal = triangle.normal();
        let light = Vertex { x: 0.5, y: 0.5, z: 1.5 }.normalise();
        let dot = (normal.x * light.x + normal.y * light.y + normal.z * light.z)
            .add(1f32).div(2f32);

        let shadow = (1.1 * dot).min(1f32);
        let specular = (1.0 - 1.5 * dot).max(0f32);

        color4f.r = (1f32 - (1f32 - color4f.r) * (1f32 - specular)) * shadow;
        color4f.g = (1f32 - (1f32 - color4f.g) * (1f32 - specular)) * shadow;
        color4f.b = (1f32 - (1f32 - color4f.b) * (1f32 - specular)) * shadow;

        let paint = Paint::new(&color4f, &bitmap.color_space());

        Self::draw_triangle(canvas, &triangle.vertices, &paint);
    }

    fn draw_triangle(canvas: &mut Canvas, vertices: &[Vertex; 3], paint: &Paint) {
        let mut path = Path::new();
        path.move_to(&vertices[0]);
        path.line_to(&vertices[1]);
        path.line_to(&vertices[2]);
        canvas.draw_path(&path, paint);
    }

    pub fn render(&mut self, bitmap: &Bitmap) -> Surface {
        let cell_size = self.cell_size as f32;

        let grid_width = bitmap.width() as f32 + cell_size * 2f32;
        let grid_height = bitmap.height() as f32 + cell_size * 2f32;

        let columns = (grid_width / cell_size).ceil() as usize + 2;
        let rows = (grid_height / (cell_size * 0.865f32)).ceil() as usize;

        let points = self.generate_points(columns, rows);
        let triangles = self.generate_triangles(columns, rows, &points);

        let mut surface = Surface::new_raster_n32_premul(bitmap.dimensions())
            .expect("create surface failure!");

        for triangle in &triangles {
            self.draw_poly(&bitmap, surface.canvas(), triangle)
        }

        surface
    }
}

#[test]
fn render() {
    let bytes = std::fs::read("./run/test.jpg").unwrap();
    let data = Data::new_copy(bytes.as_slice());
    let image = Image::from_encoded(data).unwrap();
    let mut bitmap = Bitmap::new();
    let mut temp = Surface::new_raster_n32_premul(image.dimensions()).unwrap();
    temp.canvas().draw_image(image, (0, 0), None);
    let _ = temp.canvas().read_pixels_to_bitmap(&mut bitmap, (0, 0));

    let mut lp = LowPoly {
        variance: 0.30,
        cell_size: 40 * 3 + 30,
        depth: 20,
        dither: 10,
        seed: 0,
    };

    let mut result = lp.render(&bitmap);
    let encoded = result.image_snapshot()
        .encode_to_data(EncodedImageFormat::PNG)
        .unwrap();

    std::fs::write("./run/test.png", encoded.as_bytes()).unwrap();
}

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use euclid::{default::Transform2D, point2};
use wpf_gpu_raster::{PathBuilder};


use std::{convert::TryInto, ops::{Index, Mul, Sub, Div, IndexMut, AddAssign}};
#[derive(Clone, Copy, Debug)]
struct vec<T, const M: usize> {
    data_: [T; M]
}

impl<T: Default, const N: usize> Default for vec<T, N> where [T; N]: Default {
    fn default() -> Self {
        Self { data_: Default::default() }
    }
}
impl<T: Copy, const N: usize> vec<T, N> {
    fn new(data: &[T]) -> Self {
        vec { data_: data.try_into().unwrap() }
    }
}

impl<T, const N: usize> Index<usize> for vec<T, N>  {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data_[index]

    }
}

impl<T, const N: usize> IndexMut<usize> for vec<T, N>  {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data_[index]
    }
}

impl<T, const N: usize> Mul for vec<T, N> where T: Copy + Default + Mul + AddAssign<<T as Mul>::Output>  {
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Default::default();
        for i in 0..N {
            result += self[i] * rhs[i]
        }
        result
    }
}

impl<T: Div<Output = T> + Clone + Copy, const N: usize> Div<T> for vec<T, N>  {
    type Output = vec<T, N>;

    fn div(self, rhs: T) -> Self::Output {
        let mut result = self;
        for i in 0..N {
            result[i] = result[i] / rhs
        }
        result
    }
}

type Vec2f = vec<f32, 2>;
type Vec2i = vec<i32, 2>;
type Vec3f = vec<f32, 3>;
type Vec4f = vec<f32, 4>;

impl From<Vec2i> for Vec2f {
    fn from(v: Vec2i) -> Self {
        Vec2f::new(&[v[0] as f32, v[1] as f32])
    }
}

trait Unit {
    fn unit() -> Self;
}

impl Unit for f32 {
    fn unit() -> Self { 1. }
}

fn embed<T: Unit + Default + Copy, const DIM: usize, const LEN: usize>(v: &vec<T, DIM>) ->  vec<T, LEN> where [T; LEN]: Default {
    let fill = T::unit();
    let mut ret: vec<T, LEN> = Default::default();
    for i in (0..LEN).rev() {
        ret[i] = if i < DIM {
            v[i]
        } else {
            fill
        }
    }
    ret
}

fn proj<T: Unit + Default + Copy, const LEN: usize, const DIM: usize>(v: &vec<T, DIM>) ->  vec<T, LEN> where [T; LEN]: Default {
    let _fill = 1;
    let mut ret: vec<T, LEN> = Default::default();
    for i in (0..LEN).rev() {
        ret[i] = v[i];
    }
    ret
}

fn cross<T>(v1: vec<T, 3>, v2: vec<T, 3>) -> vec<T, 3> where T: Copy + Mul<Output = T> + Sub<Output = T> {
    vec::<T, 3>::new(&[v1[1]*v2[2] - v1[2]*v2[1], v1[2]*v2[0] - v1[0]*v2[2], v1[0]*v2[1] - v1[1]*v2[0]])
}




struct Model {
    vertices: Vec<Vec3f>,
    colors: Vec<Vec3f>,
    faces: Vec<Vec<usize>>
}
impl Model {
    fn new() -> Self { Self { vertices: Vec::new(), colors: Vec::new(), faces: Vec::new() } }
    fn nfaces(&self) -> i32 { self.faces.len() as i32 }
    fn vert(&self, iface: i32, nthvert: i32) -> Vec3f { self.vertices[self.faces[iface as usize][nthvert as usize] as usize] }
    fn color(&self, iface: i32, nthvert: i32) -> Vec3f { self.colors[self.faces[iface as usize][nthvert as usize] as usize] }

}
const  WIDTH: u32  = 800;
const HEIGHT: u32  = 800;

#[derive(Clone, Copy)]
struct TGAColor(u8, u8, u8, u8);


impl TGAColor {
    fn new(a: u8, r: u8, g: u8, b: u8) -> Self { TGAColor(a, r, g, b) }
}

impl Mul<f32> for TGAColor {
    type Output = TGAColor;

    fn mul(self, rhs: f32) -> Self::Output {
        TGAColor((self.0 as f32 * rhs) as u8, (self.1 as f32 * rhs) as u8, (self.2 as f32 * rhs) as u8, (self.3 as f32 * rhs) as u8)
    }
}

fn over(src: u32, dst: u32) -> u32 {
    let a = src >> 24;
    let a = 255 - a;
    let mask = 0xff00ff;
    let t = (dst & mask) * a + 0x800080;
    let mut rb = (t + ((t >> 8) & mask)) >> 8;
    rb &= mask;

    rb += src & mask;

    // saturate
    rb |= 0x1000100 - ((rb >> 8) & mask);
    rb &= mask;

    let t = ((dst >> 8) & mask) * a + 0x800080;
    let mut ag = (t + ((t >> 8) & mask)) >> 8;
    ag &= mask;
    ag += (src >> 8) & mask;

    // saturate
    ag |= 0x1000100 - ((ag >> 8) & mask);
    ag &= mask;

    (ag << 8) + rb
}

struct TGAImage {
    buf: Vec<u8>,
    width: u32,
    height: u32
}

impl TGAImage {
    fn new(width: u32, height: u32) -> Self { Self { width, height, buf: vec![0; (width * height * 3) as usize] } }
    /*fn set(&mut self, x: i32, y: i32, color: TGAColor) {
        if x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.buf[((x + y * self.width as i32) * 3) as usize] = color.0;
        self.buf[((x + y * self.width as i32) * 3 + 1) as usize] = color.1;
        self.buf[((x + y * self.width as i32) * 3 + 2) as usize] = color.2;
    }*/
    fn blend(&mut self, x: i32, y: i32, color: TGAColor) {
        if x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = ((x + y * self.width as i32) * 3) as usize;
        let dst = 0xff << 24 |
                     (self.buf[index] as u32) << 16 |
                     (self.buf[index + 1] as u32) << 8 |
                     (self.buf[index + 2] as u32);
        let src = (color.0 as u32) << 24 |
                     (color.1 as u32) << 16 |
                     (color.2  as u32) << 8 |
                     color.3 as u32;
        let dst = over(src, dst);
        self.buf[index] = ((dst >> 16) & 0xff) as u8;
        self.buf[index + 1] = ((dst >> 8) & 0xff) as u8;
        self.buf[index + 2] = ((dst >> 0) & 0xff) as u8;
    }
    fn write(&self, path: &str) {
        use std::path::Path;
        use std::fs::File;
        use std::io::BufWriter;

        let path = Path::new(path);
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.buf).unwrap(); // Save
    }
}
struct Shader {
    coverage: Vec3f // written by vertex shader, read by fragment shader
}
impl Shader {
    fn new() -> Self { Self { coverage: Default::default() } }
    fn vertex(&mut self, model: &Model, iface: i32, nthvert: i32) -> Vec4f {
        let gl_Vertex = embed(&model.vert(iface, nthvert)); // read the vertex from .obj file
        self.coverage[nthvert as usize] = model.color(iface, nthvert)[0]; // read the color from the .obj file
        gl_Vertex
    }

    fn fragment(&self, bar: Vec3f, color: TGAColor) -> TGAColor {
        let intensity = self.coverage*bar;   // interpolate intensity for the current pixel
        color*intensity // well duh
    }
}

fn barycentric(A: Vec2f, B: Vec2f, C: Vec2f, P: Vec2f) -> Vec3f {
    let mut s: [Vec3f; 2] = Default::default();
    for i in (0..2).rev() {
        s[i][0] = C[i]-A[i];
        s[i][1] = B[i]-A[i];
        s[i][2] = A[i]-P[i];
    }
    let u = cross(s[0], s[1]);
    if u[2].abs()>1e-2 {// dont forget that u[2] is integer. If it is zero then triangle ABC is degenerate
        return Vec3f::new(&[1.-(u[0]+u[1])/u[2], u[1]/u[2], u[0]/u[2]]);
    }
    Vec3f::new(&[-1.,1.,1.]) // in this case generate negative coordinates, it will be thrown away by the rasterizator
}

fn triangle(pts: &[Vec4f], shader: &Shader, image: &mut TGAImage, color: TGAColor) {
    let mut bboxmin = Vec2f::new( &[f32::MAX,  f32::MAX]);
    let mut bboxmax = Vec2f::new(&[-f32::MAX, -f32::MAX]);
    for i in 0..3 {
        for j in 0..2 {
            bboxmin[j] = bboxmin[j].min(pts[i][j]/pts[i][3]);
            bboxmax[j] = bboxmax[j].max(pts[i][j]/pts[i][3]);
        }
    }
    for x in (bboxmin[0] as i32)..=(bboxmax[0] as i32) {
        for y in (bboxmin[1] as i32)..=(bboxmax[1] as i32) {
            let P = Vec2i::new(&[x, y]);
            let c = barycentric(proj(&(pts[0]/pts[0][3])), proj(&(pts[1]/pts[1][3])), proj(&(pts[2]/pts[2][3])), P.into());
            if c[0]<0. || c[1]<0. || c[2]<0. { continue };
            let color = shader.fragment(c, color);
            image.blend(P[0], P[1], color);
        }
    }
}

fn main() {
    let opt = usvg::Options::default();

    let rtree = usvg::Tree::from_file("tiger.svg", &opt).unwrap();

    let mut image = TGAImage::new(WIDTH, HEIGHT);
    for _ in 0..1 {
    let mut total_vertex_count = 0;
    let mut total_time = std::time::Duration::default();
    for node in rtree.root().descendants() {
        use usvg::NodeExt;
        let t = node.transform();
        let transform = Transform2D::new(
            t.a as f32, t.b as f32,
            t.c as f32, t.d as f32,
            t.e as f32, t.f as f32,
        );


        let s = 1.;
        if let usvg::NodeKind::Path(ref usvg_path) = *node.borrow() {
            let color = match usvg_path.fill {
                Some(ref fill) => {
                    match fill.paint {
                        usvg::Paint::Color(c) => TGAColor::new(255, c.red, c.green, c.blue),
                        _ => TGAColor::new(255, 0, 255, 0),
                    }
                }
                None => {
                    continue;
                }
            };
            let mut builder = PathBuilder::new();
            //dbg!(&usvg_path.segments);
            for segment in &usvg_path.segments {
                match *segment {
                    usvg::PathSegment::MoveTo { x, y } => {
                        let p = transform.transform_point(point2(x as f32, y as f32)) * s;
                        builder.move_to(p.x, p.y);
                    }
                    usvg::PathSegment::LineTo { x, y } => {
                        let p = transform.transform_point(point2(x as f32, y as f32)) * s;
                        builder.line_to(p.x, p.y);
                    }
                    usvg::PathSegment::CurveTo { x1, y1, x2, y2, x, y, } => {
                        let c1 = transform.transform_point(point2(x1 as f32, y1 as f32)) * s;
                        let c2 = transform.transform_point(point2(x2 as f32, y2 as f32)) * s;
                        let p = transform.transform_point(point2(x as f32, y as f32)) * s;
                        builder.curve_to(
                            c1.x, c1.y,
                            c2.x, c2.y,
                            p.x, p.y,
                        );
                    }
                    usvg::PathSegment::ClosePath => {
                        builder.close();
                    }
                }
            }
            let start = std::time::Instant::now();
            let result = builder.rasterize_to_tri_strip(0, 0, WIDTH as i32, HEIGHT as i32);
            let end = std::time::Instant::now();
            total_time += end - start;

            println!("vertices {}", result.len());
            total_vertex_count += result.len();
            if result.len() == 0 {
                continue;
            }
            let mut model = Model::new();
        
            for vertex in result.iter() {
                model.vertices.push(Vec3f::new(&[vertex.x - 0.5, vertex.y - 0.5, 0.]));
                let color = vertex.coverage;
                model.colors.push(Vec3f::new(&[color, color, color]));
            }
            for n in 0..result.len()-2 {
                if n % 2 == 0 {
                    model.faces.push(vec![n, n+1, n+2]);
                } else {
                    model.faces.push(vec![n+1, n, n+2]);
                }
            }
        
            let mut shader = Shader::new();
            for i in 0..model.nfaces() {
                let mut screen_coords: [Vec4f; 3] = Default::default();
                for j in 0..3 {
                    screen_coords[j as usize] = shader.vertex(&model, i, j);
                
                }
                triangle(&screen_coords, &shader, &mut image, color);
            }
        }
    }

    println!("total vertex count {}, took {}ms", total_vertex_count, total_time.as_secs_f32()*1000.);
    }


    image.write("out.png");
    use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};
    use crate::*;
    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    assert_eq!(calculate_hash(&image.buf),
        if cfg!(debug_assertions) { 0xb03d06a10a8353a1 } else { 0x4351654ca83459c0});


}

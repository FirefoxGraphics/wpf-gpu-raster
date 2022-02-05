use euclid::{default::Transform2D, point2};
use wpf_gpu_raster::PathBuilder;


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
        let mut result = self.clone();
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
    fn unit() -> Self { return 1. }
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
    return ret;
}

fn proj<T: Unit + Default + Copy, const LEN: usize, const DIM: usize>(v: &vec<T, DIM>) ->  vec<T, LEN> where [T; LEN]: Default {
    let fill = 1;
    let mut ret: vec<T, LEN> = Default::default();
    for i in (0..LEN).rev() {
        ret[i] = v[i];
    }
    return ret;
}

fn cross<T>(v1: vec<T, 3>, v2: vec<T, 3>) -> vec<T, 3> where T: Copy + Mul<Output = T> + Sub<Output = T> {
    return vec::<T, 3>::new(&[v1[1]*v2[2] - v1[2]*v2[1], v1[2]*v2[0] - v1[0]*v2[2], v1[0]*v2[1] - v1[1]*v2[0]]);
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

struct TGAColor(u8, u8, u8);


impl TGAColor {
    fn new(r: u8, g: u8, b: u8) -> Self { TGAColor(r, g, b) }
}

impl Mul<f32> for TGAColor {
    type Output = TGAColor;

    fn mul(self, rhs: f32) -> Self::Output {
        TGAColor((self.0 as f32 * rhs) as u8, (self.1 as f32 * rhs) as u8, (self.2 as f32 * rhs) as u8)
    }
}

struct TGAImage {
    buf: Vec<u8>,
    width: u32,
    height: u32
}

impl TGAImage {
    fn new(width: u32, height: u32) -> Self { Self { width, height, buf: vec![0; (width * height * 3) as usize] } }
    fn set(&mut self, x: i32, y: i32, color: TGAColor) {
        self.buf[((x + y * self.width as i32) * 3) as usize] = color.0;
        self.buf[((x + y * self.width as i32) * 3 + 1) as usize] = color.1;
        self.buf[((x + y * self.width as i32) * 3 + 2) as usize] = color.2;
    }
    fn write(&self, path: &str) {
        use std::path::Path;
        use std::fs::File;
        use std::io::BufWriter;

        let path = Path::new(path);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

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
        let gl_Vertex = embed::<_, 3, 4>(&model.vert(iface, nthvert)); // read the vertex from .obj file
        self.coverage[nthvert as usize] = model.color(iface, nthvert)[0]; // read the color from the .obj file
        return gl_Vertex;
    }

    fn fragment(&self, bar: Vec3f) -> TGAColor {
        let intensity = self.coverage*bar;   // interpolate intensity for the current pixel
        return TGAColor::new(255, 255, 255)*intensity; // well duh
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
    if (u[2].abs()>1e-2) {// dont forget that u[2] is integer. If it is zero then triangle ABC is degenerate
        return Vec3f::new(&[1.-(u[0]+u[1])/u[2], u[1]/u[2], u[0]/u[2]]);
    }
    return Vec3f::new(&[-1.,1.,1.]); // in this case generate negative coordinates, it will be thrown away by the rasterizator
}

fn triangle(pts: &[Vec4f], shader: &Shader, image: &mut TGAImage) {
    let mut bboxmin = Vec2f::new( &[f32::MAX,  f32::MAX]);
    let mut bboxmax = Vec2f::new(&[-f32::MAX, -f32::MAX]);
    for i in 0..3 {
        for j in 0..2 {
            bboxmin[j] = bboxmin[j].min(pts[i][j]/pts[i][3]);
            bboxmax[j] = bboxmax[j].max(pts[i][j]/pts[i][3]);
        }
    }
    let mut P: Vec2i = Default::default();
    for x in (bboxmin[0] as i32)..=(bboxmax[0] as i32) {
        for y in (bboxmin[1] as i32)..=(bboxmax[1] as i32) {
            P[0] = x;
            P[1] = y;
            let c = barycentric(proj::<_, 2, 4>(&(pts[0]/pts[0][3])), proj::<_, 2, 4>(&(pts[1]/pts[1][3])), proj::<_, 2, 4>(&(pts[2]/pts[2][3])), proj::<_, 2, 2>(&P.into()));
            if (c[0]<0. || c[1]<0. || c[2]<0.) { continue };
            let color = shader.fragment(c);
            image.set(P[0], P[1], color);
        }
    }
}
/* 
int main(int argc, char** argv) {
    if (2==argc) {
        model = new Model(argv[1]);
    } else {
        std::cerr << "Missing obj file\n";
        return -1;
    }

    TGAImage image(width, height, TGAImage::RGB);

    Shader shader;
    for (int i=0; i<model->nfaces(); i++) {
        Vec4f screen_coords[3];
        for (int j=0; j<3; j++) {
            screen_coords[j] = shader.vertex(i, j);
        }
        triangle(screen_coords, shader, image);
    }

    image.flip_vertically(); // to place the origin in the bottom left corner of the image
    image.write_tga_file("output.tga");

    delete model;
    return 0;
}

void output_obj_file(OutputVertex *data, size_t len) {
    for (size_t i = 0; i < len; i++) {
            float color = data[i].coverage;
            printf("v %f %f %f %f %f %f\n", data[i].x, data[i].y, 0., color, color, color);
    }

    // output a standard triangle strip face list
    for (int n = 1; n < len-1; n++) {
            if (n % 2 == 1) {
                    printf("f %d %d %d\n", n, n+1, n+2);
            } else {
                    printf("f %d %d %d\n", n+1, n, n+2);
            }
    }

}*/

fn main() {
    let opt = usvg::Options::default();
    let mut builder = PathBuilder::new();

    let rtree = usvg::Tree::from_file("tiger.svg", &opt).unwrap();
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
            dbg!(usvg_path);
            break;
        }
    }

    let result = builder.rasterize_to_tri_strip(WIDTH as i32, HEIGHT as i32);
    let mut model = Model::new();

    for vertex in result.iter() {
        model.vertices.push(Vec3f::new(&[vertex.x, vertex.y, 0.]));
        let color = vertex.coverage;
        model.colors.push(Vec3f::new(&[color, color, color]));
    }
    for n in 0..result.len()-2 {
        if (n % 2 == 0) {
            model.faces.push(vec![n, n+1, n+2]);
        } else {
            model.faces.push(vec![n+1, n, n+2]);
        }
    }


    let mut image = TGAImage::new(WIDTH, HEIGHT);

    let mut shader = Shader::new();
    for i in 0..model.nfaces() {
        let mut screen_coords: [Vec4f; 3] = Default::default();
        for j in 0..3 {
            screen_coords[j as usize] = shader.vertex(&model, i, j);
        
        }
        triangle(&screen_coords, &shader, &mut image);
    }
    image.write("out.png");

}

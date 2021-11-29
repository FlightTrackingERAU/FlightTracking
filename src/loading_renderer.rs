use std::io::Cursor;

use glium::{
    implement_vertex, index::NoIndices, texture::SrgbTexture2d, uniform, DrawParameters, Program,
    Surface,
};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub angle: f32,
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, angle, tex_coords);

pub struct LoadingScreenRenderer<'a> {
    pub program: Program,
    pub draw_parameters: DrawParameters<'a>,
    pub texture: SrgbTexture2d,
    pub indices: NoIndices,
    pub logo_angle: f32,
    pub logo_angle_delta: f32,
}

impl<'a> LoadingScreenRenderer<'a> {
    /// Creates a new LoadingScreenRenderer
    pub fn new(display: &glium::Display) -> Self {
        let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            in float angle;
            in vec2 tex_coords;

            out vec2 v_tex_coords;

            uniform mat4 matrix;
            uniform float dpi_factor;

            void main() {
                v_tex_coords = tex_coords;
                vec2 pos = position;
                vec2 new_position = vec2(pos.x * cos(angle) - pos.y * sin(angle), pos.x * sin(angle) + pos.y * cos(angle));
                vec4 scaled = matrix * vec4(new_position, 0.0, 1.0);
                gl_Position = scaled;
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec2 v_tex_coords;
            out vec4 color;

            uniform sampler2D tex;

            void main() {
                color = texture(tex, v_tex_coords);
            }
        "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        let image = image::load(
            Cursor::new(&include_bytes!("../assets/images/rust-logo.png")),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();

        let image_dimensions = image.dimensions();

        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let draw_parameters = glium::draw_parameters::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..glium::draw_parameters::DrawParameters::default()
        };

        Self {
            program,
            draw_parameters,
            texture,
            indices,
            logo_angle: 0.0,
            logo_angle_delta: 0.0008,
        }
    }

    /// Draw the rotating logo on the OpenGL Frame that is provided
    pub fn draw(&mut self, display: &glium::Display, target: &mut glium::Frame) {
        // Here we collect the dynamic numbers for rendering our OpenGL planes
        let (width, height) = target.get_dimensions();
        let width = width as f32;
        let height = height as f32;
        let dpi_factor = display.gl_window().window().scale_factor() as f32;
        let size_of_logo = height / 3.0;

        let vertices = gen_square(self.logo_angle);

        let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();

        let aspect_ratio = height as f32 / width as f32;
        let scale_factor = (size_of_logo / height as f32) * dpi_factor;
        let matrix: [[f32; 4]; 4] =
            cgmath::Matrix4::from_nonuniform_scale(aspect_ratio * scale_factor, scale_factor, 1.0)
                .into();

        let uniforms = uniform! {
            matrix: matrix,
            tex: &self.texture,
        };

        target
            .draw(
                &vertex_buffer,
                &self.indices,
                &self.program,
                &uniforms,
                &self.draw_parameters,
            )
            .unwrap();

        self.logo_angle += self.logo_angle_delta;
        self.logo_angle_delta += self.logo_angle_delta * 0.002;

        if self.logo_angle > std::f32::consts::TAU {
            self.logo_angle = self.logo_angle - std::f32::consts::TAU;
        }
    }
}

/// Generates a set of vertices that describe a textured square that must be drawn
pub fn gen_square(angle: f32) -> [Vertex; 6] {
    let vertex1 = Vertex {
        position: [-1.0, 1.0],
        angle,
        tex_coords: [0.0, 1.0],
    };
    let vertex2 = Vertex {
        position: [1.0, 1.0],
        angle,
        tex_coords: [1.0, 1.0],
    };
    let vertex3 = Vertex {
        position: [1.0, -1.0],
        angle,
        tex_coords: [1.0, 0.0],
    };
    let vertex4 = Vertex {
        position: [-1.0, -1.0],
        angle,
        tex_coords: [0.0, 0.0],
    };

    [vertex1, vertex2, vertex3, vertex4, vertex3, vertex1]
}

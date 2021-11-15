use std::io::Cursor;

use glium::{
    implement_vertex, index::NoIndices, texture::SrgbTexture2d, uniform, DrawParameters, Program,
    Surface,
};

use crate::{util, PlaneRequester};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub angle: f32,
    pub offset: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, angle, offset, tex_coords); // don't forget to add `tex_coords` here

pub struct PlaneRenderer<'a> {
    pub program: Program,
    pub draw_parameters: DrawParameters<'a>,
    pub vertices: Vec<Vertex>,
    pub texture: SrgbTexture2d,
    pub indices: NoIndices,
}

impl<'a> PlaneRenderer<'a> {
    /// Creates a new PlaneRenderer
    pub fn new(display: &glium::Display) -> Self {
        let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            in float angle;
            in vec2 offset;
            in vec2 tex_coords;

            out vec2 v_tex_coords;

            uniform mat4 matrix;
            uniform float dpi_factor;

            void main() {
                v_tex_coords = tex_coords;
                vec2 pos = position;
                vec2 new_position = vec2(pos.x * cos(angle) - pos.y * sin(angle), pos.x * sin(angle) + pos.y * cos(angle));
                vec4 scaled = matrix * vec4(new_position, 0.0, 1.0);
                vec4 with_offset = vec4(offset * dpi_factor, 0.0, 0.0) + scaled;
                gl_Position = with_offset;
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
            Cursor::new(&include_bytes!("../assets/images/airplane-image.png")),
            image::ImageFormat::PNG,
        )
        .unwrap()
        .to_rgba();

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
            vertices: Vec::new(),
            texture,
            indices,
        }
    }

    /// Draw the planes on the OpenGL Frame that is provided
    pub fn draw(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        plane_requester: &mut PlaneRequester,
        view: &crate::TileView,
    ) {
        // Here we collect the dynamic numbers for rendering our OpenGL planes
        let (width, height) = target.get_dimensions();
        let width = width as f32;
        let height = height as f32;
        let dpi_factor = display.gl_window().window().scale_factor() as f32;

        // From PlaneRequester gets all the planes we get from the Mutex
        let planes = plane_requester.planes_storage();

        // Viewport of the world
        let viewport = view.get_world_viewport(width as f64, height as f64);
        let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0)) as f32;
        let lat_bottom =
            crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0)) as f32;
        let long_left = crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)) as f32;
        let long_right =
            crate::util::longitude_from_x(viewport.bottom_right.x.rem_euclid(1.0)) as f32;

        self.vertices.clear();

        // We iterate through all the planes and generated their OpenGL vertices
        for plane in planes.iter() {
            if (plane.latitude > lat_bottom && plane.latitude < lat_top)
                && (plane.longitude > long_left && plane.longitude < long_right)
            {
                // Translates real world coordinates to window coordinates.
                let world_x = util::x_from_longitude(plane.longitude as f64);
                let world_y = util::y_from_latitude(plane.latitude as f64);

                let offset_x = world_x_to_window_x(world_x, &viewport);
                let offset_y = world_y_to_window_y(world_y, &viewport);

                let offset = [offset_x, offset_y];

                // Generate the vertices
                let plane = plane_shape(plane.track, offset);

                for vertex in plane {
                    self.vertices.push(vertex);
                }
            }
        }

        let vertex_buffer = glium::VertexBuffer::new(display, &self.vertices).unwrap();

        let aspect_ratio = height as f32 / width as f32;
        let scale_factor = (50.0 / height as f32) * dpi_factor;

        let matrix: [[f32; 4]; 4] =
            cgmath::Matrix4::from_nonuniform_scale(aspect_ratio * scale_factor, scale_factor, 1.0)
                .into();

        let uniforms = uniform! {
            matrix: matrix,
            tex: &self.texture,
            dpi_factor: dpi_factor
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
    }
}

/// Projects a x world location combined with a viewport to determine the x location in the OpenGL
/// coordinate system
pub fn world_x_to_window_x(world_x: f64, viewport: &crate::map::WorldViewport) -> f32 {
    crate::util::map(
        viewport.top_left.x,
        viewport.bottom_right.x,
        world_x,
        -1.0,
        1.0,
    ) as f32
}

/// Projects a y world location combined with a viewport to determine the y location in the OpenGL
/// coordinate system
pub fn world_y_to_window_y(world_y: f64, viewport: &crate::map::WorldViewport) -> f32 {
    crate::util::map(
        viewport.top_left.y,
        viewport.bottom_right.y,
        world_y,
        1.0,
        -1.0,
    ) as f32
}

fn plane_shape(angle: f32, offset: [f32; 2]) -> [Vertex; 6] {
    let vertex1 = Vertex {
        position: [-1.0, 1.0],
        angle,
        offset,
        tex_coords: [0.0, 1.0],
    };
    let vertex2 = Vertex {
        position: [1.0, 1.0],
        angle,
        offset,
        tex_coords: [1.0, 1.0],
    };
    let vertex3 = Vertex {
        position: [1.0, -1.0],
        angle,
        offset,
        tex_coords: [1.0, 0.0],
    };
    let vertex4 = Vertex {
        position: [-1.0, -1.0],
        angle,
        offset,
        tex_coords: [0.0, 0.0],
    };

    [vertex1, vertex2, vertex3, vertex4, vertex3, vertex1]
}

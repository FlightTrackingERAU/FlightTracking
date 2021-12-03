use std::io::Cursor;

use enum_map::{enum_map, Enum, EnumMap};
use glam::DVec2;
use glium::{
    implement_vertex, index::NoIndices, texture::SrgbTexture2d, uniform, DrawParameters, Program,
    Surface,
};

use crate::{map, util, world_x_to_pixel_x, world_y_to_pixel_y, Plane, PlaneRequester};

///Normal body of plane we select
#[derive(Clone)]
pub struct SelectedPlane {
    pub plane: Plane,
    pub location: DVec2,
    pub size: f32,
}

impl SelectedPlane {
    pub fn new(plane: Plane, location: DVec2, size: f32) -> Self {
        SelectedPlane {
            plane,
            location,
            size,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Enum)]
pub enum PlaneType {
    Commercial,
    Cargo,
    Unknown,
}
impl PlaneType {
    pub fn to_str(self) -> &'static str {
        match self {
            PlaneType::Commercial => "Commercial",
            PlaneType::Cargo => "Cargo",
            PlaneType::Unknown => "Unkown",
        }
    }
}
/// Describes a few specific airlines, and also the selections of All or Other which the user can
/// filter by
#[derive(Copy, Clone, PartialEq, Eq, Enum)]
pub enum Airline {
    American,
    Spirit,
    Southwest,
    United,
    All,
    Other,
    Delta,
}

impl Airline {
    pub fn to_str(self) -> &'static str {
        match self {
            Airline::American => "American Airlines",
            Airline::Spirit => "Spirit Airlines",
            Airline::Southwest => "Southwest Airlines",
            Airline::United => "United Airlines",
            Airline::Delta => "Delta Airlines",
            _ => "Unknown",
        }
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub angle: f32,
    pub offset: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 3],
}

implement_vertex!(Vertex, position, angle, offset, tex_coords, color);

/// This struct renders the planes that are requested by the API and displays them using custom OpenGL
pub struct PlaneRenderer<'a> {
    pub program: Program,
    pub draw_parameters: DrawParameters<'a>,
    pub vertices: Vec<Vertex>,
    pub texture: SrgbTexture2d,
    pub indices: NoIndices,
    pub color_map: EnumMap<Airline, [f32; 3]>,
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
            in vec3 color;

            out vec2 v_tex_coords;
            out vec3 v_color;

            uniform mat4 matrix;
            uniform float dpi_factor;

            void main() {
                v_tex_coords = tex_coords;
                v_color = color;
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
            in vec3 v_color;
            out vec4 color;

            uniform sampler2D tex;

            void main() {
                float tex_alpha = texture(tex, v_tex_coords).a;
                color = vec4(v_color, tex_alpha);
            }
        "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        let image = image::load(
            Cursor::new(&include_bytes!("../assets/images/airplane-image.png")),
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

        let color_map = enum_map! {
            Airline::American => [3.0 / 255.0, 5.0 / 255.0, 135.0 / 255.0],
            Airline::Spirit => [1.0, 1.0, 0.0],
            Airline::United => [146.0 / 255.0, 182.0 / 255.0, 240.0 / 255.0],
            Airline::Southwest => [229.0 / 255.0, 29.0 / 255.0, 35.0 / 255.0],
            _ => [0.0, 0.0, 0.0]
        };

        Self {
            program,
            draw_parameters,
            vertices: Vec::new(),
            texture,
            indices,
            color_map,
        }
    }

    /// Draw the planes on the OpenGL Frame that is provided
    pub fn draw(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        plane_requester: &mut PlaneRequester,
        view: &crate::TileView,
        selected_airline: Airline,
        clicked_plane: &mut Option<SelectedPlane>,
        mut last_cursor_pos: Option<DVec2>,
    ) -> Option<SelectedPlane> {
        // Here we collect the dynamic numbers for rendering our OpenGL planes
        let (width, height) = target.get_dimensions();
        let width = width as f32;
        let height = height as f32;
        let dpi_factor = display.gl_window().window().scale_factor() as f32;

        // From PlaneRequester gets all the airlines and planes
        let airlines = plane_requester.planes_storage();

        // Viewport of the world
        let viewport = view.get_world_viewport(width as f64, height as f64);
        let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0)) as f32;
        let lat_bottom =
            crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0)) as f32;
        let long_left = crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)) as f32;
        let long_right =
            crate::util::longitude_from_x(viewport.bottom_right.x.rem_euclid(1.0)) as f32;
        let zoom = view.get_zoom() as f32;

        let size_of_plane = 1.5_f32.powf(zoom) / 30.0;
        if let Some(pos) = last_cursor_pos {
            let cursor_x = map(0.0, width as f64, pos.x, -1.0, 1.0) / dpi_factor as f64;
            let cursor_y = map(0.0, height as f64, pos.y, 1.0, -1.0) / dpi_factor as f64;

            last_cursor_pos = Some(DVec2::new(cursor_x, cursor_y));
        }

        let mut selected_plane = None;
        let closest_x = 0.01;
        let closest_y = 0.01;

        self.vertices.clear();

        let mut plane_position: DVec2 = DVec2::new(0.0, 0.0);

        // We iterate through all the planes and generated their OpenGL vertices
        for plane in airlines.iter() {
            let airline = plane.airline;
            if airline == selected_airline || selected_airline == Airline::All {
                let airline_color = self.color_map[airline];

                for plane in plane.planes.iter() {
                    if (plane.latitude > lat_bottom && plane.latitude < lat_top)
                        && (plane.longitude > long_left && plane.longitude < long_right)
                    {
                        // Translates real world coordinates to window coordinates.
                        let world_x = util::x_from_longitude(plane.longitude as f64);
                        let world_y = util::y_from_latitude(plane.latitude as f64);

                        let offset_x = world_x_to_window_x(world_x, &viewport);
                        let offset_y = world_y_to_window_y(world_y, &viewport);

                        let pixel_x = world_x_to_pixel_x(world_x, &viewport, width as f64);
                        let pixel_y = world_y_to_pixel_y(world_y, &viewport, height as f64);

                        let color = if let Some(last_cursor_pos) = last_cursor_pos {
                            if (offset_x - last_cursor_pos.x as f32).abs() < closest_x
                                && (offset_y - last_cursor_pos.y as f32).abs() < closest_y
                            {
                                //Gets the plane position as a DVec2
                                plane_position = DVec2::new(pixel_x, pixel_y);

                                selected_plane = Some(plane.clone());

                                // Draw it as white
                                [1.0, 1.0, 1.0]
                            } else {
                                airline_color
                            }
                        } else {
                            airline_color
                        };

                        //Show details about already clicked planes
                        if let Some(clicked_plane) = clicked_plane {
                            if clicked_plane.plane.callsign == plane.callsign {
                                if clicked_plane.plane.latitude != plane.latitude
                                    && clicked_plane.plane.longitude != plane.longitude
                                {
                                    //Updates the new plane data.
                                    clicked_plane.plane = plane.clone();
                                }
                            }
                        }

                        let offset = [offset_x, offset_y];

                        // Generate the vertices
                        let plane = plane_shape(plane.track, offset, color);

                        for vertex in plane {
                            self.vertices.push(vertex);
                        }
                    }
                }
            }
        }

        let vertex_buffer = glium::VertexBuffer::new(display, &self.vertices).unwrap();

        let aspect_ratio = height as f32 / width as f32;
        let scale_factor = (size_of_plane / height as f32) * dpi_factor;

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

        selected_plane.map(|plane| SelectedPlane::new(plane, plane_position, size_of_plane))
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

/// Generates a set of vertices that describe a single plane that must be drawn
pub fn plane_shape(angle: f32, offset: [f32; 2], color: [f32; 3]) -> [Vertex; 6] {
    let vertex1 = Vertex {
        position: [-1.0, 1.0],
        angle,
        offset,
        tex_coords: [0.0, 1.0],
        color,
    };
    let vertex2 = Vertex {
        position: [1.0, 1.0],
        angle,
        offset,
        tex_coords: [1.0, 1.0],
        color,
    };
    let vertex3 = Vertex {
        position: [1.0, -1.0],
        angle,
        offset,
        tex_coords: [1.0, 0.0],
        color,
    };
    let vertex4 = Vertex {
        position: [-1.0, -1.0],
        angle,
        offset,
        tex_coords: [0.0, 0.0],
        color,
    };

    [vertex1, vertex2, vertex3, vertex4, vertex3, vertex1]
}

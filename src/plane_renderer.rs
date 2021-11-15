use crate::{util, PlaneRequester, Vertex};

///Draw the plane on conrod ui
pub fn draw(
    plane_requester: &mut PlaneRequester,
    view: &crate::TileView,
    window_width: f64,
    window_height: f64,
    dpi_factor: f32,
    vertices: &mut Vec<Vertex>,
) {
    // From PlaneRequester gets all the planes we get from the Mutex
    let planes = plane_requester.planes_storage();

    // Viewport of the world
    let viewport = view.get_world_viewport(window_width, window_height);
    let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0)) as f32;
    let lat_bottom = crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0)) as f32;
    let long_left = crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)) as f32;
    let long_right = crate::util::longitude_from_x(viewport.bottom_right.x.rem_euclid(1.0)) as f32;

    vertices.clear();

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
            let plane = plane_shape(plane.track, offset, dpi_factor);

            for vertex in plane {
                vertices.push(vertex);
            }
        }
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

fn plane_shape(angle: f32, offset: [f32; 2], dpi_factor: f32) -> [Vertex; 6] {
    let vertex1 = Vertex {
        position: [-1.0, 1.0],
        angle,
        offset,
        dpi_factor,
        tex_coords: [0.0, 1.0],
    };
    let vertex2 = Vertex {
        position: [1.0, 1.0],
        angle,
        offset,
        dpi_factor,
        tex_coords: [1.0, 1.0],
    };
    let vertex3 = Vertex {
        position: [1.0, -1.0],
        angle,
        offset,
        dpi_factor,
        tex_coords: [1.0, 0.0],
    };
    let vertex4 = Vertex {
        position: [-1.0, -1.0],
        angle,
        offset,
        dpi_factor,
        tex_coords: [0.0, 0.0],
    };

    [vertex1, vertex2, vertex3, vertex4, vertex3, vertex1]
}

use std::time::Duration;

use conrod_core::{text::Font, widget, widget_ids, Colorable, Positionable, Sizeable, Widget};
//use conrod_core::{text::Font, widget, widget_ids, Colorable, Positionable, Sizeable, Widget};
use glam::DVec2;
use glium::Surface;

mod airports;
mod button_widget;
mod map;
mod map_renderer;
mod support;
mod tile;
mod ui_filter;
mod util;

pub use airports::*;
pub use button_widget::*;
pub use map::*;
pub use map_renderer::*;
pub use tile::*;
pub use ui_filter::*;
pub use util::*;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

widget_ids!(pub struct Ids {
    debug_menu[],
    text,
    viewport,
    map_images[],
    satellite_tiles[],
    tiles[],
    weather_tiles[],
    weather_button,
    airplane_button,
    debug_button,
    latitude_lines[],
    latitude_text[],
    longitude_lines[],
    longitude_text[],
    filer_button[],
    airports[]
});

pub use util::MAP_PERF_DATA;

/// The app's "main" function. Our real main inside `main.rs` calls this function
pub fn run_app() {
    // Create our UI's event loop
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Conrod Window")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &event_loop).unwrap();

    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate our widget identifiers
    let mut ids = Ids::new(ui.widget_id_generator());

    let mut image_map: conrod_core::image::Map<glium::Texture2d> = conrod_core::image::Map::new();

    //Making airplane image ids
    let airplane_image_bytes = include_bytes!("../assets/images/airplane-icon.png");
    let airplane_ids = return_image_essentials(&display, airplane_image_bytes, &mut image_map);

    let weather_image_bytes = include_bytes!("../assets/images/weather-icon.png");
    let weather_id = return_image_essentials(&display, weather_image_bytes, &mut image_map);

    let gear_icon_bytes = include_bytes!("../assets/images/gear-icon.png");
    let gear_id = return_image_essentials(&display, gear_icon_bytes, &mut image_map);

    let noto_sans_ttf = include_bytes!("../assets/fonts/NotoSans/NotoSans-Regular.ttf");
    let noto_sans = Font::from_bytes(noto_sans_ttf).expect("Failed to decode font");
    let _noto_sans = ui.fonts.insert(noto_sans);

    let b612_ttf = include_bytes!("../assets/fonts/B612Mono/B612Mono-Regular.ttf");
    let b612 = Font::from_bytes(b612_ttf).expect("Failed to decode font");
    let b612 = ui.fonts.insert(b612);

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let mut last_time = std::time::Instant::now();
    let mut frame_time_ms = 0.0;

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create Tokio runtime!");

    let mut pipelines = tile::pipelines(&runtime);

    let airports_bin = include_bytes!("../assets/data/airports.bin");
    let airports = airports_from_bytes(airports_bin).expect("Failed to load airports");

    let mut should_update_ui = true;
    let mut viewer = map::TileView::new(0.0, 0.0, 2.0, 1080.0 / 2.0);
    let mut last_cursor_pos: Option<DVec2> = None;
    let mut left_pressed = false;

    let mut weather_enabled = false;
    let mut debug_enabled = true;

    event_loop.run(move |event, _, control_flow| {
        use glium::glutin::event::{
            ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
        };
        // Break from the loop upon `Escape` or closed window.
        if let Event::WindowEvent { event, .. } = &event {
            match event {
                // Break from the loop upon `Escape`.
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        glium::glutin::event::KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
                WindowEvent::MouseWheel { delta, .. } => {
                    let zoom_change = match delta {
                        MouseScrollDelta::LineDelta(_x, y) => *y as f64,
                        MouseScrollDelta::PixelDelta(data) => data.y / 100.0,
                    };
                    let zoom_change = (-zoom_change / 6.0).clamp(-0.5, 0.5);
                    viewer.multiply_zoom(1.0 + zoom_change);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position = DVec2::new(position.x, position.y);
                    if let Some(last) = last_cursor_pos {
                        let delta = (last - position).clamp_length_max(300.0);
                        if left_pressed {
                            viewer.move_camera_pixels(delta);
                        }
                    }

                    last_cursor_pos = Some(position);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if matches!(button, MouseButton::Left) {
                        left_pressed = matches!(state, ElementState::Pressed);
                    }
                }
                _ => {}
            }
        }

        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = support::convert_event(&event, display.gl_window().window()) {
            ui.handle_event(event);
            should_update_ui = true;
        }

        match &event {
            glium::glutin::event::Event::MainEventsCleared => {
                if should_update_ui {
                    // should_update_ui = false;

                    // Set the widgets.
                    let mut ui = ui.set_widgets();
                    let ui = &mut ui;
                    ids.filer_button.resize(4, &mut ui.widget_id_generator());

                    //========== Draw Map ==========
                    {
                        let map_state = map_renderer::MapRendererState {
                            tile_cache: &mut pipelines,
                            view: &viewer,
                            display: &display,
                            image_map: &mut image_map,
                            ids: &mut ids,
                            weather_enabled,
                        };
                        map_renderer::draw(map_state, ui);
                    }

                    //========== Draw Airports ==========

                    airports::airport_renderer::draw(&airports, &viewer, &display, &mut ids, ui);

                    //========== Draw Debug Data ==========

                    let perf_data = crate::take_profile_data();

                    if debug_enabled {
                        let _scope_debug_view = crate::profile_scope("Render Debug Information");
                        let mut perf_data: Vec<_> = perf_data.into_iter().collect();
                        perf_data.sort_unstable_by(|a, b| a.0.cmp(b.0));

                        //========== Draw Debug Text ==========
                        let map_data = {
                            let mut guard = MAP_PERF_DATA.lock();
                            guard.snapshot()
                        };

                        let mut debug_text = vec![
                            format!(
                                "FT: {:.2}, FPS: {}",
                                frame_time_ms,
                                (1000.0 / frame_time_ms) as u32
                            ),
                            format!(
                                "Zoom: {}, Tiles: {}",
                                map_data.zoom, map_data.tiles_rendered
                            ),
                            format!(
                                "Decode: {:.2}ms, Upload: {:.2}ms",
                                map_data.tile_decode_time.as_secs_f64() * 1000.0,
                                map_data.tile_upload_time.as_secs_f64() * 1000.0
                            ),
                        ];
                        for (backend_name, time) in map_data.backend_request_secs {
                            debug_text.push(format!(
                                " {}: {:.2}ms",
                                backend_name,
                                time.as_secs_f64() * 1000.0
                            ));
                        }
                        for (name, data) in perf_data {
                            let samples = data.get_samples();
                            let text = if samples.len() == 1 {
                                format!("{}: {:?}", name, samples[0])
                            } else {
                                let avg: Duration =
                                    samples.iter().sum::<Duration>() / samples.len() as u32;
                                format!("{}: {} times, {:?} avg", name, samples.len(), avg)
                            };
                            debug_text.push(text);
                        }
                        ids.debug_menu
                            .resize(debug_text.len(), &mut ui.widget_id_generator());

                        for (i, text) in debug_text.iter().enumerate() {
                            let gui_text = widget::Text::new(text.as_str())
                                .color(conrod_core::color::WHITE)
                                .left_justify()
                                .font_size(8)
                                .font_id(b612);

                            let width = gui_text.get_w(ui).unwrap();
                            let x = -ui.win_w / 2.0 + width / 2.0 + 4.0;
                            let y = ui.win_h / 2.0 - 8.0 - i as f64 * 11.0;
                            gui_text.x_y(x, y).set(ids.debug_menu[i], ui);
                        }
                    }

                    //========== Draw Buttons ==========
                    let scope_render_buttons = crate::profile_scope("Render Buttons");

                    let widget_x_position = (ui.win_w / 2.0) * 0.95 - 25.0;
                    let widget_y_position = (ui.win_h / 2.0) * 0.90;

                    if button_widget::draw_circle_with_image(
                        ids.weather_button,
                        ui,
                        weather_id,
                        widget_x_position,
                        widget_y_position - 70.0,
                    ) {
                        weather_enabled = !weather_enabled;
                    }

                    if button_widget::draw_circle_with_image(
                        ids.debug_button,
                        ui,
                        gear_id,
                        widget_x_position,
                        widget_y_position - 140.0,
                    ) {
                        debug_enabled = !debug_enabled;
                    }

                    button_widget::draw_circle_with_image(
                        ids.airplane_button,
                        ui,
                        airplane_ids,
                        widget_x_position,
                        widget_y_position,
                    );

                    ui_filter::draw(
                        ids.filer_button[0],
                        ui,
                        String::from("American Airlanes"),
                        widget_x_position - 130.0,
                        widget_y_position,
                    );

                    ui_filter::draw(
                        ids.filer_button[1],
                        ui,
                        String::from("Spirit"),
                        widget_x_position - 130.0,
                        widget_y_position - 40.0,
                    );

                    ui_filter::draw(
                        ids.filer_button[2],
                        ui,
                        String::from("Southwest"),
                        widget_x_position - 130.0,
                        widget_y_position - 80.0,
                    );

                    ui_filter::draw(
                        ids.filer_button[3],
                        ui,
                        String::from("United"),
                        widget_x_position - 130.0,
                        widget_y_position - 120.0,
                    );
                    scope_render_buttons.end();

                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                //render and swap buffers
                let primitives = ui.draw();

                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.21, 0.32, 0.4, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();

                //Time calculations
                let now = std::time::Instant::now();
                frame_time_ms = (now - last_time).as_nanos() as f64 / 1_000_000.0;
                last_time = now;
            }
            _ => {}
        }
    })
}

//Function to return the Id for images
//Must convert image path to bytes
fn return_image_essentials(
    display: &glium::Display,
    bytes: &[u8],
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
) -> ImageId {
    let image_2d = load_image(display, bytes);

    ImageId {
        normal: image_map.insert(image_2d),
        hover: image_map.insert(load_image(display, bytes)),
        press: image_map.insert(load_image(display, bytes)),
    }
}
// Load an image from our assets folder as a texture we can draw to the screen.
fn load_image(display: &glium::Display, bytes: &[u8]) -> glium::texture::Texture2d {
    let rgba_image = image::load_from_memory(bytes).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &rgba_image.into_raw(),
        image_dimensions,
    );
    glium::texture::Texture2d::new(display, raw_image).unwrap()
}

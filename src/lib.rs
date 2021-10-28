use conrod_core::{
    text::Font, widget, widget_ids, Colorable, Labelable, Positionable, Sizeable, Widget,
};
use glam::DVec2;
use glium::Surface;

mod button_widget;
mod map;
mod map_renderer;
mod support;
mod tile_cache;
mod tile_requester;
mod ui_filter;
mod util;

pub use button_widget::*;
pub use map::*;
pub use map_renderer::*;
pub use tile_cache::*;
pub use tile_requester::*;
pub use ui_filter::*;
pub use util::*;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const MAX_ZOOM_LEVEL: u32 = 20;

widget_ids!(pub struct Ids { fps_logger, text, viewport, map_images[], squares[], tiles[], square_text[], weather_button, airplane_button, latitude_lines[], latitude_text[], longitude_lines[], longitude_text[], filter_widget,filer_button[] });

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

    let noto_sans_ttf = include_bytes!("../assets/fonts/NotoSans/NotoSans-Regular.ttf");

    let font = Font::from_bytes(noto_sans_ttf).expect("Failed to decode font");
    ui.fonts.insert(font);

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let mut last_time = std::time::Instant::now();
    let mut frame_time_ms = 0.0;

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create Tokio runtime!");

    let mut tile_cache = TileCache::new(&runtime);

    let mut should_update_ui = true;
    let mut viewer = map::TileView::new(0.0, 0.0, 2.0, 1080.0 / 2.0);
    let mut last_cursor_pos: Option<DVec2> = None;
    let mut left_pressed = false;

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
                    let ui = &mut ui.set_widgets();

                    map_renderer::draw(
                        &mut tile_cache,
                        &viewer,
                        &display,
                        &mut image_map,
                        &mut ids,
                        ui,
                    );

                    let frame_time_str = format!(
                        "FT: {:.2}, FPS: {}",
                        frame_time_ms,
                        (1000.0 / frame_time_ms) as u32
                    );

                    let widget_x_position = (ui.win_w / 2.0) * 0.95;
                    let widget_y_position = (ui.win_h / 2.0) * 0.90;

                    ids.filer_button.resize(3, &mut ui.widget_id_generator());

                    widget::Text::new(frame_time_str.as_str())
                        .top_left()
                        .color(conrod_core::color::WHITE)
                        .justify(conrod_core::text::Justify::Right)
                        .font_size(12)
                        .set(ids.fps_logger, ui);

                    if let Some(_clicks) = CircularButton::image(airplane_ids.normal)
                        .x(widget_x_position)
                        .y(widget_y_position)
                        .w_h(50.0, 50.0)
                        .label_color(conrod_core::color::WHITE)
                        .label("Airplane Button")
                        .set(ids.airplane_button, ui)
                    {
                        println!("{:?}", ui.xy_of(ids.airplane_button));
                    }

                    if let Some(_clicks) = CircularButton::image(weather_id.normal)
                        .x(widget_x_position)
                        .y(widget_y_position - 70.0)
                        .w_h(50.0, 50.0)
                        .label_color(conrod_core::color::WHITE)
                        .label("Weather Button")
                        .set(ids.weather_button, ui)
                    {
                        println!("{:?}", ui.xy_of(ids.weather_button));
                    }

                    // Request redraw if the `Ui` has changed.
                    //
                    //
                    //

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

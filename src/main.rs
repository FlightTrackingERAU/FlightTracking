use conrod_core::{widget, widget_ids, Colorable, Positionable, Widget};
use glium::Surface;

mod map;
mod map_renderer;

mod support;
mod util;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;

widget_ids!(pub struct Ids { fps_logger, text, viewport, squares[], square_text[] });

fn main() {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Conrod Window")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &event_loop).unwrap();

    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    let mut ids = Ids::new(ui.widget_id_generator());

    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();

    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    let mut last_time = std::time::Instant::now();
    let mut frame_time_ms = 0.0;
    let mut should_update_ui = true;
    let viewer = map::TileView::new(0.0, 0.0, 2.0, 1080 / 2);

    event_loop.run(move |event, _, control_flow| {
        // Break from the loop upon `Escape` or closed window.
        if let glium::glutin::event::Event::WindowEvent { event, .. } = &event {
            match event {
                // Break from the loop upon `Escape`.
                glium::glutin::event::WindowEvent::CloseRequested
                | glium::glutin::event::WindowEvent::KeyboardInput {
                    input:
                        glium::glutin::event::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
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
                    //should_update_ui = false;

                    // Set the widgets.
                    let ui = &mut ui.set_widgets();

                    // "Hello World!" in the middle of the screen.
                    widget::Text::new("SE 300")
                        .middle_of(ui.window)
                        .color(conrod_core::color::WHITE)
                        .font_size(32)
                        .set(ids.text, ui);

                    let frame_time_str = format!(
                        "FT: {:.2}, FPS: {}",
                        frame_time_ms,
                        (1000.0 / frame_time_ms) as u32
                    );
                    widget::Text::new(frame_time_str.as_str())
                        .top_right()
                        .color(conrod_core::color::WHITE)
                        .justify(conrod_core::text::Justify::Right)
                        .font_size(12)
                        .set(ids.fps_logger, ui);

                    map_renderer::draw(&viewer, &mut ids, ui);

                    // Request redraw if the `Ui` has changed.
                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                //render and swap buffers
                let primitives = ui.draw();

                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(1.0, 0.0, 1.0, 1.0);
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

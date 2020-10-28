use glium::{glutin, program, implement_vertex, uniform, Surface};
use glium::index::PrimitiveType;
use anyhow::Result;
use std::time::{Instant, Duration};

fn main() -> Result<()> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::Size::Logical(glutin::dpi::LogicalSize::new(512.0, 512.0)))
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top(true)
        .with_title("Just Breathe");
    let wb = if cfg!(target_os = "linux") {
        use glutin::platform::unix::WindowBuilderExtUnix;
        wb
            .with_class("just-breathe".to_string(), "42".to_string())
            .with_x11_window_type(vec![glutin::platform::unix::XWindowType::Dnd])
    }
    else {
        wb
    };


    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop)?;

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            colour: [f32; 3],
        }

        implement_vertex!(Vertex, position, colour);

        glium::VertexBuffer::new(&display, &[
            Vertex { position: [-0.5, -0.5], colour: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.0,  0.5], colour: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5], colour: [1.0, 0.0, 0.0] },
        ])?
    };

    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[0u16, 1, 2])?;

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec3 colour;
                out vec3 _colour;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    _colour = colour;
                }
            ",

            fragment: "
                #version 140
                in vec3 _colour;
                out vec4 f_color;
                void main() {
                    f_color = vec4(_colour, 1.0);
                }
            "
        },
    )?;

    {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.finish()?;
    }

    let mut last_time = Instant::now();
    let mut last_render_time = Instant::now();
    let mut time_accumulator: f64 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(last_render_time + Duration::from_secs_f64(1.0 / 60.0f64));
        match event {
            glutin::event::Event::LoopDestroyed => return,
            glutin::event::Event::MainEventsCleared => {
                display.gl_window().window().request_redraw();
            },
            glutin::event::Event::RedrawRequested(_) => {
                let uniforms = uniform! {
                    matrix: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32],
                    ]
                };

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
                target.finish().unwrap();
            },
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::Resized(..) => {
                    display.gl_window().window().request_redraw();
                },
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },
            _ => (),
        }
    });
}

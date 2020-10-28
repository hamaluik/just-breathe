use glium::{glutin, program, implement_vertex, uniform, Surface};
use glium::index::PrimitiveType;
use anyhow::Result;
use std::time::{Instant, Duration};

const UPDATE_PERIOD: f64 = 1.0 / 60.0;

#[derive(Copy, Clone, Debug)]
enum BreatheState {
    In(f64),
    HoldIn(f64),
    Out(f64),
    HoldOut(f64),
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    ((1.0 - t) * a) + (t * b)
}

fn ease_in_out_cubic(x: f64) -> f64 {
    if x < 0.5 { 4. * x * x * x }
    else {
        let y = x.mul_add(2., -2.);
        (y * y * y).mul_add(0.5, 1.)
    }
}

impl BreatheState {
    fn scale(&self) -> f32 {
        match self {
            BreatheState::In(t) => lerp(ease_in_out_cubic(t / 4.0), 0.25, 1.0) as f32,
            BreatheState::HoldIn(_) => 1.0,
            BreatheState::Out(t) => lerp(ease_in_out_cubic(t / 4.0), 1.0, 0.25) as f32,
            BreatheState::HoldOut(_) => 0.25,
        }
    }

    fn colour(&self) -> (f32, f32, f32) {
        let blue = 260.0;
        let red = 330.0;

        let hue = match self {
            BreatheState::In(_) => blue,
            BreatheState::HoldIn(t) => lerp(ease_in_out_cubic(t / 4.0), blue, red),
            BreatheState::Out(_) => red,
            BreatheState::HoldOut(t) => lerp(ease_in_out_cubic(t / 4.0), red, blue),
        };

        let colour = palette::Hsl::new(palette::RgbHue::from_degrees(hue), 0.5, 0.5);
        let colour = palette::LinSrgb::from(colour);

        (colour.red as f32, colour.green as f32, colour.blue as f32)
    }

    fn advance(&mut self, dt: f64) {
        *self = match self {
            BreatheState::In(mut t) => {
                t += dt;
                if t >= 4.0 {
                    t -= 4.0;
                    BreatheState::HoldIn(t)
                }
                else {
                    BreatheState::In(t)
                }
            },
            BreatheState::HoldIn(mut t) => {
                t += dt;
                if t >= 4.0 {
                    t -= 4.0;
                    BreatheState::Out(t)
                }
                else {
                    BreatheState::HoldIn(t)
                }
            },
            BreatheState::Out(mut t) => {
                t += dt;
                if t >= 4.0 {
                    t -= 4.0;
                    BreatheState::HoldOut(t)
                }
                else {
                    BreatheState::Out(t)
                }
            },
            BreatheState::HoldOut(mut t) => {
                t += dt;
                if t >= 4.0 {
                    t -= 4.0;
                    BreatheState::In(t)
                }
                else {
                    BreatheState::HoldOut(t)
                }
            },
        };
    }
}

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


    let cb = glutin::ContextBuilder::new()
        .with_srgb(true)
        .with_vsync(true)
        .with_multisampling(8);
    let display = glium::Display::new(wb, cb, &event_loop)?;

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        let mut vertices: [Vertex; 257] = [ Vertex { position: [ 0.0, 0.0 ] }; 257 ];
        let mut theta: f32 = 0.0;
        let dtheta: f32 = 2.0 * std::f32::consts::PI / 255.0;
        for i in 1..257 {
            vertices[i].position[0] = theta.cos();
            vertices[i].position[1] = theta.sin();
            theta += dtheta;
        }
        glium::VertexBuffer::immutable(&display, &vertices)?
    };

    let index_buffer = {
        let mut indices: [u16; 257] = [0; 257];
        for i in 1..257 {
            indices[i] = i as u16;
        }
        glium::IndexBuffer::immutable(&display, PrimitiveType::TriangleFan, &indices)?
    };

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                }
            ",

            fragment: "
                #version 140
                uniform vec4 colour;
                out vec4 _color;
                void main() {
                    _color = colour;
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

    let mut breathe_state = BreatheState::In(0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(last_render_time + Duration::from_secs_f64(UPDATE_PERIOD));
        match event {
            glutin::event::Event::LoopDestroyed => return,
            glutin::event::Event::MainEventsCleared => {
                let now = Instant::now();
                let delta = now - last_time;
                last_time = now;
                breathe_state.advance(delta.as_secs_f64());

                display.gl_window().window().request_redraw();
            },
            glutin::event::Event::RedrawRequested(_) => {
                last_render_time = Instant::now();

                let scale = breathe_state.scale();
                let colour = breathe_state.colour();

                let uniforms = uniform! {
                    matrix: [
                        [scale, 0.0, 0.0, 0.0],
                        [0.0, scale, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32],
                    ],
                    colour: [colour.0, colour.1, colour.2, 1.0f32],
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

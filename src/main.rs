
#[macro_use]
extern crate glium;
use glium::{winit::keyboard::{KeyCode, PhysicalKey}, Surface};
use nalgebra_glm::{self, Mat4};
mod block;
use block::Block;

fn main() {

    
    let event_loop = glium::winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let dirt_image = image::load_from_memory(include_bytes!("../textures/tnt_side.png")).unwrap().to_rgba8();
    let image_dimensions = dirt_image.dimensions();

    let dirt_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&dirt_image.into_raw(), image_dimensions);

    let dirt_texture = glium::texture::SrgbTexture2d::new(&display, dirt_image).unwrap();


    #[derive(Copy, Clone)]
    struct Vertex { 
        position: [f32; 3],
    }

    implement_vertex!(Vertex, position);

    let vertex_shader_src = r#"
        #version 140
        in vec3 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 projection;
        
        void main() {
            v_tex_coords = tex_coords;
            gl_Position = projection * view * model * vec4(position, 1.0);
        }
    "#;

   
    // Update fragment shader to use texture
    let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;
        
        void main() {
 + 0.5;           color = texture(tex, v_tex_coords);
        }
    "#;


    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut t: f32 = 0.0;
    const SPEED: f32 = 0.5;

    let mut camera_pos: [f32; 3] = [0.0, 0.0, 0.0];

    let _ = event_loop.run(move |event, window_target| {
        match event {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
                glium::winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                }
                glium::winit::event::WindowEvent::RedrawRequested => {
                    t += 0.02;
                    let off: f32 = t.sin();

                    

                    let block = Block::new([0.0,0.0,0.0]);
                    let vertex_buffer = glium::VertexBuffer::new(&display, &block.vertices).unwrap();
                    let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &block.indices).unwrap();

                    let mut target = display.draw();

                    // Model Matrix
                    let model: [[f32; 4]; 4] = nalgebra_glm::rotate(&nalgebra_glm::Mat4::identity(), t, &nalgebra_glm::vec3(0.0, 1.0, 0.0)).into(); // rotate (irrespective of camera)

                    // View Matrix (move backwards)
                    let view: [[f32;4];4] = nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &nalgebra_glm::vec3(camera_pos[0] + 0.0, camera_pos[1] + 0.0, camera_pos[2] - 3.0)).into();

                    // Projection Matrix
                    let projection: [[f32; 4]; 4] = nalgebra_glm::perspective(45.0, 800.0 / 600.0, 0.1, 100.0).into();
                    
                    target.clear_color(0.0, 0.0, 0.0, 1.0);
                    target.draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniform! {
                            model: model,
                            view: view,
                            projection: projection,
                            tex: &dirt_texture
                        },
                        &Default::default())
                        .unwrap();

                    target.finish().unwrap();
                },
                glium::winit::event::WindowEvent::KeyboardInput {  device_id, event, is_synthetic } => {
                    if event.state == glium::winit::event::ElementState::Pressed {
                        match event.physical_key {
                            // let's handle arrow keys to move the camera ? 
                            PhysicalKey::Code(KeyCode::KeyW) => {
                                camera_pos[2] += SPEED;
                            },
                            PhysicalKey::Code(KeyCode::KeyS) => {
                                camera_pos[2] -= SPEED;
                            },
                            PhysicalKey::Code(KeyCode::KeyA) => {
                                camera_pos[0] -= SPEED;
                            },
                            PhysicalKey::Code(KeyCode::KeyD) => {
                                camera_pos[0] += SPEED;
                            },
                            // Escape
                            PhysicalKey::Code(KeyCode::Escape) => {
                                window_target.exit();
                            }
                            _ => ()
                        }
                    }
                }
                _ => (),
            },
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        }
    });


}

#[macro_use]
extern crate glium;
use std::{collections::HashSet, time::Instant};

use glium::{winit::{event::{ElementState, Event, WindowEvent}, keyboard::{KeyCode, PhysicalKey}}, IndexBuffer, Surface, VertexBuffer};
use nalgebra_glm::{self, Vec3};
mod block;
use block::{Block, Vertex};
mod player;
use player::Player;
mod chunk_manager;
mod chunk;
use chunk_manager::ChunkManager;

fn main() {

    let mut delta_time: f32 = 0.0;
    let mut last_frame: Instant = Instant::now();
    let mut last_x: f32 = 0.0;
    let mut last_y: f32 = 0.0;
    
    
    let event_loop = glium::winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let dirt_image = image::load_from_memory(include_bytes!("../textures/dirt.png")).unwrap().to_rgba8();
    let image_dimensions = dirt_image.dimensions();

    let dirt_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&dirt_image.into_raw(), image_dimensions);

    let dirt_texture = glium::texture::SrgbTexture2d::new(&display, dirt_image).unwrap();

    let dirt_texture = dirt_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).minify_filter(glium::uniforms::MinifySamplerFilter::Nearest);

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut keys_pressed = HashSet::new();


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
       color = texture(tex, v_tex_coords);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut player = Player::new(Vec3::new(0.0, 70.0, 0.0));

    let mut vertex_buffer: Option<VertexBuffer<Vertex>> = None;
    let mut index_buffer: Option<IndexBuffer<u32>> = None;
    let mut chunk_manager = ChunkManager::new();
    let mut last_chunk_pos: [i32; 3] = player.chunk_pos;
    let mut do_chunk_updates = false;

    chunk_manager.update_chunks(player.position);
    let _ = event_loop.run(move |event, window_target| {
        match event { 
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => window_target.exit(),
                WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                }
                WindowEvent::RedrawRequested => {
                    // Update time values!

                    let current_frame = Instant::now();
                    delta_time = current_frame.duration_since(last_frame).as_secs_f32();
                    last_frame = current_frame;

                    do_chunk_updates = keys_pressed.contains(&PhysicalKey::Code(KeyCode::Backslash));

                    player.handle_keyboard_inputs(
                        &keys_pressed, 
                        &window_target, 
                        delta_time
                    );


                    if vertex_buffer.is_none() || index_buffer.is_none() || (do_chunk_updates && player.chunk_pos != last_chunk_pos) {
                        last_chunk_pos = player.chunk_pos;
                        chunk_manager.update_chunks(player.position);
                        let (vertices, indices) = chunk_manager.get_buffers();
                        vertex_buffer = Some(glium::VertexBuffer::new(&display, &vertices).unwrap());
                        index_buffer = Some(glium::IndexBuffer::new(
                            &display,
                            glium::index::PrimitiveType::TrianglesList,
                            &indices
                        ).unwrap());
                    }

                    let mut target = display.draw();

                    // Model Matrix
                    let model: [[f32; 4]; 4] = nalgebra_glm::Mat4::identity().into();

                    // Calculate camera front from yaw and pitch
                    let view = player.get_view_matrix();

                    // Projection Matrix
                    let projection: [[f32; 4]; 4] = nalgebra_glm::perspective(
                        45.0_f32.to_radians(),    // FOV in radians
                        target.get_dimensions().0 as f32 / target.get_dimensions().1 as f32,  // Actual aspect ratio
                        0.1,
                        100.0
                    ).into();
                    // projection: [[f32; 4]; 4] = nalgebra_glm::Mat4::identity().into();
                    target.clear_color_and_depth((120.0/255.0, 167.0/255.0, 255.0/255.0, 1.0), 1.0);
                    target.draw(
                        vertex_buffer.as_ref().unwrap(),
                        index_buffer.as_ref().unwrap(),
                        &program,
                        &uniform! {
                            model: model,
                            view: view,
                            projection: projection,
                            tex: dirt_texture,
                        },
                        &draw_parameters)
                        .unwrap();

                    target.finish().unwrap();
                },
                WindowEvent::KeyboardInput {  device_id, event, is_synthetic  } => {
                    match event.state {
                        ElementState::Pressed => {
                            keys_pressed.insert(event.physical_key);
                        }
                        ,
                        ElementState::Released => {
                            keys_pressed.remove(&event.physical_key);
                        },
                    }
                },
                WindowEvent::CursorMoved {position, ..} => {
                    
                    let xpos = position.x as f32;
                    let ypos = position.y as f32;

                    let x_offset = xpos - last_x;
                    let y_offset = last_y - ypos;
                    last_x = xpos;
                    last_y = ypos;

                    player.handle_mouse_inputs(x_offset, y_offset);
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

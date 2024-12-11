#[macro_use]
extern crate glium;
use std::{collections::{HashMap, HashSet}, sync::{mpsc::{self, channel}, Arc, Mutex, RwLock}, thread, time::Instant};

use chunk::Chunk;
use glium::{winit::{event::{ElementState, Event, WindowEvent}, keyboard::{KeyCode, PhysicalKey}}, IndexBuffer, Surface, VertexBuffer};
use nalgebra_glm::{self, Vec3};
mod block;
use block::{Block, Vertex};
mod player;
use player::Player;
mod chunk_manager;
mod chunk;
use chunk_manager::{ChunkManager, ChunkMeshData, WorkerMessage};
use threadpool::ThreadPool;

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
        #version 150
        in vec3 position;
        in vec2 tex_coords;
        in vec3 normal;
        
        out vec2 v_tex_coords;
        out vec3 v_normal;
        out vec3 v_position;
        
        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 projection;
        
        void main() {
            v_tex_coords = tex_coords;
            v_normal = mat3(transpose(inverse(model))) * normal;  // Transform normal to world space
            v_position = vec3(model * vec4(position, 1.0));
            gl_Position = projection * view * model * vec4(position, 1.0);
        }
    "#;

    // Update fragment shader to use texture
    let fragment_shader_src = r#"
        #version 150
        in vec2 v_tex_coords;
        in vec3 v_normal;
        in vec3 v_position;
        
        out vec4 color;
        uniform sampler2D tex;
        
        void main() {
            // Light direction (pointing downward and slightly to the side)
            vec3 light_dir = normalize(vec3(-0.2, -1.0, -0.3));
            
            // Ambient lighting
            float ambient_strength = 0.3;
            vec3 ambient = ambient_strength * vec3(1.0, 1.0, 1.0);
            
            // Diffuse lighting
            vec3 norm = normalize(v_normal);
            float diff = max(dot(norm, -light_dir), 0.0);
            vec3 diffuse = diff * vec3(1.0, 1.0, 1.0);
            
            // Combine lighting with texture
            vec4 tex_color = texture(tex, v_tex_coords);
            vec3 result = (ambient + diffuse) * tex_color.rgb;
            color = vec4(result, tex_color.a);
        }
    "#;

    let (task_sender, task_receiver) = mpsc::channel::<WorkerMessage>();
    let (result_sender, result_receiver) = mpsc::channel::<WorkerMessage>();


    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut player = Player::new(Vec3::new(0.0, 30.0, 0.0));

    // i need to initialize these somehow
    let mut vertex_buffer: Option<VertexBuffer<Vertex>> = None;
    let mut index_buffer: Option<IndexBuffer<u32>> = None;

    let (buffer_task_sender, buffer_task_receiver) = mpsc::channel::<BufferTask>();
    let (buffer_result_sender, buffer_result_receiver) = mpsc::channel::<(Vec<Vertex>, Vec<u32>)>();
    
    // Wrap chunk_manager in Arc<Mutex>
    let mut chunk_manager = ChunkManager::new(task_sender);

    let mesh_map: Arc<RwLock<HashMap<[i32; 3], ChunkMeshData>>> = Arc::new(RwLock::new(HashMap::new()));

    let mesh_map_clone = mesh_map.clone();
    let mut last_chunk_pos: [i32; 3] = player.chunk_pos;
    let mut do_chunk_updates = true;
    let mut last_chunk_count = 0;
    let mut initial_chunks_requested = false;

    enum BufferTask {
        // pass a PIT clone of the chunk map and the mesh map
        UpdateBuffers(HashMap<[i32; 3], Chunk>, Arc<RwLock<HashMap<[i32; 3], ChunkMeshData>>>),
        Shutdown,
    }

    let buffer_worker = thread::spawn(move || {
        loop {
            match buffer_task_receiver.recv() {
                Ok(BufferTask::UpdateBuffers(chunk_map, mesh_map)) => {
                    println!("Buffer worker received update buffers");
                    let (vertices, indices) = ChunkManager::get_buffers(chunk_map, mesh_map.clone());
                    buffer_result_sender.send((vertices, indices)).unwrap();
                    println!("Buffer worker Unlocked chunk manager");
                }
                Ok(BufferTask::Shutdown) => {
                    println!("Buffer worker shutting down!");
                    break;
                }
                Err(_) => break
            }
        }
    });

    let pool = ThreadPool::new(100);
   
    let worker = thread::spawn(move || {
        loop {
            match task_receiver.recv() {
                Ok(WorkerMessage::LoadChunkTask(task)) => {

                    let mut chunks_to_insert: Vec<Chunk> = Vec::new();

                    {
                        let mut map = task.chunk_map.write().unwrap();
                        let mut num_jobs = 0;

                        let (tx, rx) = channel::<Arc<Chunk>>();

                        for (i, origin) in task.origins.iter().enumerate() {
                            let origin = *origin;
                            if !map.contains_key(&origin) {
                                num_jobs += 1;
                                let tx = tx.clone();
                                pool.execute(move || {
                                    let chunk = Chunk::new(origin);
                                    tx.send(Arc::new(chunk)).expect("Failed to send chunk");
                                });
                            }
                        }

                        drop(tx);

                        let mut chunks_to_insert = Vec::with_capacity(num_jobs);
                        for i in 0..num_jobs {
                            match rx.recv() {
                                Ok(chunk) => {
                                    chunks_to_insert.push((*chunk).clone());
                                }
                                Err(e) => println!("Failed to receive chunk #{}: {:?}", i+1, e),
                            }
                        }
                        
                        println!("Received all chunks, inserting into map");
                        for chunk in chunks_to_insert {
                            map.insert(chunk.origin, chunk);
                        }
                    }
                    
                    println!("Worker releasing write lock on chunk map");
                    let map = task.chunk_map.read().unwrap();
                    println!("Worker acquired read lock for sending to buffer task");
                    buffer_task_sender.send(BufferTask::UpdateBuffers(map.clone(), mesh_map.clone())).unwrap();
                    println!("Worker sent buffer task");
                },
                Ok(WorkerMessage::Shutdown) => {
                    println!("Chunk worker received shutdown signal");
                    break;
                }
                Err(e) => {
                    println!("Chunk worker error receiving task: {:?}", e);
                    break;
                }
            } 
        }
    });


    let _ = event_loop.run(move |event, window_target| {
        match event { 
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                    chunk_manager.shutdown_sender();
                },
                WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                }
                WindowEvent::RedrawRequested => {
                    // Update time values!
                    // println!("RedrawRequested");
                    let current_frame = Instant::now();
                    delta_time = current_frame.duration_since(last_frame).as_secs_f32();
                    last_frame = current_frame;

                    //do_chunk_updates = keys_pressed.contains(&PhysicalKey::Code(KeyCode::Backslash));

                    player.handle_keyboard_inputs(
                        &keys_pressed, 
                        &window_target, 
                        delta_time
                    );


                    if ((vertex_buffer.is_none() || index_buffer.is_none()) && !initial_chunks_requested) || (do_chunk_updates && player.chunk_pos != last_chunk_pos) {
                       //println!("calling update chunks");
                       last_chunk_pos = player.chunk_pos;
                       println!("RedrawRequested is trying to lock chunk manager");
                        initial_chunks_requested = true;
                        chunk_manager.update_chunks(player.position, mesh_map_clone.clone());

                    }

                    // look for buffer updates?
                    if let Ok(buffer_result) = buffer_result_receiver.try_recv() {
                        if buffer_result.0.len() > 0 {
                            vertex_buffer = Some(glium::VertexBuffer::new(&display, &buffer_result.0).unwrap());
                            index_buffer = Some(glium::IndexBuffer::new(
                                &display,
                                glium::index::PrimitiveType::TrianglesList,
                                &buffer_result.1
                            ).unwrap());
                        }
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

                    if vertex_buffer.is_some() && index_buffer.is_some() {
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

                    }
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

worker.join().unwrap();

}

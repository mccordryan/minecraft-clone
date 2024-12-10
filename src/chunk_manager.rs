use std::{collections::{HashMap, HashSet}, sync::{mpsc::Sender, Arc, Mutex, MutexGuard}};
use nalgebra_glm::Vec3;
use crate::{block::{Block, BlockType, FaceDir, Vertex}, chunk::Chunk};

pub struct ChunkManager {
    pub chunks: Arc<Mutex<HashMap<[i32; 3], Chunk>>>,
    pub task_sender: Sender<WorkerMessage>,
    pub mesh_map: HashMap<[i32; 3], ChunkMeshData>,
}

pub struct ChunkMeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct LoadChunkTask {
    pub origin: [i32; 3],
    pub chunk_map: Arc<Mutex<HashMap<[i32; 3], Chunk>>>,
}

pub enum WorkerMessage {
    LoadChunkTask(LoadChunkTask),
    Shutdown,
}

impl ChunkManager {
    pub fn new(task_sender: Sender<WorkerMessage>) -> Self {
        ChunkManager {
            chunks: Arc::new(Mutex::new(HashMap::new())),
            task_sender,
            mesh_map: HashMap::new(),
        }
    }

    pub fn chunk_in_range(chunk_pos: [i32; 3], user_chunk_pos: [i32; 3], render_distance: i32) -> bool {
        //println!("Checking if chunk {:?} is in range of user chunk {:?} with distance {}", 
                // chunk_pos, user_chunk_pos, render_distance);
        let in_range = chunk_pos[0] >= user_chunk_pos[0] - render_distance &&
               chunk_pos[0] <= user_chunk_pos[0] + render_distance &&
               chunk_pos[1] >= user_chunk_pos[1] - render_distance &&
               chunk_pos[1] <= user_chunk_pos[1] + render_distance &&
               chunk_pos[2] >= user_chunk_pos[2] - render_distance &&
               chunk_pos[2] <= user_chunk_pos[2] + render_distance;
       // println!("Result: {}", in_range);
        return in_range;
    }



    pub fn update_chunks(&mut self, position: Vec3) {
        println!("Updating chunks");
       
        let chunk_size: i32 = 16;
        let render_distance: i32 = 2;
        
        let user_chunk_pos = ChunkManager::get_chunk_at(position.into());

        let mut chunks = self.chunks.lock().unwrap();
        println!("{} chunks at beginning of update_chunks", chunks.len());
        let chunks_to_remove: Vec<[i32; 3]> = chunks.keys()
            .filter(|&key| !ChunkManager::chunk_in_range(*key, user_chunk_pos, render_distance))
            .copied()
            .collect();

        let removed = chunks_to_remove.len();
            
        for key in chunks_to_remove {
            chunks.remove(&key);
        }

        println!("{} chunks after removing {} chunks", chunks.len(), removed);

        let existing_chunks: HashSet<_> = chunks.keys().cloned().collect();

        println!("{} existing chunks identified", existing_chunks.len());

        drop(chunks);
        let mut chunk_tasks: i32 = 0;

        for x in (user_chunk_pos[0] - render_distance)..(user_chunk_pos[0] + render_distance ) {
            for y in (user_chunk_pos[1] - render_distance)..(user_chunk_pos[1] + render_distance) {
                for z in (user_chunk_pos[2] - render_distance)..(user_chunk_pos[2] + render_distance) {
                    let chunk_pos = [x, y, z];
                    if ChunkManager::chunk_in_range(chunk_pos, user_chunk_pos, render_distance) {
                        let origin = [x,y,z];
                        if !existing_chunks.contains(&origin) {
                            // send a task to the worker to insert this chunk asynchronously
                            // println!("Sending a chunk task");
                            chunk_tasks += 1;
                            self.task_sender.send(WorkerMessage::LoadChunkTask(LoadChunkTask{
                                origin,
                                chunk_map: self.chunks.clone()
                            })
                        ).unwrap();
                        }
                    }
                }
            }
        }
        println!("generated {} tasks", chunk_tasks);
    }


    pub fn get_chunk_at(pos: [f32; 3]) -> [i32; 3] {
        let chunk_size = 16;
        [
            (pos[0] as i32).div_euclid(chunk_size),
            (pos[1] as i32).div_euclid(chunk_size),
            (pos[2] as i32).div_euclid(chunk_size),
        ]
    }
    
    fn get_block(&self, world_pos: [i32; 3], chunks: &MutexGuard<HashMap<[i32; 3], Chunk>>) -> Option<Block> {
        let chunk_size = 16;
        // Calculate chunk origin and local coordinate
        let chunk_origin = [
            (world_pos[0].div_euclid(chunk_size)) * chunk_size,
            (world_pos[1].div_euclid(chunk_size)) * chunk_size,
            (world_pos[2].div_euclid(chunk_size)) * chunk_size,
        ];
        
        let local_pos = [
            world_pos[0].rem_euclid(chunk_size) as usize,
            world_pos[1].rem_euclid(chunk_size) as usize,
            world_pos[2].rem_euclid(chunk_size) as usize,
        ];

        // test here
        chunks.get(&chunk_origin)
            .map(|chunk| &chunk.blocks[local_pos[0]][local_pos[1]][local_pos[2]]).copied()
    }

    fn should_render_face(&self, neighbor_pos: [i32; 3], block: &Block, chunks: &MutexGuard<HashMap<[i32;3], Chunk>>) -> bool {
        if let Some(neighbor) = self.get_block(neighbor_pos, chunks) {
            if neighbor.block_type != BlockType::Air{
            //println!("Neighbor: {:?}\n Self: {:?}", neighbor, block);
            }
             neighbor.block_type == BlockType::Air
        } else {
            // Convert i32 array to f32 array manually
            //let pos_f32 = [neighbor_pos[0] as f32, neighbor_pos[1] as f32, neighbor_pos[2] as f32];
            //println!("Neighbor Position: {:?}, Neighbor Chunk: {:?}", neighbor_pos, ChunkManager::get_chunk_at(pos_f32));
            true
        }
    }

    pub fn shutdown_sender(&self) {
        self.task_sender.send(WorkerMessage::Shutdown).unwrap();
    }

    pub fn get_buffers(&mut self) -> (Vec<Vertex>, Vec<u32> ) {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_offset: u32 = 0;

        println!("get_buffers is locking chunks");
        let chunks = self.chunks.lock().unwrap();

        for (origin, chunk) in chunks.iter() {

            // define here so we can cache
            let mut chunk_indices: Vec<u32> = Vec::new();
            let mut chunk_vertices: Vec<Vertex> = Vec::new();

            if self.mesh_map.contains_key(origin){
                let chunk_mesh_data = self.mesh_map.get(origin).unwrap();
                vertices.extend(chunk_mesh_data.vertices.clone());
                indices.extend(chunk_mesh_data.indices.iter().map(|i| i + vertex_offset));
                vertex_offset += chunk_mesh_data.vertices.len() as u32;
                continue;
            }

            let mut chunk_vertex_offset = 0;

            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        let block = &chunk.blocks[x][y][z];
                        if block.block_type == BlockType::Air {
                            continue;
                        }

                        let chunk_size = 16; // todo test idk
                        let world_x = origin[0] * chunk_size + x as i32;
                        let world_y = origin[1] * chunk_size  + y as i32;
                        let world_z = origin[2] *chunk_size + z as i32;

                        // Check each face direction
                        let mut faces_to_render = Vec::new();
                        
                        // Up face (checking above)
                        if self.should_render_face([world_x, world_y + 1, world_z], &block, &chunks) {
                            faces_to_render.push(FaceDir::Up); 
                        }
                        // Down face (checking below)
                        if self.should_render_face([world_x, world_y - 1, world_z], &block, &chunks) {
                            faces_to_render.push(FaceDir::Down); 
                        }
                        // Right face (now Left)
                        if self.should_render_face([world_x - 1, world_y, world_z], &block, &chunks) {
                            faces_to_render.push(FaceDir::Left);
                        }
                        // Left face (now Right)
                        if self.should_render_face([world_x + 1, world_y, world_z], &block, &chunks) {
                            faces_to_render.push(FaceDir::Right);
                        }
                        // Front face
                        if self.should_render_face([world_x, world_y, world_z - 1], &block, &chunks) {
                            // println!("{} {} {}", world_x, world_y, world_z);
                            faces_to_render.push(FaceDir::Front);
                        }
                        // Back face
                        if self.should_render_face([world_x, world_y, world_z + 1], &block, &chunks) {
                            faces_to_render.push(FaceDir::Back);
                        }

                        // Generate only the necessary faces
                        let (block_vertices, block_indices) = block.generate_faces(faces_to_render);
                        
                        // Add the vertices
                        chunk_vertices.extend_from_slice(&block_vertices);
                        
                        // Add the indices with offset
                        chunk_indices.extend(block_indices.iter().map(|i| i + chunk_vertex_offset as u32));
                        
                        chunk_vertex_offset += block_vertices.len() as u32;
                    }
                }
            }

            vertices.extend(&chunk_vertices);
            indices.extend(chunk_indices.iter().map(|i| i + vertex_offset));

            vertex_offset += chunk_vertex_offset;
            
            // add to mesh map
            let chunk_mesh_data = ChunkMeshData {
                vertices: chunk_vertices,
                indices: chunk_indices,
            };

            self.mesh_map.insert(*origin, chunk_mesh_data);
        }

        (vertices, indices)
    }
}
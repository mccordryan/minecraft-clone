use std::collections::HashMap;
use nalgebra_glm::Vec3;
use crate::{block::{Block, BlockType, FaceDir, Vertex}, chunk::Chunk};

pub struct ChunkManager {
    pub chunks: HashMap<[i32; 3], Chunk>
}

impl ChunkManager {
    pub fn new() -> Self {
        ChunkManager {
            chunks: HashMap::new()
        }
    }

    pub fn update_chunks(&mut self, position: Vec3) {
        let mut new_chunks = HashMap::new();
        let chunk_size: i32 = 16;
        let render_distance: i32 = 1;
        
        let user_chunk_pos = ChunkManager::get_chunk_at(position.into());

        for x in (user_chunk_pos[0] - render_distance)..(user_chunk_pos[0] + render_distance) {
            for y in (user_chunk_pos[1] - render_distance)..(user_chunk_pos[1] + render_distance) {
                for z in (user_chunk_pos[2] - render_distance)..(user_chunk_pos[2] + render_distance) {
                    let origin = [x * chunk_size, y * chunk_size, z * chunk_size];
                    new_chunks.insert(origin, Chunk::new(origin));
                }
            }
        }

        self.chunks = new_chunks;
    }


    pub fn get_chunk_at(pos: [f32; 3]) -> [i32; 3] {
        let chunk_size = 16;
        [
            (pos[0] as i32).div_euclid(chunk_size),
            (pos[1] as i32).div_euclid(chunk_size),
            (pos[2] as i32).div_euclid(chunk_size),
        ]
    }
    
    fn get_block(&self, world_pos: [i32; 3]) -> Option<&Block> {
        let chunk_size = 16;
        // Calculate chunk origin and local coordinates
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

        self.chunks.get(&chunk_origin)
            .map(|chunk| &chunk.blocks[local_pos[0]][local_pos[1]][local_pos[2]])
    }

    fn should_render_face(&self, neighbor_pos: [i32; 3]) -> bool {
        if let Some(neighbor) = self.get_block(neighbor_pos) {
             neighbor.block_type == BlockType::Air
        } else {
            // Convert i32 array to f32 array manually
            let pos_f32 = [neighbor_pos[0] as f32, neighbor_pos[1] as f32, neighbor_pos[2] as f32];
            //println!("Neighbor Position: {:?}, Neighbor Chunk: {:?}", neighbor_pos, ChunkManager::get_chunk_at(pos_f32));
            true
        }
    }

    pub fn get_buffers(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_offset: u32 = 0;

        for (origin, chunk) in &self.chunks {
            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        let block = &chunk.blocks[x][y][z];
                        if block.block_type == BlockType::Air {
                            continue;
                        }

                        let world_x = origin[0] + x as i32;
                        let world_y = origin[1] + y as i32;
                        let world_z = origin[2] + z as i32;
                        let block_world_pos = [world_x, world_y, world_z];

                        // Check each face direction
                        let mut faces_to_render = Vec::new();
                        
                        // Up face
                        if self.should_render_face( [world_x, world_y - 1, world_z]) {
                            faces_to_render.push(FaceDir::Up);
                        }
                        // Down face
                        if self.should_render_face( [world_x, world_y + 1, world_z]) {
                            faces_to_render.push(FaceDir::Down);
                        }
                        // Right face
                        if self.should_render_face( [world_x + 1, world_y, world_z]) {
                            faces_to_render.push(FaceDir::Left);
                        }
                        // Left face
                        if self.should_render_face([world_x - 1, world_y, world_z]) {
                            faces_to_render.push(FaceDir::Right);
                        }
                        // Front face
                        if self.should_render_face( [world_x, world_y, world_z + 1]) {
                            faces_to_render.push(FaceDir::Front);
                        }
                        // Back face
                        if self.should_render_face( [world_x, world_y, world_z - 1]) {
                            faces_to_render.push(FaceDir::Back);
                        }

                        // Generate only the necessary faces
                        let (block_vertices, block_indices) = block.generate_faces(faces_to_render);
                        
                        // Add the vertices
                        vertices.extend_from_slice(&block_vertices);
                        
                        // Add the indices with offset
                        for index in block_indices {
                            indices.push(index + vertex_offset);
                        }
                        
                        vertex_offset += block_vertices.len() as u32;
                    }
                }
            }
        }

        (vertices, indices)
    }
}
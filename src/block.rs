use nalgebra_glm::Vec3;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

#[derive(Debug, EnumIter, PartialEq)]
pub enum FaceDir {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

impl FaceDir {
    // Returns (normal_axis, u_axis, v_axis) where:
    // normal_axis is the axis perpendicular to the face
    // u_axis is the horizontal texture axis
    // v_axis is the vertical texture axis
    fn get_axes(&self) -> (Vec3, Vec3, Vec3) {
        match self {
            FaceDir::Up => (
                Vec3::new(0.0, 1.0, 0.0),  // Normal points up
                Vec3::new(1.0, 0.0, 0.0),  // U axis points right
                Vec3::new(0.0, 0.0, -1.0), // V axis points forward
            ),
            FaceDir::Down => (
                Vec3::new(0.0, -1.0, 0.0), // Normal points down
                Vec3::new(1.0, 0.0, 0.0),  // U axis points right
                Vec3::new(0.0, 0.0, 1.0),  // V axis points backward
            ),
            FaceDir::Left => (
                Vec3::new(-1.0, 0.0, 0.0), // Normal points left
                Vec3::new(0.0, 0.0, -1.0), // U axis points forward
                Vec3::new(0.0, 1.0, 0.0),  // V axis points up
            ),
            FaceDir::Right => (
                Vec3::new(1.0, 0.0, 0.0),  // Normal points right
                Vec3::new(0.0, 0.0, 1.0),  // U axis points backward
                Vec3::new(0.0, 1.0, 0.0),  // V axis points up
            ),
            FaceDir::Front => (
                Vec3::new(0.0, 0.0, -1.0), // Normal points forward
                Vec3::new(1.0, 0.0, 0.0),  // U axis points right
                Vec3::new(0.0, 1.0, 0.0),  // V axis points up
            ),
            FaceDir::Back => (
                Vec3::new(0.0, 0.0, 1.0),  // Normal points backward
                Vec3::new(-1.0, 0.0, 0.0), // U axis points left
                Vec3::new(0.0, 1.0, 0.0),  // V axis points up
            ),
        }
    }
}

pub struct Block {
    pub block_type: BlockType,
    pub pos: [f32; 3],
} 

#[derive(PartialEq)]
pub enum BlockType {
        Air,
        TNT,
    }

impl Block {
   

    pub fn new(pos: [f32; 3]) -> Self {
        let block_type: BlockType;

        if pos[1] > 0.0 { // y limit for now
            block_type = BlockType::Air;
        } else {
            block_type = BlockType::TNT;
        }
        
        Block { block_type, pos}
    }

    pub fn generate_faces(&self, faces: Vec<FaceDir>) -> (Vec<Vertex>, Vec<u32>) {
        let base_pos = Vec3::from(self.pos);
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        if self.block_type == BlockType::Air {
            return (vertices, indices)
        }

        for face in faces {
            Self::add_face(&mut vertices, &mut indices, base_pos, face);
        }

        (vertices, indices)
    }

    fn add_face(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, base_pos: Vec3, face_dir: FaceDir) {
        let (normal, u_axis, v_axis) = face_dir.get_axes();
        let vertex_start = vertices.len() as u32;
        
        // Calculate the starting corner position for this face
        // Move half a block in the opposite direction of the normal
        let corner_pos = base_pos - normal * 0.5;
        
        // Generate the four corners of the face
        let positions = [
            corner_pos - u_axis * 0.5 - v_axis * 0.5, // Bottom Left
            corner_pos - u_axis * 0.5 + v_axis * 0.5, // Top Left
            corner_pos + u_axis * 0.5 + v_axis * 0.5, // Top Right
            corner_pos + u_axis * 0.5 - v_axis * 0.5, // Bottom Right
        ];
        
        // Add vertices with texture coordinates
        for (i, pos) in positions.iter().enumerate() {
            let tex_coords = match i {
                0 => [0.0, 0.0], // Bottom Left
                1 => [0.0, 1.0], // Top Left
                2 => [1.0, 1.0], // Top Right
                3 => [1.0, 0.0], // Bottom Right
                _ => unreachable!(),
            };
            
            vertices.push(Vertex {
                position: [pos.x, pos.y, pos.z],
                tex_coords,
            });
        }
        
        // Add indices for the two triangles that make up the face
        indices.extend_from_slice(&[
            vertex_start,     // Bottom Left
            vertex_start + 1, // Top Left
            vertex_start + 2, // Top Right
            vertex_start,     // Bottom Left
            vertex_start + 2, // Top Right
            vertex_start + 3, // Bottom Right
        ]);
    }
}

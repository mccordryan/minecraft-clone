use std::array;

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::block::{Block, BlockType};

#[derive(Clone, Copy)]
pub struct Chunk {
    pub blocks: [[[Block; 16]; 16]; 16],
    pub origin: [i32; 3], // front bottom left of cubic 16x16x16 chunk ? 
}


impl Chunk {
    pub fn new(origin: [i32;3]) -> Self {
        println!("New chunk at {}, {}, {}", origin[0], origin[1], origin[2]);
       
       let scale = 150.0;
        // Pre-calculate heights for all x,z coordinates
        let heights: [[i32; 16]; 16] = array::from_fn(|x| 
            array::from_fn(|z| {
                let fbm = Fbm::<Perlin>::new(0)
                .set_octaves(4)
                .set_persistence(0.6)
                .set_frequency(0.7)
                .set_lacunarity(2.2);

     
                
                let world_x = origin[0] as f64 + x as f64;
                let world_z = origin[2] as f64 + z as f64;
                
                let noise_value = fbm.get([world_x / scale, world_z / scale]);
                // Transform from [-1, 1] to [0, 1] then scale to reasonable height
                let height = ((noise_value + 1.0) * 0.5 * 60.0) as i32;
                height // This will give heights roughly in the range [0, 32]
            })
        );

        let blocks = array::from_fn(|x| 
            array::from_fn(|y|
                array::from_fn(|z| {
                    let block_x = (origin[0] + x as i32) as f64;
                    let block_y = (origin[1] + y as i32) as f64;
                    let block_z = (origin[2] + z as i32) as f64;
                    
                    let block_type = if heights[x][z] >= origin[1] + y as i32 {
                        BlockType::Grass
                    } else {
                        BlockType::Air
                    };

                    Block::new(
                        [block_x as f32, 
                         block_y as f32, 
                         block_z as f32],
                        block_type)
                })
            )
        );

        Chunk { blocks, origin }
    }
}
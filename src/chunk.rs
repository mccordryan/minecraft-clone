use std::array;

use crate::block::Block;

pub struct Chunk {
    pub blocks: [[[Block; 16]; 16]; 16],
    pub origin: [i32; 3], // front bottom left of cubic 16x16x16 chunk ? 
}


impl Chunk {
    pub fn new(origin: [i32;3]) -> Self {
        println!("New chunk at {}, {}, {}", origin[0], origin[1], origin[2]);
        let blocks = array::from_fn(|x| 
            array::from_fn(|y| 
                array::from_fn(|z| 
                    Block::new([(origin[0] + x as i32) as f32, 
                              (origin[1] + y as i32) as f32, 
                              (origin[2] + z as i32) as f32])
                )
            )
        );
        
        Chunk { blocks, origin }
    }
}
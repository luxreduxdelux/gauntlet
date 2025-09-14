use std::collections::HashMap;

use crate::external::r3d::*;
use crate::state::*;

pub struct Scene {
    light: HashMap<u32, Light>,
    light_index: u32,
}

impl Scene {
    pub fn new(_context: &mut Context) -> Self {
        let mut scene = Self {
            light: HashMap::default(),
            light_index: u32::default(),
        };

        //context.r3d.set_bloom_mode(BloomMode::Mix);
        //context.r3d.set_fog_mode(FogMode::Linear);
        //context.r3d.set_depth_of_field(true);
        //scene.light.set_active(true);
        //scene.light.set_position(Vector3::new(0.0, 2.0, 0.0));
        //scene.light.enable_shadow(512);

        scene
    }

    pub fn new_light(&mut self, context: &mut Context, light_type: LightType) -> u32 {
        self.light
            .insert(self.light_index, Light::new(&mut context.r3d, light_type));

        self.light_index += 1;

        self.light_index - 1
    }
}

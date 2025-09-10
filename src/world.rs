/*
* Copyright (c) 2025 luxreduxdelux
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions are met:
*
* 1. Redistributions of source code must retain the above copyright notice,
* this list of conditions and the following disclaimer.
*
* 2. Redistributions in binary form must reproduce the above copyright notice,
* this list of conditions and the following disclaimer in the documentation
* and/or other materials provided with the distribution.
*
* Subject to the terms and conditions of this license, each copyright holder
* and contributor hereby grants to those receiving rights under this license
* a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
* (except for failure to satisfy the conditions of this license) patent license
* to make, have made, use, offer to sell, sell, import, and otherwise transfer
* this software, where such license applies only to those patent claims, already
* acquired or hereafter acquired, licensable by such copyright holder or
* contributor that are necessarily infringed by:
*
* (a) their Contribution(s) (the licensed copyrights of copyright holders and
* non-copyrightable additions of contributors, in source or binary form) alone;
* or
*
* (b) combination of their Contribution(s) with the work of authorship to which
* such Contribution(s) was added by such copyright holder or contributor, if,
* at the time the Contribution is added, such addition causes such combination
* to be necessarily infringed. The patent license shall not apply to any other
* combinations which include the Contribution.
*
* Except as expressly stated above, no rights or licenses from any copyright
* holder or contributor is granted under this license, whether expressly, by
* implication, estoppel or otherwise.
*
* DISCLAIMER
*
* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
* AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
* IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE
* FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
* DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
* SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
* CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
* OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
* OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use crate::entity::*;
use crate::physical::*;
use crate::player::*;
use crate::state::*;
use crate::tina::*;

//================================================================

use raylib::prelude::*;
use serde::Deserialize;

//================================================================

#[derive(Deserialize)]
pub struct World {
    pub entity_list: Vec<Box<dyn Entity>>,
    #[serde(skip, default = "World::default_camera")]
    pub camera_3d: Camera3D,
    #[serde(skip)]
    pub camera_2d: Camera2D,
    #[serde(skip)]
    pub physical: Physical,
    #[serde(skip)]
    pub time: f32,
    #[serde(skip)]
    step: f32,
}

impl World {
    pub const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new(state: &mut State, context: &mut Context) -> anyhow::Result<Self> {
        let mut world = Self {
            entity_list: Vec::default(),
            camera_3d: Camera3D::perspective(
                Vector3::default(),
                Vector3::default(),
                Vector3::up(),
                90.0,
            ),
            camera_2d: Camera2D::default(),
            physical: Physical::default(),
            time: f32::default(),
            step: f32::default(),
        };

        let entity = Box::new(Player::new(state, context, &mut world)?);
        world.entity_list.push(entity);
        let entity = Box::new(Tina::new(state, context, &mut world)?);
        world.entity_list.push(entity);

        Ok(world)
    }

    pub fn main(
        &mut self,
        state: &mut State,
        draw: &mut RaylibDrawHandle<'_>,
        context: &mut Context,
    ) -> anyhow::Result<()> {
        if state.layout.is_none() {
            let frame_time = context.handle.get_frame_time().min(0.25);

            self.step += frame_time;

            while self.step >= Self::TIME_STEP {
                self.physical.tick();

                unsafe {
                    let world = self as *mut Self;

                    for entity in &mut self.entity_list {
                        entity.tick(state, &mut context.handle, &mut *world)?;
                    }
                }

                self.time += Self::TIME_STEP;
                self.step -= Self::TIME_STEP;
            }
        }

        unsafe {
            let world = self as *mut Self;

            for entity in &mut self.entity_list {
                entity.main(state, draw, &mut *world)?;
            }
        }
        {
            let mut draw_3d = draw.begin_mode3D(self.camera_3d);

            unsafe {
                let world = self as *mut Self;

                for entity in &mut self.entity_list {
                    entity.draw_3d(state, &mut draw_3d, &mut *world)?;
                }
            }
        }
        {
            let mut draw_2d = draw.begin_mode2D(Camera2D {
                offset: Vector2::zero(),
                target: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            });

            unsafe {
                let world = self as *mut Self;

                for entity in &mut self.entity_list {
                    entity.draw_2d(state, &mut draw_2d, &mut *world)?;
                }
            }
        }

        Ok(())
    }

    // remove this when raylib-rs has a default function for Camera3D.
    fn default_camera() -> Camera3D {
        Camera3D::perspective(
            Vector3::default(),
            Vector3::default(),
            Vector3::default(),
            f32::default(),
        )
    }
}

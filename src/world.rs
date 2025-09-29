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

use crate::entity::implementation::*;
use crate::physical::*;
use crate::scene::*;
use crate::state::*;

//================================================================

use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::Deserialize;

//================================================================

#[derive(Deserialize)]
pub struct World<'a> {
    pub level: Vec<String>,
    pub entity_list: Vec<Box<dyn Entity>>,
    #[serde(skip)]
    pub time: f32,
    #[serde(skip)]
    step: f32,
    #[serde(skip)]
    entity_index: usize,
    #[serde(skip)]
    entity_attach: Vec<Box<dyn Entity>>,
    #[serde(skip)]
    pub scene: Scene<'a>,
    #[serde(skip)]
    pub player: Option<usize>,
}

impl<'a> World<'a> {
    pub const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new(state: &mut State, context: &mut Context, path: &str) -> anyhow::Result<Self> {
        let file = &format!("data/level/{path}/{path}.json");
        let file = std::fs::read_to_string(file)?;
        let mut file: Self = serde_json::from_str(&file)?;

        for level in &file.level {
            file.scene
                .room_add(context, &format!("data/level/{path}/{level}"))?;
        }

        // entity index 0 is meant for the level's rigid-body.
        file.entity_index = 1;

        unsafe {
            let world = &mut file as *mut Self;
            let ctx = context as *mut Context;

            for entity in &mut file.entity_list {
                file.entity_index += 1;
                entity.get_info_mutable().index = file.entity_index - 1;
                entity.initialize(state, &mut *ctx, &mut *world)?;
            }
        }

        file.scene.initialize(context)?;

        Ok(file)
    }

    pub fn entity_from_collider(
        &mut self,
        collider: ColliderHandle,
    ) -> anyhow::Result<Option<&Box<dyn Entity>>> {
        let collider = self.scene.physical.get_collider(collider)?;

        if let Some(parent) = collider.parent()
            && let Ok(rigid) = self.scene.physical.get_rigid(parent)
        {
            return Ok(self.entity_find(rigid.user_data as usize));
        }

        Ok(None)
    }

    pub fn entity_attach<T: Entity>(&mut self, mut entity: T) {
        let info = entity.get_info_mutable();
        info.index = self.entity_index;
        self.entity_index += 1;
        self.entity_attach.push(Box::new(entity));
    }

    pub fn entity_detach<T: Entity>(&mut self, entity: &mut T) {
        entity.get_info_mutable().close = true;
        // TO-DO run entity.close() destructor here?
    }

    pub fn entity_find(&self, index: usize) -> Option<&Box<dyn Entity>> {
        self.entity_list
            .iter()
            .find(|entity| entity.get_info().index == index)
            .map(|v| v as _)
    }

    pub fn entity_find_mutable(&mut self, index: usize) -> Option<&mut Box<dyn Entity>> {
        self.entity_list
            .iter_mut()
            .find(|entity| entity.get_info().index == index)
            .map(|v| v as _)
    }

    pub fn main(
        &mut self,
        state: &mut State,
        draw: &mut RaylibDrawHandle<'_>,
        context: &mut Context,
    ) -> anyhow::Result<()> {
        let world = self as *mut Self;
        let pause = state.layout.is_some();

        if !pause {
            let frame_time = context.handle.get_frame_time().min(0.25);

            self.step += frame_time;

            while self.step >= Self::TIME_STEP {
                self.scene.physical.tick();

                // improve this API, please.
                if let Ok(lock) = &self.scene.physical.collision_handler.collision_list.lock() {
                    for event in lock.iter() {
                        // for each event, find entity A and B's rigid body. then, get their user data.
                        // the user data is the entity's index. find the entity, then call each other's "touch" method
                        // with each other as the "other" argument.
                        println!("{event:?}");
                    }
                }

                unsafe {
                    for entity in &mut self.entity_list {
                        entity.tick(state, &mut context.handle, &mut *world)?;
                    }
                }

                if !self.entity_attach.is_empty() {
                    self.entity_list.append(&mut self.entity_attach);
                }

                self.entity_list
                    .retain_mut(|entity| !entity.get_info().close);

                self.time += Self::TIME_STEP;
                self.step -= Self::TIME_STEP;
            }
        }

        self.scene.update(state)?;

        unsafe {
            let context = context as *mut Context;

            if !pause {
                self.scene.draw_r3d(&mut *context, |_| {
                    for entity in &mut self.entity_list {
                        entity.draw_r3d(state, &mut *context, &mut *world)?;
                    }

                    Ok(())
                })?;

                self.scene.draw_3d(&mut *context, draw, |draw| {
                    for entity in &mut self.entity_list {
                        entity.draw_3d(state, draw, &mut *world)?;
                    }

                    Ok(())
                })?;
            }

            self.scene.draw_2d(&mut *context, draw, |draw| {
                if !pause {
                    for entity in &mut self.entity_list {
                        entity.draw_2d(state, draw, &mut *world)?;
                    }
                }

                Ok(())
            })?;
        }

        Ok(())
    }
}

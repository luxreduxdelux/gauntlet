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

use crate::app::*;
use crate::entity::implementation::*;
use crate::scene::*;

//================================================================

use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::Deserialize;

//================================================================

#[derive(Default)]
pub struct World<'a> {
    pub entity_list: Vec<Box<dyn Entity>>,
    pub time: f32,
    step: f32,
    entity_index: usize,
    entity_attach: Vec<Box<dyn Entity>>,
    pub scene: Scene<'a>,
    pub player: Option<usize>,
}

impl<'a> World<'a> {
    pub const TIME_STEP: f32 = 1.0 / 60.0;

    pub fn new(app: &mut App, context: &mut Context) -> anyhow::Result<Self> {
        let mut world = World::default();

        world.scene.initialize(app, context)?;

        if app.user.tutorial {
            let level = Level::new("data/level/tutorial/tutorial.json")?;

            for model in &level.level {
                world
                    .scene
                    .room_add(context, &format!("data/level/tutorial/{model}"))?;
            }

            world.fuse_level(level);
        } else {
            // TO-DO random level generation algorithm goes here.
        }

        // entity index 0 is meant for the level's rigid-body.
        world.entity_index = 1;

        unsafe {
            let wrl = &mut world as *mut Self;
            let ctx = context as *mut Context;

            for entity in &mut world.entity_list {
                world.entity_index += 1;
                entity.get_info_mutable().index = world.entity_index - 1;
                entity.create(app, &mut *ctx, &mut *wrl)?;
            }
        }

        world.scene.link()?;

        Ok(world)
    }

    pub fn main(
        &mut self,
        app: &mut App,
        draw: &mut RaylibDrawHandle<'_>,
        context: &mut Context,
    ) -> anyhow::Result<()> {
        let world = self as *mut Self;
        let pause = app.layout.is_some();

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
                        entity.tick(app, context, &mut *world)?;
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

        self.scene.update(app, context)?;

        unsafe {
            if !pause {
                self.scene.draw_3d(context, draw, |draw| {
                    for entity in &mut self.entity_list {
                        entity.draw_3d(app, draw, &mut *world)?;
                    }

                    Ok(())
                })?;
            }

            self.scene.draw_2d(context, draw, |draw| {
                if !pause {
                    for entity in &mut self.entity_list {
                        entity.draw_2d(app, draw, &mut *world)?;
                    }
                }

                Ok(())
            })?;
        }

        Ok(())
    }

    pub fn entity_from_collider(
        &self,
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

    pub fn entity_from_collider_mutable(
        &mut self,
        collider: ColliderHandle,
    ) -> anyhow::Result<Option<&mut Box<dyn Entity>>> {
        let collider = self.scene.physical.get_collider(collider)?;

        if let Some(parent) = collider.parent()
            && let Ok(rigid) = self.scene.physical.get_rigid(parent)
        {
            return Ok(self.entity_find_mutable(rigid.user_data as usize));
        }

        Ok(None)
    }

    pub fn entity_attach<T: Entity>(
        &mut self,
        app: &mut App,
        context: &mut Context,
        mut entity: T,
    ) -> anyhow::Result<usize> {
        let ctx = { context as *mut Context };

        self.entity_index += 1;
        entity.get_info_mutable().index = self.entity_index - 1;
        entity.create(app, unsafe { &mut *ctx }, self)?;
        self.entity_attach.push(Box::new(entity));

        Ok(self.entity_index - 1)
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

    pub fn entity_find_type<T: Entity>(&self, index: usize) -> Option<&T> {
        self.entity_list
            .iter()
            .find(|entity| entity.get_info().index == index && entity.as_any().is::<T>())
            .map(|v| v.as_any().downcast_ref::<T>().unwrap())
    }

    pub fn entity_find_mutable_type<T: Entity>(&mut self, index: usize) -> Option<&mut T> {
        self.entity_list
            .iter_mut()
            .find(|entity| entity.get_info().index == index && entity.as_any().is::<T>())
            .map(|v| v.as_any_mut().downcast_mut::<T>().unwrap())
    }

    fn fuse_level(&mut self, level: Level) {
        self.entity_list.extend(level.entity_list);
    }
}

//================================================================

#[derive(Deserialize)]
struct Level {
    level: Vec<String>,
    entity_list: Vec<Box<dyn Entity>>,
}

impl Level {
    fn new(path: &str) -> anyhow::Result<Self> {
        let file = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }
}

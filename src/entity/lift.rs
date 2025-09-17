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
use crate::state::*;
use crate::utility::*;
use crate::world::*;

//================================================================

use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Serialize, Deserialize)]
pub struct Lift {
    point: Vector3,
    angle: Vector3,
    #[serde(skip)]
    scale: f32,
    #[serde(skip)]
    collider: ColliderHandle,
    #[serde(skip)]
    info: EntityInfo,
}

impl Lift {}

#[typetag::serde]
impl Entity for Lift {
    fn get_info(&self) -> &EntityInfo {
        &self.info
    }
    fn get_info_mutable(&mut self) -> &mut EntityInfo {
        &mut self.info
    }

    fn initialize(
        &mut self,
        state: &mut State,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.collider = world.scene.physical.new_cuboid(Vector3::new(1.0, 0.25, 1.0));
        world.scene.physical
            .set_collider_point(self.collider, self.point)?;

        world
            .scene
            .asset
            .set_model(context, "data/video/lift.glb")?;

        Ok(())
    }

    fn draw_r3d(
        &mut self,
        state: &mut State,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let direction = Direction::new_from_angle(&self.angle);
        let point = self.point + direction.y * ease_in_out_cubic(self.scale) * 2.0;
        let model = world.scene.asset.get_model("data/video/lift.glb")?;

        model.draw(&mut context.r3d, point, 1.0);

        Ok(())
    }

    fn tick(
        &mut self,
        _state: &mut State,
        _handle: &mut RaylibHandle,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let direction = Direction::new_from_angle(&self.angle);

        let up = world.scene.physical.cast_cuboid(
            self.point,
            Vector3::new(1.0, 0.5, 1.0),
            direction.y,
            3.0,
            QueryFilter::default()
                .exclude_sensors()
                .exclude_collider(self.collider),
        );

        // TO-DO exclude level geometry as well.
        if let Some((collider, _)) = up {
            self.scale += World::TIME_STEP * 1.0;
        } else {
            self.scale -= World::TIME_STEP * 1.0;
        }

        self.scale = self.scale.clamp(0.0, 1.0);

        let point = self.point + direction.y * ease_in_out_cubic(self.scale) * 2.0;

        world.scene.physical.set_collider_point(self.collider, point)?;

        Ok(())
    }
}

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
use crate::physical::*;
use crate::scene::View;
use crate::utility::*;
use crate::world::*;

//================================================================

use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Serialize, Deserialize)]
pub struct Door {
    point: Vector3,
    angle: Vector3,
    #[serde(skip)]
    presence: Presence,
    #[serde(skip)]
    solid: ColliderHandle,
    #[serde(skip)]
    scale: f32,
    #[serde(skip)]
    view: usize,
    #[serde(skip)]
    info: EntityInfo,
}

impl Door {
    const CUBOID_SCALE: Vector3 = Vector3::new(0.6, 1.2, 0.2);
}

#[typetag::serde]
impl Entity for Door {
    fn get_info(&self) -> &EntityInfo {
        &self.info
    }
    fn get_info_mutable(&mut self) -> &mut EntityInfo {
        &mut self.info
    }

    fn create(
        &mut self,
        _app: &mut App,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let direction = Direction::new_from_angle(&self.angle);

        self.presence = Presence::new_rigid_cuboid_fixed(
            &mut world.scene.physical,
            self.point - direction.z * 0.6,
            Vector3::zero(),
            Self::CUBOID_SCALE,
            &self.info,
        )?;

        self.solid = self.presence.attach_cuboid(
            &mut world.scene.physical,
            direction.z * 1.2,
            Self::CUBOID_SCALE,
        )?;

        world
            .scene
            .physical
            .set_collider_angle(self.presence.collider, self.angle);
        world
            .scene
            .physical
            .set_collider_angle(self.solid, self.angle);

        world.scene.set_model(context, "data/video/door_a.glb")?;
        world.scene.set_model(context, "data/video/door_b.glb")?;

        self.view = View::new(&mut world.scene, self.point, self.angle)?;

        Ok(())
    }

    fn draw_3d(
        &mut self,
        app: &mut App,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let direction = Direction::new_from_angle(&self.angle);
        let point_a = self.point + direction.z * ease_in_out_cubic(self.scale) * 1.00;
        let point_b = self.point - direction.z * ease_in_out_cubic(self.scale) * 1.35;

        let model_a = world.scene.asset.get_model("data/video/door_a.glb")?;

        let color = if app.user.debug_draw_entity {
            Color::new(255, 255, 255, 33)
        } else {
            Color::WHITE
        };

        draw.draw_model_ex(
            &model_a.model,
            point_a,
            direction.y,
            self.angle.x,
            Vector3::one(),
            color,
        );

        let model_b = world.scene.asset.get_model("data/video/door_b.glb")?;

        draw.draw_model_ex(
            &model_b.model,
            point_b,
            direction.y,
            self.angle.x,
            Vector3::one(),
            color,
        );

        if app.user.debug_draw_entity {
            draw.draw_cube_v(
                self.point,
                (Self::CUBOID_SCALE + Vector3::new(0.0, 0.0, 2.0)) * 0.5,
                Color::new(255, 0, 0, 33),
            );
        }

        Ok(())
    }

    fn tick(
        &mut self,
        app: &mut App,
        _context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let cast = world.scene.physical.intersect_cuboid(
            self.point,
            self.angle,
            Self::CUBOID_SCALE + Vector3::new(0.0, 0.0, 2.0),
            Some(self.presence.rigid),
            QueryFilter::default().groups(InteractionGroups::new(
                Physical::GROUP_ENTITY,
                Physical::GROUP_ENTITY,
            )),
        );

        if cast.is_some() {
            self.scale += World::TIME_STEP * 3.0;
        } else {
            self.scale -= World::TIME_STEP * 3.0;
        }

        self.scale = self.scale.clamp(0.0, 1.0);

        let view = &mut world.scene.view_list[self.view];
        view.visible = self.scale > 0.0;

        let direction = Direction::new_from_angle(&self.angle);
        let point_a = direction.z * ease_in_out_cubic(self.scale) * 1.00 * -1.0;
        let point_b = direction.z * ease_in_out_cubic(self.scale) * 1.00 + direction.z * 1.2;

        world
            .scene
            .physical
            .set_collider_point(self.presence.collider, point_a)?;
        world
            .scene
            .physical
            .set_collider_point(self.solid, point_b)?;

        Ok(())
    }
}

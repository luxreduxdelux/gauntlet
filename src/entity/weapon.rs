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
use crate::entity::player::*;
use crate::physical::*;
use crate::helper::Direction;
use crate::helper::draw_model_transform;
use crate::world::*;

//================================================================

use rapier3d::prelude::QueryFilter;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct Weapon {
    point: Vector3,
    angle: Vector3,
    #[serde(skip, default = "Weapon::ammo_default")]
    ammo: u32,
    #[serde(skip)]
    grab: f32,
    #[serde(skip)]
    force: Option<Vector3>,
    #[serde(skip)]
    presence: Presence,
    #[serde(skip)]
    info: EntityInfo,
}

impl Weapon {
    fn ammo_default() -> u32 {
        8
    }
}

#[typetag::serde]
impl Entity for Weapon {
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
        world
            .scene
            .asset
            .set_model(context, "data/video/weapon.glb")?;

        self.presence = Presence::new_rigid_cuboid_dynamic(
            &mut world.scene.physical,
            self.point,
            self.angle,
            Vector3::new(0.10, 0.25, 0.35),
            &self.info,
        )?;

        let rigid = world
            .scene
            .physical
            .get_rigid_mutable(self.presence.rigid)?;
        rigid.enable_ccd(true);

        if let Some(force) = &mut self.force {
            world
                .scene
                .physical
                .apply_rigid_impulse(self.presence.rigid, *force)?;

            //self.gone = None;
        }

        Ok(())
    }

    fn remove(
        &mut self,
        _app: &mut App,
        _context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.presence.remove(&mut world.scene.physical);

        Ok(())
    }

    fn draw_3d(
        &mut self,
        _app: &mut App,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let model = world.scene.asset.get_model("data/video/weapon.glb")?;

        let transform = world
            .scene
            .physical
            .get_rigid_transform(self.presence.rigid)?;

        draw_model_transform(draw, model, transform);

        Ok(())
    }

    fn tick(
        &mut self,
        _app: &mut App,
        _context: &mut Context,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        self.grab = (self.grab - World::TIME_STEP).max(0.0);

        Ok(())
    }

    fn interact(
        &mut self,
        app: &mut App,
        context: &mut Context,
        world: &mut World,
        other: &mut dyn Entity,
    ) -> anyhow::Result<()> {
        if let Some(player) = other.as_any_mut().downcast_mut::<Player>() {
            player.wield = Some(Box::new(self.clone()));
            self.detach(app, context, world)?;
        }

        Ok(())
    }
}

impl Wield for Weapon {
    fn draw_2d(
        &mut self,
        _app: &mut App,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        draw.draw_text("Wielding weapon!", 8, 8 + 32 * 0, 32, Color::RED);
        draw.draw_text(
            &format!("Ammo: {}", self.ammo),
            8,
            8 + 32 * 1,
            32,
            Color::RED,
        );

        Ok(())
    }

    fn tick(
        &mut self,
        app: &mut App,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.grab = (self.grab - World::TIME_STEP).max(0.0);

        let wrl = { world as *mut World };

        if app.user.input_push.get_press()
            && self.ammo > 0
            && let Some(player) = world.player
            && let Some(player) = world.entity_find_mutable_type::<Player>(player)
        {
            let angle = Direction::new_from_angle(&player.angle);

            unsafe {
                let cast = (*wrl).scene.physical.cast_ray(
                    (*wrl).scene.camera_3d.position,
                    angle.x,
                    16.0,
                    true,
                    Some(player.presence.rigid),
                    QueryFilter::default().exclude_sensors(),
                );

                if let Some((collider, _)) = cast
                    && let Ok(Some(entity)) = (*wrl).entity_from_collider_mutable(collider)
                {
                    entity.damage(app, context, &mut *wrl, player, 1)?;
                }
            }

            self.ammo -= 1;
        }

        let point = world.scene.camera_3d.position;

        if app.user.input_pull.get_press()
            && let Some(player) = world.player
            && let Some(player) = world.entity_find_mutable_type::<Player>(player)
        {
            let angle = Direction::new_from_angle(&player.angle);

            let weapon = Weapon {
                point: point + angle.x * 2.0,
                angle: Vector3::default(),
                ammo: self.ammo,
                grab: 0.25,
                force: Some(angle.x * 2.0),
                presence: Presence::default(),
                info: EntityInfo::default(),
            };

            player.wield = None;
            world.entity_attach(app, context, *Box::new(weapon))?;
        }

        Ok(())
    }
}

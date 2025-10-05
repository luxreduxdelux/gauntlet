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
use crate::scene::Room;
use crate::utility::*;
use crate::world::*;

//================================================================

use rapier3d::control::KinematicCharacterController;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Default)]
enum EnemyState {
    #[default]
    Idle,
    Walk,
    Fire,
}

impl EnemyState {
    fn update(world: &mut World, enemy: &mut Enemy) {
        match enemy.app {
            Self::Idle => {
                // if player is in same room as us, check if we have line of sight.
                // if we do, check if we can fire, or if we can't,
                // then get a path to player, and transition to walk app.

                if let Some(index) = world.player
                    && let Some(player) = world.entity_find(index)
                    && let Some(other) = player.as_any().downcast_ref::<Player>()
                {
                    let enemy_room = Room::active_index(&world.scene, enemy.point);
                    let other_room = Room::active_index(&world.scene, other.point);

                    if let Some(e_room) = enemy_room
                        && let Some(o_room) = other_room
                        && e_room == o_room
                    {}
                }
            }
            Self::Walk => {
                // if we have line of sight to player, then fire.
                // if we don't, then walk the entire path to player,
                // constantly checking if we can fire at player.
            }
            Self::Fire => {
                // fire at player if we have line of sight. otherwise,
                // go to walk app.
            }
        }
    }

    fn to_idle(world: &mut World, enemy: &mut Enemy) -> anyhow::Result<()> {
        enemy.app = Self::Idle;
        enemy.animation = Animation::new(
            world.scene.asset.get_model("data/video/test.glb")?,
            "Idle_Loop",
            60.0,
        );

        Ok(())
    }

    fn to_walk(world: &mut World, enemy: &mut Enemy) -> anyhow::Result<()> {
        enemy.app = Self::Walk;
        enemy.animation = Animation::new(
            world.scene.asset.get_model("data/video/test.glb")?,
            "Walk_Loop",
            60.0,
        );

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Enemy {
    point: Vector3,
    angle: Vector3,
    #[serde(skip)]
    presence: Presence,
    #[serde(skip)]
    character: KinematicCharacterController,
    #[serde(skip)]
    animation: Animation,
    #[serde(skip)]
    app: EnemyState,
    #[serde(skip)]
    info: EntityInfo,
}

#[typetag::serde]
impl Entity for Enemy {
    fn get_info(&self) -> &EntityInfo {
        &self.info
    }
    fn get_info_mutable(&mut self) -> &mut EntityInfo {
        &mut self.info
    }

    #[rustfmt::skip]
    fn create(
        &mut self,
        _app: &mut App,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        unsafe {
            let ctx = context as *mut Context;
            world.scene.asset.set_sound(&*ctx, "data/audio/step_0.wav", 0)?;
            world.scene.asset.set_sound(&*ctx, "data/audio/step_1.wav", 0)?;
            world.scene.asset.set_sound(&*ctx, "data/audio/pistol_0.wav", 0)?;
            world.scene.asset.set_sound(&*ctx, "data/audio/pistol_1.wav", 0)?;
            world.scene.asset.set_sound(&*ctx, "data/audio/fall.wav", 0)?;
        }

        let model = world
            .scene
            .asset
            .set_model(context, "data/video/test.glb")?;

        self.presence = Presence::new_rigid_cuboid_fixed(
            &mut world.scene.physical,
            self.point,
            Vector3::default(),
            Vector3::new(0.25, 0.50, 0.25),
            &self.info,
        )?;

        self.character = KinematicCharacterController::default();
        self.character.snap_to_ground = None;

        self.animation = Animation::new(model, "Idle_Loop", 60.0);

        Ok(())
    }

    fn draw_3d(
        &mut self,
        _app: &mut App,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let model = world.scene.asset.get_model("data/video/test.glb")?;

        if let Ok(Some((point, _, _))) = self.animation.get_bone_data(model, "DEF-headtip") {
            draw.draw_cube_v(self.point + point, Vector3::one() * 0.1, Color::RED);
        }

        /*
        if world.scene.room_active_box(
            self.point,
            Vector3::default(),
            Vector3::new(0.25, 0.5, 0.25),
        ) {
            let model = world.scene.asset.get_model("data/video/test.glb")?;

            if context.handle.is_key_pressed(KeyboardKey::KEY_ONE) {
                self.animation = Animation::new(model, "Idle_Loop", 60.0);
            } else if context.handle.is_key_pressed(KeyboardKey::KEY_TWO) {
                self.animation = Animation::new(model, "Pistol_Reload", 60.0);
            } else if context.handle.is_key_pressed(KeyboardKey::KEY_THREE) {
                self.animation = Animation::new(model, "Death01", 60.0);
            }

            //model.model.draw(
            //    &mut context.r3d,
            //    self.point - Vector3::new(0.0, 0.5, 0.0),
            //    1.0,
            //);
        }
        */

        Ok(())
    }

    fn tick(
        &mut self,
        app: &mut App,
        _context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.animation
            .update(app, world, "data/video/test.glb", self.point)?;
        EnemyState::update(world, self);

        Ok(())
    }
}

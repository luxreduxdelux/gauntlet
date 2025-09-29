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

use rapier3d::control::KinematicCharacterController;
use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use super::player::Player;

//================================================================

#[derive(Serialize, Deserialize)]
pub struct Dummy {
    point: Vector3,
    angle: Vector3,
    #[serde(skip)]
    animation: Animation,
    #[serde(skip)]
    info: EntityInfo,
}

impl Dummy {
    const BONE_TORSO: [&str; 5] = [
        "DEF-shoulderR",
        "DEF-shoulderL",
        "DEF-spine003",
        "DEF-spine002",
        "DEF-spine001",
    ];

    const BONE_L_ARM: [&str; 3] = ["DEF-upper_armL", "DEF-forearmL", "DEF-handL"];

    const BONE_R_ARM: [&str; 3] = ["DEF-upper_armR", "DEF-forearmR", "DEF-handR"];
}

#[typetag::serde]
impl Entity for Dummy {
    fn get_info(&self) -> &EntityInfo {
        &self.info
    }
    fn get_info_mutable(&mut self) -> &mut EntityInfo {
        &mut self.info
    }

    #[rustfmt::skip]
    fn initialize(
        &mut self,
        state: &mut State,
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

        self.animation = Animation::new(model, "Idle_Loop", 60.0);

    // --- Helper to spawn body part ---
    let mut spawn_part = |point: Vector3, scale: Vector3| {
        let rigid = world.scene.physical.new_rigid_dynamic();
        world.scene.physical.set_rigid_point(rigid, self.point + point).unwrap();
        let solid = world.scene.physical.new_cuboid(scale, Some(rigid));
        let solid = world.scene.physical.get_collider_mutable(solid).unwrap();
        solid.set_density(1.0);
        rigid
    };

    // --- Body parts ---
    let torso = spawn_part(Vector3::new(0.0, 1.5, 0.0), Vector3::new(0.3, 0.5, 0.15));
    let head = spawn_part(Vector3::new(0.0, 2.3, 0.0), Vector3::new(0.2, 0.2, 0.2));

    let left_arm = spawn_part(Vector3::new(-0.6, 1.5, 0.0), Vector3::new(0.15, 0.4, 0.15));
    let right_arm = spawn_part(Vector3::new(0.6, 1.5, 0.0), Vector3::new(0.15, 0.4, 0.15));

    let left_leg = spawn_part(Vector3::new(-0.2, 0.5, 0.0), Vector3::new(0.15, 0.5, 0.15));
    let right_leg = spawn_part(Vector3::new(0.2, 0.5, 0.0), Vector3::new(0.15, 0.5, 0.15));

    // --- Joints ---
    // Head to torso
    let joint = SphericalJointBuilder::new().local_anchor1(point![0.0, 0.5, 0.0]).local_anchor2(point![0.0, -0.2, 0.0]);
    world.scene.physical.impulse_joint_set.insert(torso, head, joint, true);

    // Arms to torso
    let left_joint = SphericalJointBuilder::new()
        .local_anchor1(point![-0.3, 0.3, 0.0])
        .local_anchor2(point![0.0, 0.4, 0.0]);
    world.scene.physical.impulse_joint_set.insert(torso, left_arm, left_joint, true);

    let right_joint = SphericalJointBuilder::new()
        .local_anchor1(point![0.3, 0.3, 0.0])
        .local_anchor2(point![0.0, 0.4, 0.0]);
    world.scene.physical.impulse_joint_set.insert(torso, right_arm, right_joint, true);

    // Legs to torso
    let left_leg_joint = SphericalJointBuilder::new()
        .local_anchor1(point![-0.2, -0.5, 0.0])
        .local_anchor2(point![0.0, 0.5, 0.0]);
    world.scene.physical.impulse_joint_set.insert(torso, left_leg, left_leg_joint, true);

    let right_leg_joint = SphericalJointBuilder::new()
        .local_anchor1(point![0.2, -0.5, 0.0])
        .local_anchor2(point![0.0, 0.5, 0.0]);
    world.scene.physical.impulse_joint_set.insert(torso, right_leg, right_leg_joint, true);

        /*
        for bone in Self::BONE_TORSO {
            let point = self.animation.get_bone_data(model, bone)?;
            let joint = SphericalJointBuilder::new();
        }
        */

        Ok(())
    }

    fn draw_r3d(
        &mut self,
        state: &mut State,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
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

        Ok(())
    }

    fn draw_3d(
        &mut self,
        state: &mut State,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let model = world.scene.asset.get_model("data/video/test.glb")?;

        /*
        if let Ok(Some(point)) = self.animation.get_bone_data(model, "DEF-headtip") {
            draw.draw_cube_v(
                self.point - Vector3::new(0.0, 0.5, 0.0) + point,
                Vector3::one() * 0.1,
                Color::RED,
            );
        }
        */

        Ok(())
    }

    fn tick(
        &mut self,
        state: &mut State,
        handle: &mut RaylibHandle,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.animation
            .update(state, world, "data/video/test.glb", self.point)?;

        Ok(())
    }
}

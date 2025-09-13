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
use crate::external::r3d::LightType;
use crate::state::*;
use crate::utility::*;
use crate::world::*;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Light {
    point: Vector3,
    angle: Vector3,
    mode: LightType,
    color: Color,
    #[serde(skip)]
    speed: Vector3,
    #[serde(skip)]
    handle: Option<crate::external::r3d::Light>,
}

impl Light {}

#[typetag::serde]
impl Entity for Light {
    fn initialize(
        &mut self,
        _state: &mut State,
        context: &mut Context,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        let mut light = crate::external::r3d::Light::new(&mut context.r3d, self.mode);

        light.set_active(true);
        light.set_color(self.color);

        let direction = Direction::new_from_angle(&self.angle);

        light.set_shadow_depth_bias(light.get_shadow_depth_bias() * 16.0);
        light.set_shadow_update_mode(crate::external::r3d::ShadowUpdateMode::Manual);
        light.enable_shadow(512);
        light.look_at(self.point, self.point + direction.x);

        self.handle = Some(light);

        Ok(())
    }

    fn get_point(&mut self) -> &mut Vector3 {
        &mut self.point
    }

    fn get_angle(&mut self) -> &mut Vector3 {
        &mut self.angle
    }

    fn get_speed(&mut self) -> &mut Vector3 {
        &mut self.speed
    }
}

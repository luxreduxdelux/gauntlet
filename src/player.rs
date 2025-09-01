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
use crate::state::*;

//================================================================

use raylib::prelude::*;

//================================================================

#[derive(Default)]
pub struct Player {
    point: Vector3,
    angle: Vector3,
    speed: Vector3,
}

pub struct Direction {
    pub forward: Vector3,
    pub up: Vector3,
    pub right: Vector3,
}

impl Direction {
    pub fn new_from_angle(angle: &Vector3) -> Self {
        // angle.x - pitch
        // angle.y - yaw

        let angle = Vector3::new(
            angle.x.to_radians(),
            angle.y.to_radians(),
            angle.z.to_radians(),
        );

        let forward = Vector3::new(
            angle.y.cos() * angle.x.sin(),
            angle.y.sin() * -1.0,
            angle.y.cos() * angle.x.cos(),
        );

        let up = Vector3::new(
            angle.y.sin() * angle.x.sin(),
            angle.y.cos(),
            angle.y.sin() * angle.x.cos(),
        );

        let right = Vector3::new(angle.x.cos(), 0.0, angle.x.sin() * -1.0);

        Self { forward, up, right }
    }
}

impl Entity for Player {
    fn get_point(&mut self) -> &mut Vector3 {
        &mut self.point
    }

    fn get_angle(&mut self) -> &mut Vector3 {
        &mut self.angle
    }

    fn get_speed(&mut self) -> &mut Vector3 {
        &mut self.speed
    }

    #[rustfmt::skip]
    fn draw_3d(&mut self, state: &mut State, draw: &mut RaylibMode3D<'_, RaylibDrawHandle<'_>>) {
        let mouse = draw.get_mouse_delta();

        self.angle.x -= mouse.x * 0.1;
        self.angle.y += mouse.y * 0.1;

        let direction = Direction::new_from_angle(&self.angle);

        state.camera_3d.position = self.point;
        state.camera_3d.target   = self.point + direction.forward;
        state.camera_3d.up       = direction.up;
    }

    fn tick(&mut self, state: &mut State, handle: &mut RaylibHandle) {
        let direction = Direction::new_from_angle(&self.angle);

        if handle.is_key_down(KeyboardKey::KEY_W) {
            self.point += direction.forward * State::TIME_STEP * 10.0;
        }

        if handle.is_key_down(KeyboardKey::KEY_S) {
            self.point -= direction.forward * State::TIME_STEP * 10.0;
        }
    }
}

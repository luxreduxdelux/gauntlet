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
use crate::world::*;

use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
enum TutorialKind {
    #[default]
    Move,
    Jump,
    Duck,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Tutorial {
    point: Vector3,
    scale: Vector3,
    which: TutorialKind,
    #[serde(skip)]
    collider: ColliderHandle,
    #[serde(skip)]
    index: usize,
}

impl Tutorial {}

#[typetag::serde]
impl Entity for Tutorial {
    fn get_index(&mut self) -> &mut usize {
        &mut self.index
    }

    fn initialize(
        &mut self,
        _state: &mut State,
        _context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.collider = world.physical.new_cuboid(self.scale);
        world
            .physical
            .set_collider_point(self.collider, self.point)?;
        world.physical.set_collider_sensor(self.collider, true)?;

        Ok(())
    }

    fn draw_2d(
        &mut self,
        _state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        let half = Vector2::new(
            draw.get_render_width() as f32 * 0.5,
            draw.get_render_height() as f32 * 0.5,
        );

        match self.which {
            TutorialKind::Move => {
                draw.draw_text("foo", half.x as i32, half.y as i32, 32, Color::RED);
            }
            TutorialKind::Jump => {
                draw.draw_text("bar", half.x as i32, half.y as i32, 32, Color::RED);
            }
            TutorialKind::Duck => {
                draw.draw_text("baz", half.x as i32, half.y as i32, 32, Color::RED);
            }
        }

        Ok(())
    }
}

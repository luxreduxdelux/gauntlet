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
use crate::external::r3d::*;
use crate::utility::*;
use crate::window::Window;
use crate::world::*;

//================================================================

use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

//================================================================

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Default)]
enum LightKind {
    // 0
    #[default]
    Normal,
    // 1
    FlickerA,
    // 6
    FlickerB,
    // 10
    FlickerC,
    // 3
    CandleA,
    // 7
    CandleB,
    // 8
    CandleC,
    // 2
    PulseA,
    // 4
    PulseB,
    // 5
    PulseC,
    // 9
    PulseD,
    // 11
    PulseE,
}

impl Display for LightKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            LightKind::Normal => "Normal",
            LightKind::FlickerA => "FlickerA",
            LightKind::FlickerB => "FlickerB",
            LightKind::FlickerC => "FlickerC",
            LightKind::CandleA => "CandleA",
            LightKind::CandleB => "CandleB",
            LightKind::CandleC => "CandleC",
            LightKind::PulseA => "PulseA",
            LightKind::PulseB => "PulseB",
            LightKind::PulseC => "PulseC",
            LightKind::PulseD => "PulseD",
            LightKind::PulseE => "PulseE",
        };

        f.write_str(string)
    }
}

/*
def light_style(string):
    print("&[")
    for character in string:
        index = ord(character)
        value = (index - 97) / (122 - 97)
        print(value, ",")
    print("]")
*/

impl LightKind {
    fn animation(&self) -> &[f32] {
        match self {
            LightKind::Normal => &[],
            LightKind::FlickerA => &[
                0.48, 0.48, 0.52, 0.48, 0.48, 0.56, 0.48, 0.48, 0.56, 0.48, 0.48, 0.52, 0.56, 0.52,
                0.48, 0.48, 0.56, 0.52, 0.64, 0.52, 0.48, 0.48, 0.56,
            ],
            LightKind::FlickerB => &[
                0.52, 0.48, 0.56, 0.52, 0.64, 0.52, 0.48, 0.56, 0.48, 0.52, 0.48, 0.56, 0.48, 0.56,
                0.48, 0.52, 0.56,
            ],
            LightKind::FlickerC => &[
                0.48, 0.48, 0.0, 0.48, 0.0, 0.48, 0.48, 0.48, 0.48, 0.0, 0.48, 0.48, 0.0, 0.48,
                0.0, 0.48, 0.0, 0.0, 0.0, 0.48, 0.0, 0.48, 0.48, 0.48, 0.0,
            ],
            LightKind::CandleA => &[
                0.48, 0.48, 0.48, 0.48, 0.48, 0.0, 0.0, 0.0, 0.0, 0.0, 0.48, 0.48, 0.48, 0.48,
                0.48, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.04, 0.08, 0.12, 0.16, 0.2, 0.24, 0.0, 0.04,
                0.08, 0.12, 0.16, 0.2, 0.24,
            ],
            LightKind::CandleB => &[
                0.48, 0.48, 0.48, 0.0, 0.0, 0.0, 0.0, 0.04, 0.08, 0.12, 0.16, 0.2, 0.24, 0.48,
                0.48, 0.48, 0.48, 0.0, 0.0, 0.0, 0.0, 0.48, 0.48, 0.48, 0.0, 0.0, 0.48, 0.48,
            ],
            LightKind::CandleC => &[
                0.48, 0.48, 0.48, 0.0, 0.0, 0.0, 0.48, 0.48, 0.48, 0.0, 0.0, 0.0, 0.48, 0.48, 0.48,
                0.0, 0.04, 0.08, 0.12, 0.16, 0.2, 0.0, 0.0, 0.0, 0.0, 0.48, 0.48, 0.48, 0.48, 0.0,
                0.04, 0.08, 0.12, 0.16, 0.2, 0.48, 0.48, 0.48, 0.0, 0.0, 0.0, 0.0,
            ],
            LightKind::PulseA => &[
                0.0, 0.04, 0.08, 0.12, 0.16, 0.2, 0.24, 0.28, 0.32, 0.36, 0.4, 0.44, 0.48, 0.52,
                0.56, 0.6, 0.64, 0.68, 0.72, 0.76, 0.8, 0.84, 0.88, 0.92, 0.96, 1.0, 0.96, 0.92,
                0.88, 0.84, 0.8, 0.76, 0.72, 0.68, 0.64, 0.6, 0.56, 0.52, 0.48, 0.44, 0.4, 0.36,
                0.32, 0.28, 0.24, 0.2, 0.16, 0.12, 0.08, 0.04, 0.0,
            ],
            LightKind::PulseB => &[
                0.48, 0.0, 0.48, 0.0, 0.48, 0.0, 0.48, 0.0, 0.48, 0.0, 0.48, 0.0,
            ],
            LightKind::PulseC => &[
                0.36, 0.4, 0.44, 0.48, 0.52, 0.56, 0.6, 0.64, 0.68, 0.72, 0.76, 0.8, 0.84, 0.88,
                0.92, 0.96, 1.0, 0.96, 0.92, 0.88, 0.84, 0.8, 0.76, 0.72, 0.68, 0.64, 0.6, 0.56,
                0.52, 0.48, 0.44, 0.4, 0.36,
            ],
            LightKind::PulseD => &[
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ],
            LightKind::PulseE => &[
                0.0, 0.04, 0.08, 0.12, 0.16, 0.2, 0.24, 0.28, 0.32, 0.36, 0.4, 0.44, 0.48, 0.52,
                0.56, 0.6, 0.64, 0.68, 0.68, 0.64, 0.6, 0.56, 0.52, 0.48, 0.44, 0.4, 0.36, 0.32,
                0.28, 0.24, 0.2, 0.16, 0.12, 0.08, 0.04, 0.0,
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Light {
    point: Vector3,
    angle: Vector3,
    mode: LightType,
    kind: LightKind,
    color: Color,
    #[serde(skip)]
    frame: f32,
    #[serde(skip)]
    focus: bool,
    #[serde(skip)]
    handle: Option<crate::external::r3d::Light>,
    #[serde(skip)]
    info: EntityInfo,
}

#[typetag::serde]
impl Entity for Light {
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
        _world: &mut World,
    ) -> anyhow::Result<()> {
        let mut light = crate::external::r3d::Light::new(&mut context.r3d, self.mode);

        light.set_active(true);
        light.set_color(self.color);
        light.set_specular(0.0);

        let direction = Direction::new_from_angle(&self.angle);

        light.set_shadow_depth_bias(light.get_shadow_depth_bias() * 4.0);
        light.set_shadow_update_mode(crate::external::r3d::ShadowUpdateMode::Manual);
        light.enable_shadow(256);
        light.look_at(self.point, self.point + direction.x);

        self.handle = Some(light);

        Ok(())
    }

    fn draw_r3d(
        &mut self,
        _app: &mut App,
        _context: &mut Context,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        if let Some(handle) = &mut self.handle {
            let active = _world.scene.room_active(self.point);

            if (active && !handle.is_active()) || (!active && handle.is_active()) {
                handle.set_active(active);
            }
        }

        Ok(())
    }

    fn draw_3d(
        &mut self,
        _app: &mut App,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        draw.draw_cube_v(self.point, Vector3::one() * 0.5, Color::RED);

        Ok(())
    }

    #[rustfmt::skip]
    fn draw_2d(
        &mut self,
        app: &mut App,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let focus = self.focus;

        if self.focus {
            Window::draw(app, draw, |app, draw| {
                let handle = self.handle.as_mut().unwrap();
                let color = handle.get_color();
                let mut energy = handle.get_energy();
                let mut range = handle.get_range();
                let mut attenuation = handle.get_attenuation();
                let mut r = color.r as f32;
                let mut g = color.g as f32;
                let mut b = color.b as f32;

                app.window.slider(draw, "Color (R)", &mut r, (0.0, 255.0), 1.0)?;
                app.window.slider(draw, "Color (G)", &mut g, (0.0, 255.0), 1.0)?;
                app.window.slider(draw, "Color (B)", &mut b, (0.0, 255.0), 1.0)?;
                app.window.slider(draw, "Energy", &mut energy, (0.0, 4.0), 0.1)?;
                app.window.slider(draw, "Range", &mut range, (0.0, 64.0), 1.0)?;
                app.window.slider(draw, "Attenuation", &mut attenuation, (0.0, 4.0), 0.1)?;
                app.window.switch(draw, "Kind", &mut self.kind, &[
                    LightKind::Normal,
                    LightKind::FlickerA,
                    LightKind::FlickerB,
                    LightKind::FlickerC,
                    LightKind::CandleA,
                    LightKind::CandleB,
                    LightKind::CandleC,
                    LightKind::PulseA,
                    LightKind::PulseB,
                    LightKind::PulseC,
                    LightKind::PulseD,
                    LightKind::PulseE,
                ])?;

                handle.set_energy(energy);

                handle.set_color(Color::new(r as u8, g as u8, b as u8, 255));
                handle.set_energy(energy);
                handle.set_range(range);
                handle.set_attenuation(attenuation);

                let x_a = draw.is_key_pressed(KeyboardKey::KEY_W) || draw.is_key_pressed_repeat(KeyboardKey::KEY_W);
                let x_b = draw.is_key_pressed(KeyboardKey::KEY_S) || draw.is_key_pressed_repeat(KeyboardKey::KEY_S);
                let y_a = draw.is_key_pressed(KeyboardKey::KEY_Z) || draw.is_key_pressed_repeat(KeyboardKey::KEY_Z);
                let y_b = draw.is_key_pressed(KeyboardKey::KEY_C) || draw.is_key_pressed_repeat(KeyboardKey::KEY_C);
                let z_a = draw.is_key_pressed(KeyboardKey::KEY_A) || draw.is_key_pressed_repeat(KeyboardKey::KEY_A);
                let z_b = draw.is_key_pressed(KeyboardKey::KEY_D) || draw.is_key_pressed_repeat(KeyboardKey::KEY_D);

                let mut point = self.point;

                point.x += if x_a { 1.0 } else if x_b { -1.0 } else { 0.0 };
                point.y += if y_a { 1.0 } else if y_b { -1.0 } else { 0.0 };
                point.z += if z_a { 1.0 } else if z_b { -1.0 } else { 0.0 };

                self.point = point;
                handle.set_position(point);

                if draw.is_key_pressed(KeyboardKey::KEY_Q) {
                    self.focus = false;
                    // TO-DO make this happen automatically on set_device
                    draw.disable_cursor();
                };
                Ok(())
            })?;
        }

        let ray = Ray::new(
            world.scene.camera_3d.position,
            world.scene.camera_3d.target - world.scene.camera_3d.position,
        );

        let bound = BoundingBox::new(
            self.point + Vector3::one() * 0.25 * -1.0,
            self.point + Vector3::one() * 0.25,
        );

        let collision = bound.get_ray_collision_box(ray);

        if collision.hit && collision.distance <= 8.0
            && draw.is_key_pressed(KeyboardKey::KEY_Q) && !focus {
                self.focus = true;
                app.window.set_device(crate::window::Device::Mouse { lock: true });
                // TO-DO make this happen automatically on set_device
                draw.enable_cursor();
            }

        Ok(())
    }

    fn tick(
        &mut self,
        _app: &mut App,
        _context: &mut Context,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        if let Some(light) = &mut self.handle
            && light.is_active()
        {
            let animation = self.kind.animation();

            if !animation.is_empty() {
                self.frame += World::TIME_STEP * 10.0;
                self.frame %= animation.len() as f32;

                let frame = self.frame as usize;

                if frame > 0 {
                    let frame_a = animation[frame - 1];
                    let frame_b = animation[frame];
                    let factor = (self.frame - frame as f32) / ((frame + 1) as f32 - frame as f32);
                    light
                        .set_energy(interpolate(frame_a, frame_b, ease_in_out_cubic(factor)) * 2.0);
                } else {
                    light.set_energy(animation[frame] * 2.0);
                }
            }

            light.update_shadow_map();
        }

        Ok(())
    }
}

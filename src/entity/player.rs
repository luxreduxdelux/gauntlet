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

// interpolate entity point/angle from previous frame to current to smooth out 60tick update rate

use crate::entity::implementation::*;
use crate::state::*;
use crate::user::*;
use crate::utility::*;
use crate::world::*;

//================================================================

use rand::Rng;
use rapier3d::control::KinematicCharacterController;
use rapier3d::prelude::*;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Default, Serialize, Deserialize)]
pub struct Player {
    point: Vector3,
    angle: Vector3,
    #[serde(skip)]
    speed: Vector3,
    #[serde(skip)]
    collider: ColliderHandle,
    #[serde(skip)]
    character: KinematicCharacterController,
    #[serde(skip)]
    state: PlayerState,
    #[serde(skip)]
    view: View,
    #[serde(skip)]
    floor: bool,
    #[serde(skip)]
    slide: bool,
    #[serde(skip)]
    clash: bool,
    #[serde(skip)]
    shake: f32,
    #[serde(skip)]
    push: f32,
    #[serde(skip)]
    info: EntityInfo,
}

impl Player {
    const ANGLE_MIN: f32 = -90.0;
    const ANGLE_MAX: f32 = 90.00;
    const CUBE_SHAPE: Vector3 = Vector3::new(0.25, 0.5, 0.25);
}

#[typetag::serde]
impl Entity for Player {
    fn get_info(&self) -> &EntityInfo {
        &self.info
    }
    fn get_info_mutable(&mut self) -> &mut EntityInfo {
        &mut self.info
    }

    fn initialize<'a>(
        &mut self,
        state: &mut State,
        context: &'a mut Context,
        world: &mut World<'a>,
    ) -> anyhow::Result<()> {
        self.collider = world.scene.physical.new_cuboid(Self::CUBE_SHAPE, None);
        world
            .scene
            .physical
            .set_collider_point(self.collider, self.point)?;

        self.character = KinematicCharacterController::default();

        self.view = View::new(
            Vector3::up() * 2.0,
            Vector3::default(),
            state.user.screen_field,
        );

        Ok(())
    }

    fn draw_3d(
        &mut self,
        state: &mut State,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        if draw.is_key_down(KeyboardKey::KEY_TAB) {
            world.scene.physical.draw();
        }

        //================================================================

        state.user.move_x_a.poll(draw);
        state.user.move_x_b.poll(draw);
        state.user.move_z_a.poll(draw);
        state.user.move_z_b.poll(draw);
        state.user.jump.poll(draw);
        state.user.push.poll(draw);
        state.user.pull.poll(draw);

        //================================================================

        let mouse = &draw.get_mouse_delta();

        self.angle.x -= mouse.x * 0.1 * state.user.mouse_speed;
        self.angle.y += mouse.y * 0.1 * state.user.mouse_speed;
        self.angle.x %= 359.0;
        self.angle.y = self.angle.y.clamp(Self::ANGLE_MIN, Self::ANGLE_MAX);

        //================================================================

        let shake = {
            if self.shake > 0.0 {
                let mut random = rand::rng();
                Vector3::new(
                    random.random_range(-self.shake..self.shake),
                    random.random_range(-self.shake..self.shake),
                    random.random_range(-self.shake..self.shake),
                )
            } else {
                Vector3::zero()
            }
        };

        self.view.blend(draw, &PlayerState::view(self, state, draw));

        let direction =
            Direction::new_from_angle(&(self.angle + Vector3::new(0.0, 0.0, self.view.angle.z)));
        let point = self.point + shake + self.view.point;
        let focus = point + Vector3::new(self.view.angle.x, self.view.angle.y, 0.0) + direction.x;

        world.scene.camera_3d.position = point;
        world.scene.camera_3d.target = focus;
        world.scene.camera_3d.up = direction.y;
        world.scene.camera_3d.fovy = self.view.scale;

        Ok(())
    }

    fn draw_2d(
        &mut self,
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let full = Vector2::new(
            draw.get_render_width() as f32,
            draw.get_render_height() as f32,
        );
        let half = full * 0.5;

        draw.draw_circle_v(half, 8.0, Color::new(0, 0, 0, 127));
        draw.draw_circle_sector(
            half,
            8.0,
            -90.0,
            interpolate(-90.0, 270.0, self.push),
            16,
            Color::RED,
        );
        draw.draw_circle_v(half, 4.0, Color::WHITE);

        let time = (world.time).min(1.0);
        let scale = ease_in_out_cubic(time);

        draw.draw_rectangle_v(
            Vector2::zero(),
            full,
            Color::BLACK.lerp(Color::new(0, 0, 0, 0), scale),
        );

        Ok(())
    }

    fn tick(
        &mut self,
        state: &mut State,
        handle: &mut RaylibHandle,
        world: &mut World,
    ) -> anyhow::Result<()> {
        // TO-DO slide player alongside wall
        // TO-DO bounce player off ceiling if bumping head
        // TO-DO fix being able to jump off side of rigid body
        // TO-DO fix snap-to-ground on slope

        self.shake = (self.shake - World::TIME_STEP * self.shake * 4.0).max(0.0);

        if state.user.push.down(handle) {
            self.push = (self.push + World::TIME_STEP * 1.5).min(1.0);
        } else {
            if self.push > 0.0 {
                let angle = Direction::new_from_angle(&self.angle);

                let cast = world.scene.physical.cast_ray(
                    raylib::math::Ray::new(world.scene.camera_3d.position, angle.x),
                    3.0,
                    true,
                    QueryFilter::default()
                        .exclude_sensors()
                        .exclude_collider(self.collider),
                );

                if cast.is_some() {
                    // TO-DO past a certain threshold, ignore angle on push jump and just push upward anyway?
                    let boost = angle.x * 5.0 * self.push;

                    if self.speed.y < 0.0 {
                        // allow player to pogo jump off floor if falling down
                        self.speed.x -= boost.x;
                        self.speed.y = -boost.y;
                        self.speed.z -= boost.z;
                    } else {
                        self.speed -= boost;
                    }
                } else {
                    if !self.floor {
                        if self.angle.y >= 60.0 {
                            self.speed.x = 0.0;
                            self.speed.z = 0.0;
                            self.state = PlayerState::Slam { time: 0.0 };
                        } else {
                            self.speed += angle.x * 5.0 * self.push;
                        }
                    }
                }
            }

            self.push = 0.0;
        }

        //================================================================

        PlayerState::tick(self, state, handle);

        let movement =
            world
                .scene
                .physical
                .move_controller(self.collider, self.character, self.speed)?;
        let position = if handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            self.speed * World::TIME_STEP
        } else {
            Vector3::new(
                movement.translation.x,
                movement.translation.y,
                movement.translation.z,
            )
        };

        self.clash = false;
        self.slide = movement.is_sliding_down_slope;

        if self.speed * World::TIME_STEP != position {
            self.clash = true;
        }

        self.point += position;
        self.floor = if handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            true
        } else {
            movement.grounded
        };

        state.user.move_x_a.wipe();
        state.user.move_x_b.wipe();
        state.user.move_z_a.wipe();
        state.user.move_z_b.wipe();
        state.user.jump.wipe();
        state.user.push.wipe();
        state.user.pull.wipe();

        Ok(())
    }
}

//================================================================

enum PlayerState {
    Walk { jump: f32 },
    Slam { time: f32 },
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Walk { jump: 0.0 }
    }
}

impl PlayerState {
    const SPEED_MIN: f32 = 0.10;
    const SPEED_MAX: f32 = 8.00;
    const SPEED_RISE: f32 = 4.50;
    const SPEED_FALL: f32 = 8.00;
    const SPEED_AIR_MIN: f32 = 0.50;
    const SPEED_AIR_RISE: f32 = 4.00;
    const SPEED_AIR_FALL: f32 = 8.00;
    const SPEED_JUMP: f32 = 3.00;

    fn get_movement_key(handle: &RaylibHandle, key_a: Input, key_b: Input) -> f32 {
        if key_a.down(handle) {
            return Self::SPEED_MAX;
        }

        if key_b.down(handle) {
            return -Self::SPEED_MAX;
        }

        0.0
    }

    fn tick(player: &mut Player, state: &State, handle: &RaylibHandle) {
        match player.state {
            Self::Walk { ref mut jump } => {
                *jump -= *jump * World::TIME_STEP * 4.0;

                let move_angle = Direction::new_from_angle(&Vector3::new(player.angle.x, 0.0, 0.0));
                let move_x = move_angle.x
                    * Self::get_movement_key(handle, state.user.move_x_a, state.user.move_x_b);
                let move_z = move_angle.z
                    * Self::get_movement_key(handle, state.user.move_z_a, state.user.move_z_b);
                let move_which = move_x + move_z;
                let move_where = move_which.normalized();
                let move_speed = move_which.length();

                //================================================================

                if player.floor {
                    // on-floor movement.
                    if player.speed.y != 0.0 {
                        // camera fall animation.
                        if player.speed.y <= -2.0 {
                            *jump = -0.5
                        }

                        player.speed.y = 0.0;
                    }

                    if state.user.jump.down(handle) {
                        player.speed.y = Self::SPEED_JUMP;
                        *jump = 0.5;
                    }

                    let self_speed = Vector3::new(player.speed.x, 0.0, player.speed.z);

                    if self_speed.x.abs() >= 0.0 || self_speed.z.abs() >= 0.0 {
                        let mut self_length = self_speed.length();

                        if self_length < Self::SPEED_MIN {
                            self_length = 1.0
                                - World::TIME_STEP
                                    * (Self::SPEED_MIN / self_length)
                                    * Self::SPEED_FALL;
                        } else {
                            self_length = 1.0 - World::TIME_STEP * Self::SPEED_FALL;
                        }

                        if self_length < 0.0 {
                            player.speed.x = 0.0;
                            player.speed.z = 0.0;
                        } else {
                            player.speed.x *= self_length;
                            player.speed.z *= self_length;
                        }
                    }

                    let self_length = move_speed - (player.speed.dot(move_where));

                    if self_length > 0.0 {
                        player.speed += move_where
                            * self_length.min(Self::SPEED_RISE * move_speed * World::TIME_STEP);
                    }
                } else {
                    // in-air movement.
                    player.speed.y -= Self::SPEED_AIR_FALL * World::TIME_STEP;

                    let speed_length = if move_speed < Self::SPEED_AIR_MIN {
                        move_speed - (player.speed.dot(move_where))
                    } else {
                        Self::SPEED_AIR_MIN - (player.speed.dot(move_where))
                    };

                    if speed_length > 0.0 {
                        player.speed += move_where
                            * speed_length
                                .min(Self::SPEED_AIR_RISE * move_speed * World::TIME_STEP);
                    }
                }
            }
            Self::Slam { ref mut time } => {
                *time += World::TIME_STEP * 4.0;

                player.speed.y -= 8.0_f32.powf(*time + 1.0) * World::TIME_STEP;

                if player.floor {
                    let shake = (player.speed.y.abs() / 64.0).min(1.0) * 0.5;

                    player.shake = shake;
                    player.state = Self::Walk { jump: 0.0 }
                }
            }
        }
    }

    fn view(player: &Player, state: &State, draw: &RaylibHandle) -> View {
        match player.state {
            Self::Walk { jump, .. } => {
                let direction = Direction::new_from_angle(&player.angle);

                let speed = Vector3::new(player.speed.x, 0.0, player.speed.z);
                let scale = (draw.get_time() as f32 * 8.0).sin();
                let tilt = (direction.z.dot(speed) / 4.5) * -2.5;
                let sway = (direction.x.dot(speed) / 4.5) * 0.10;
                let point = if player.speed.y.abs() > 0.0 {
                    Vector3::zero()
                } else {
                    Vector3::up() * scale * sway
                };

                View::new(
                    point
                        + Vector3::new(
                            0.0,
                            Player::CUBE_SHAPE.y - f32::EPSILON + jump.min(0.0),
                            0.0,
                        ),
                    Vector3::new(0.0, jump * 0.1, tilt),
                    state.user.screen_field,
                )
            }
            Self::Slam { .. } => View::new(
                Vector3::new(0.0, Player::CUBE_SHAPE.y - f32::EPSILON, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                state.user.screen_field + 10.0,
            ),
        }
    }
}

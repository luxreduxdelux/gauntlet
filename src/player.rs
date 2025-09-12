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
// bunny hop player movement state: if you slide, jump, then hit the next jump immediately after falling, every subsequent jump will have increased forward velocity with the limitation of less strafe movement
// fix attaching to wall after jump off wall run

use crate::entity::*;
use crate::setting::*;
use crate::state::*;
use crate::utility::*;
use crate::world::*;

//================================================================

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
}

impl Player {
    const ANGLE_MIN: f32 = -90.0;
    const ANGLE_MAX: f32 = 90.0;
    const CUBE_SHAPE: Vector3 = Vector3::new(0.25, 0.5, 0.25);

    fn movement(
        &mut self,
        state: &mut State,
        handle: &mut RaylibHandle,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.shake = (self.shake - World::TIME_STEP * self.shake * 4.0).max(0.0);

        PlayerState::tick(self, state, handle, world);

        //================================================================

        let movement = world
            .physical
            .move_controller(self.collider, self.character, self.speed)?;
        let position = Vector3::new(
            movement.translation.x,
            movement.translation.y,
            movement.translation.z,
        );

        self.clash = false;
        self.slide = movement.is_sliding_down_slope;

        if self.speed * World::TIME_STEP != position {
            self.clash = true;
        }

        self.point += position;
        self.floor = movement.grounded;

        Ok(())
    }
}

#[typetag::serde]
impl Entity for Player {
    fn initialize(
        &mut self,
        state: &mut State,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        self.collider = world.physical.new_cuboid(Self::CUBE_SHAPE);
        world
            .physical
            .set_collider_point(self.collider, self.point)?;

        // this has something to do with being stuck on the ground after a dash jump.
        self.character = KinematicCharacterController::default();
        self.character.snap_to_ground = None;

        self.view = View::new(
            Vector3::up() * 2.0,
            Vector3::default(),
            state.setting.screen_field,
        );

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

    fn draw_3d(
        &mut self,
        state: &mut State,
        draw: &mut RaylibMode3D<'_, RaylibDrawHandle<'_>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        world.physical.draw();

        //================================================================

        state.setting.move_x_a.poll(draw);
        state.setting.move_x_b.poll(draw);
        state.setting.move_z_a.poll(draw);
        state.setting.move_z_b.poll(draw);
        state.setting.jump.poll(draw);
        state.setting.duck.poll(draw);
        state.setting.fire_a.poll(draw);
        state.setting.fire_b.poll(draw);

        //================================================================

        let mouse = draw.get_mouse_delta();

        self.angle.x -= mouse.x * 0.1 * state.setting.mouse_speed;
        self.angle.y += mouse.y * 0.1 * state.setting.mouse_speed;
        self.angle.x %= 359.0;
        self.angle.y = self.angle.y.clamp(Self::ANGLE_MIN, Self::ANGLE_MAX);

        //================================================================

        use rand::Rng;

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

        world.camera_3d.position = point;
        world.camera_3d.target = focus;
        world.camera_3d.up = direction.y;
        world.camera_3d.fovy = self.view.scale;

        Ok(())
    }

    fn draw_2d(
        &mut self,
        _state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        let full = Vector2::new(
            draw.get_render_width() as f32,
            draw.get_render_height() as f32,
        );
        let half = full * 0.5;

        draw.draw_circle_v(half, 8.0, Color::BLACK);
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
        self.movement(state, handle, world)?;

        state.setting.move_x_a.wipe();
        state.setting.move_x_b.wipe();
        state.setting.move_z_a.wipe();
        state.setting.move_z_b.wipe();
        state.setting.jump.wipe();
        state.setting.duck.wipe();
        state.setting.fire_a.wipe();
        state.setting.fire_b.wipe();

        Ok(())
    }
}

//================================================================

enum PlayerState {
    Walk {
        time: f32,
        jump: f32,
        null: f32,
    },
    Dash {
        time: f32,
    },
    Duck {
        time: f32,
    },
    Slam {
        time: f32,
    },
    Wall {
        direction: f32,
        plane: Vector3,
        ray: Vector3,
    },
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Walk {
            time: 0.0,
            jump: 0.0,
            null: 0.0,
        }
    }
}

impl PlayerState {
    const DUCK_SCALE: f32 = 0.30;
    const DUCK_SPEED: f32 = 8.00;
    const SPEED_MIN: f32 = 0.10;
    const SPEED_MAX: f32 = 8.00;
    const SPEED_RISE: f32 = 4.50;
    const SPEED_FALL: f32 = 8.00;
    const SPEED_AIR_MIN: f32 = 0.50;
    const SPEED_AIR_RISE: f32 = 4.00;
    const SPEED_AIR_FALL: f32 = 8.00;
    const SPEED_JUMP: f32 = 3.00;
    const WALL_PLANE_FALL_FORCE: f32 = 2.00;
    const WALL_PLANE_JUMP_FORCE: f32 = 4.00;
    const WALL_PLANE_FALL_TIME: f32 = 0.00;
    const WALL_PLANE_JUMP_TIME: f32 = 0.10;
    const WALL_PLANE_FALL_NULL: f32 = 0.50;
    const WALL_PLANE_JUMP_NULL: f32 = 0.50;

    fn get_movement_key(handle: &RaylibHandle, key_a: Input, key_b: Input) -> f32 {
        if key_a.down(handle) {
            return Self::SPEED_MAX;
        }

        if key_b.down(handle) {
            return -Self::SPEED_MAX;
        }

        0.0
    }

    fn tick(player: &mut Player, state: &State, handle: &RaylibHandle, world: &World) {
        match player.state {
            Self::Walk {
                ref mut time,
                ref mut jump,
                ref mut null,
            } => {
                *time -= World::TIME_STEP;
                *time = time.max(0.0);

                *jump -= *jump * World::TIME_STEP * 4.0;

                *null -= World::TIME_STEP;
                *null = null.max(0.0);

                let move_angle = Direction::new_from_angle(&Vector3::new(player.angle.x, 0.0, 0.0));
                let move_x = move_angle.x
                    * Self::get_movement_key(
                        handle,
                        state.setting.move_x_a,
                        state.setting.move_x_b,
                    );
                let move_z = move_angle.z
                    * Self::get_movement_key(
                        handle,
                        state.setting.move_z_a,
                        state.setting.move_z_b,
                    );
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

                    if state.setting.jump.down(handle) {
                        player.speed.y = Self::SPEED_JUMP;
                        *jump = 0.5;
                    }

                    if state.setting.duck.press() && *time <= 0.0 {
                        if state.setting.move_z_a.down(handle)
                            || state.setting.move_x_b.down(handle)
                            || state.setting.move_z_b.down(handle)
                        {
                            player.speed = move_which * 0.75;
                            player.speed.y = Self::SPEED_JUMP;
                            player.state = PlayerState::Dash { time: 0.40 };
                        } else {
                            Self::to_duck(player);
                        }
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

                    if *null <= 0.0 {
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

                    let ray_list = [
                        Player::CUBE_SHAPE * -1.0,
                        Player::CUBE_SHAPE * 0.00,
                        Player::CUBE_SHAPE * 1.00,
                    ];

                    /*
                    if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
                        for x in 0..3 {
                            let ray = raylib::math::Ray::new(player.point + ray_list[x], move_angle.x);

                            // wall-run.
                            if let Some((_, info)) = state.physical.cast_ray(ray, 0.05, true, QueryFilter::default().exclude_collider(player.collider)) {
                                let plane = Vector3::new(info.normal.x, info.normal.y, info.normal.z);

                                player.speed   = plane * 4.0;
                                player.speed.y = Self::SPEED_JUMP * 1.0;
                                break;
                            }
                        }
                    }
                    */

                    if *time <= 0.0 {
                        for x in 0..3 {
                            let ray = raylib::math::Ray::new(player.point + ray_list[x], move_z);

                            // wall-run.
                            if let Some((_, info)) = world.physical.cast_ray(
                                ray,
                                0.05,
                                true,
                                QueryFilter::default().exclude_collider(player.collider),
                            ) {
                                let plane =
                                    Vector3::new(info.normal.x, info.normal.y, info.normal.z);
                                let mut slide = move_angle.x - plane * (plane.dot(move_angle.x));
                                slide.y = 0.0;

                                player.speed = slide.normalized() * player.speed.length();
                                player.state = Self::Wall {
                                    direction: Self::get_movement_key(
                                        handle,
                                        state.setting.move_z_a,
                                        state.setting.move_z_b,
                                    ),
                                    plane,
                                    ray: move_z,
                                };
                                break;
                            }
                        }
                    }

                    if state.setting.duck.press() {
                        player.speed.x = 0.0;
                        player.speed.y = -8.0;
                        player.speed.z = 0.0;
                        player.state = Self::Slam { time: 0.0 };
                    }
                }
            }
            Self::Dash { ref mut time } => {
                *time -= World::TIME_STEP;

                player.speed.y -= Self::SPEED_AIR_FALL * World::TIME_STEP;

                if *time <= 0.0 || (player.clash && !player.slide) {
                    player.state = Self::Walk {
                        time: 0.5,
                        jump: 0.0,
                        null: 0.0,
                    }
                }
            }
            Self::Duck { ref mut time } => {
                *time -= World::TIME_STEP;

                if *time <= 0.0 || (player.clash && !player.slide) {
                    player.state = Self::Walk {
                        time: 0.35,
                        jump: 0.0,
                        null: 0.0,
                    }
                }

                if state.setting.jump.press() {
                    player.speed.y = Self::SPEED_JUMP;
                    player.state = Self::Walk {
                        time: 0.35,
                        jump: 0.0,
                        null: 0.0,
                    };
                }
            }
            Self::Slam { ref mut time } => {
                *time += World::TIME_STEP * 4.0;

                player.speed.y -= 8.0_f32.powf(*time + 1.0) * World::TIME_STEP;

                if player.floor {
                    let shake = (player.speed.y.abs() / 64.0).min(1.0) * 0.5;

                    player.shake = shake;
                    player.state = Self::Walk {
                        time: 0.0,
                        jump: 0.0,
                        null: 0.0,
                    }
                }
            }
            Self::Wall {
                direction,
                plane,
                ray,
            } => {
                let wall_direction =
                    Self::get_movement_key(handle, state.setting.move_z_a, state.setting.move_z_b);
                let mut wall_none = true;

                let ray_list = [
                    Player::CUBE_SHAPE * -1.0,
                    Player::CUBE_SHAPE * 0.00,
                    Player::CUBE_SHAPE * 1.00,
                ];

                for x in 0..3 {
                    let ray = raylib::math::Ray::new(player.point + ray_list[x], ray);

                    // wall-run.
                    if let Some(_) = world.physical.cast_ray(
                        ray,
                        0.05,
                        true,
                        QueryFilter::default().exclude_collider(player.collider),
                    ) {
                        wall_none = false;
                        break;
                    }
                }

                // wall with an angle is acting kind of weird
                // if player should fall/clash against a wall, make player unable to wall run until they touch the ground again.
                // lock horizontal/vertical angle to limited range.
                // use cast shape instead of ray cast for wall detection.

                if wall_direction != direction || player.clash || wall_none {
                    player.speed += plane * Self::WALL_PLANE_FALL_FORCE;
                    player.state = Self::Walk {
                        time: Self::WALL_PLANE_FALL_TIME,
                        jump: 0.0,
                        null: Self::WALL_PLANE_FALL_NULL,
                    };
                    return;
                }

                if state.setting.jump.press() {
                    player.speed += plane * Self::WALL_PLANE_JUMP_FORCE * 0.5;
                    player.speed.y = Self::SPEED_JUMP * 1.25;
                    player.state = Self::Walk {
                        time: Self::WALL_PLANE_JUMP_TIME,
                        jump: 0.0,
                        null: Self::WALL_PLANE_JUMP_NULL,
                    };
                    return;
                }

                if state.setting.duck.press() {
                    player.speed += plane * Self::WALL_PLANE_JUMP_FORCE * 2.0;
                    player.state = Self::Walk {
                        time: Self::WALL_PLANE_JUMP_TIME,
                        jump: 0.0,
                        null: Self::WALL_PLANE_JUMP_NULL,
                    };
                }
            }
        }
    }

    fn view(player: &Player, state: &State, draw: &RaylibMode3D<'_, RaylibDrawHandle<'_>>) -> View {
        match player.state {
            Self::Walk { jump, .. } => {
                let direction = Direction::new_from_angle(&player.angle);

                let speed = Vector3::new(player.speed.x, 0.0, player.speed.z);
                let scale = (draw.get_time() as f32 * 8.0).sin();
                let tilt = if player.speed.y.abs() > 0.0 {
                    Self::get_movement_key(draw, state.setting.move_z_a, state.setting.move_z_b)
                        * -0.75
                } else {
                    (direction.z.dot(speed) / 4.5) * -2.5
                };
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
                    state.setting.screen_field,
                )
            }
            Self::Dash { .. } => {
                let direction = Direction::new_from_angle(&player.angle);

                let speed = Vector3::new(player.speed.x, 0.0, player.speed.z);
                let x = direction.x.dot(speed).clamp(-1.0, 1.0) * -0.10;
                let z = direction.z.dot(speed).clamp(-1.0, 1.0) * -5.00;

                View::new(
                    Vector3::new(0.0, Player::CUBE_SHAPE.y - f32::EPSILON, 0.0),
                    Vector3::new(0.0, x, z),
                    state.setting.screen_field + 10.0,
                )
            }
            Self::Duck { .. } => View::new(
                Vector3::new(0.0, 0.25 - Player::CUBE_SHAPE.y, 0.0),
                Vector3::new(0.0, 0.1, 0.0),
                state.setting.screen_field + 10.0,
            ),
            Self::Slam { .. } => View::new(
                Vector3::new(0.0, Player::CUBE_SHAPE.y - f32::EPSILON, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                state.setting.screen_field + 10.0,
            ),
            Self::Wall { direction, .. } => {
                let speed = player.speed.length() / 8.0;
                let sin = (draw.get_time() as f32 * 12.0).sin() * 0.1 * speed;
                let cos = (draw.get_time() as f32 * 24.0).cos() * 0.1 * speed;

                View::new(
                    Vector3::new(0.0, Player::CUBE_SHAPE.y - f32::EPSILON + cos, sin),
                    Vector3::new(0.0, 0.0, direction * 2.5),
                    state.setting.screen_field + 10.0,
                )
            }
        }
    }

    fn to_duck(player: &mut Player) {
        let move_speed = Vector3::new(player.speed.x, 0.0, player.speed.z).length();

        if move_speed >= 1.0 {
            let move_angle = Direction::new_from_angle(&Vector3::new(player.angle.x, 0.0, 0.0));
            let move_speed = move_angle.x * Self::DUCK_SPEED;

            player.state = Self::Duck {
                time: Self::DUCK_SCALE,
            };
            player.speed = move_speed + player.speed * move_speed.dot(player.speed).signum()
        }
    }
}

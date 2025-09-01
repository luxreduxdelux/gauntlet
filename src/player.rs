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

use rapier3d::control::KinematicCharacterController;
use rapier3d::prelude::*;
use raylib::prelude::*;

//================================================================

#[derive(Default)]
struct Camera {
    point: Vector3,
    angle: Vector3,
    scale: f32,
}

impl Camera {
    fn new(point: Vector3, angle: Vector3, scale: f32) -> Self {
        Self {
            point,
            angle,
            scale
        }
    }

    fn blend(&mut self, camera: &Camera) {
        self.point += (camera.point - self.point) * State::TIME_STEP * 16.0;
        self.angle += (camera.angle - self.angle) * State::TIME_STEP * 32.0;
        self.scale += (camera.scale - self.scale) * State::TIME_STEP * 16.0;
    }
}

#[derive(Debug, PartialEq)]
enum PlayerState {
    Normal {
        time: f32,
        jump: f32,
        null: f32,
    },
    Duck {
        time: f32,
        fall: f32,
    },
    Wall {
        direction: f32,
        plane: Vector3,
    },
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Normal { time: 0.0, jump: 0.0, null: 0.0 }
    }
}

impl PlayerState {
    const DUCK_SCALE: f32 = 0.25;
    const DUCK_SPEED: f32 = 8.00;
    const SPEED_MIN: f32 = 0.1;
    const SPEED_MAX: f32 = 8.0;
    const SPEED_RISE: f32 = 4.5;
    const SPEED_FALL: f32 = 8.0;
    const SPEED_AIR_MIN: f32 = 0.5;
    const SPEED_AIR_RISE: f32 = 4.0;
    const SPEED_AIR_FALL: f32 = 8.0;
    const SPEED_JUMP: f32 = 3.0;

    fn get_movement_key(handle: &RaylibHandle, key_a: KeyboardKey, key_b: KeyboardKey) -> f32 {
        if handle.is_key_down(key_a) {
            return Self::SPEED_MAX;
        }

        if handle.is_key_down(key_b) {
            return Self::SPEED_MAX * -1.0;
        }

        0.0
    }

    fn tick(player: &mut Player, state: &State, handle: &RaylibHandle) {
        match player.state {
            Self::Normal { ref mut time, ref mut jump, ref mut null } => {
                *time -= State::TIME_STEP;
                *time  = time.max(0.0);

                *jump -= *jump * State::TIME_STEP * 4.0;

                *null -= State::TIME_STEP;
                *null  = null.max(0.0);

                let move_angle = Direction::new_from_angle(&Vector3::new(player.angle.x, 0.0, 0.0));
                let move_x =
                    move_angle.x * Self::get_movement_key(handle, KeyboardKey::KEY_W, KeyboardKey::KEY_S);
                let move_z =
                    move_angle.z * Self::get_movement_key(handle, KeyboardKey::KEY_A, KeyboardKey::KEY_D);
                let move_which = move_x + move_z;
                let move_where = move_which.normalized();
                let move_speed = move_which.length();

                //================================================================

                if player.floor {
                    // on-floor movement.
                    if player.speed.y != 0.0 {
                        if player.speed.y <= -2.0 {
                            *jump = -0.5
                        }

                        player.speed.y = 0.0;
                    }

                    if handle.is_key_down(KeyboardKey::KEY_SPACE) {
                        player.speed.y = Self::SPEED_JUMP;
                        *jump = 0.5;
                    }

                    if handle.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && *time <= 0.0 {
                        Self::to_duck(player);
                    }

                    let self_speed = Vector3::new(player.speed.x, 0.0, player.speed.z);

                    if self_speed.x.abs() >= 0.0 || self_speed.z.abs() >= 0.0 {
                        let mut self_length = self_speed.length();

                        if self_length < Self::SPEED_MIN {
                            self_length =
                                1.0 - State::TIME_STEP * (Self::SPEED_MIN / self_length) * Self::SPEED_FALL;
                        } else {
                            self_length = 1.0 - State::TIME_STEP * Self::SPEED_FALL;
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
                        player.speed +=
                            move_where * self_length.min(Self::SPEED_RISE * move_speed * State::TIME_STEP);
                    }
                } else {
                    // in-air movement.
                    player.speed.y -= Self::SPEED_AIR_FALL * State::TIME_STEP;

                    if *null > 0.0 {
                        return;
                    }

                    let speed_length = if move_speed < Self::SPEED_AIR_MIN {
                        move_speed - (player.speed.dot(move_where))
                    } else {
                        Self::SPEED_AIR_MIN - (player.speed.dot(move_where))
                    };

                    if speed_length > 0.0 {
                        player.speed += move_where
                            * speed_length.min(Self::SPEED_AIR_RISE * move_speed * State::TIME_STEP);
                    }

                    let ray = raylib::math::Ray::new(player.point, move_z);

                    // wall-run.
                    if let Some((collider, info)) = state.physical.cast_ray(ray, 0.1, true, QueryFilter::default().exclude_collider(player.collider)) {
                        let plane = Vector3::new(info.normal.x, info.normal.y, info.normal.z);
                        let speed = Vector3::new(player.speed.x, 0.0, player.speed.z);
                        let slide = speed - plane * (plane.dot(speed));

                        println!("{}", slide.length());

                        player.speed = slide;
                        player.state = Self::Wall { direction: Self::get_movement_key(handle, KeyboardKey::KEY_A, KeyboardKey::KEY_D), plane }
                    }
                }
            },
            Self::Duck { ref mut time, ref mut fall } => {
                *time -= State::TIME_STEP;

                // hitting an obstacle should throw player off duck.

                if !player.floor {
                    if handle.is_key_down(KeyboardKey::KEY_SPACE) {
                        player.speed.y = Self::SPEED_JUMP;
                    }
                }

                if *time <= 0.0 || handle.is_key_down(KeyboardKey::KEY_SPACE) {
                    player.state = Self::Normal { time: 0.5, jump: 0.0, null: 0.0 }
                }
            },
            Self::Wall { direction, plane } => {
                let wall_direction = Self::get_movement_key(handle, KeyboardKey::KEY_A, KeyboardKey::KEY_D);

                // SPACE should bounce player off wall.
                // hitting an obstacle should throw player off wall.

                if wall_direction != direction {
                    player.speed += plane * 2.0;
                    player.state  = Self::Normal { time: 0.0, jump: 0.0, null: 0.5 }
                }

                if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    player.speed   += plane * 4.0;
                    player.speed.y  = Self::SPEED_JUMP;
                    player.state    = Self::Normal { time: 0.0, jump: 0.0, null: 0.5 }
                }
            },
        }
    }

    fn camera(player: &Player, draw: &mut RaylibMode3D<'_, RaylibDrawHandle<'_>>,) -> Camera {
        match player.state {
            Self::Duck { .. } => {
                Camera::new(Vector3::new(0.0, 0.5 - Player::CUBE_SHAPE.y, 0.0), Vector3::new(0.0, 0.1, 0.0), 100.0)
            },
            Self::Normal { time, jump, .. } => {
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

                Camera::new(point + Vector3::new(0.0, Player::CUBE_SHAPE.y - f32::EPSILON, 0.0), Vector3::new(0.0, jump * 0.1, tilt), 90.0)
            },
            Self::Wall { direction, .. } => {
                let speed = player.speed.length() / 8.0;
                let sin = (draw.get_time() as f32 * 12.0).sin() * 0.1 * speed;
                let cos = (draw.get_time() as f32 * 24.0).cos() * 0.1 * speed;

                Camera::new(Vector3::new(0.0, Player::CUBE_SHAPE.y - f32::EPSILON + cos, sin), Vector3::new(0.0, 0.0, direction * 2.5), 110.0)
            },
        }
    }

    fn to_duck(player: &mut Player) {
        let move_speed = Vector3::new(player.speed.x, 0.0, player.speed.z).length();

        if move_speed >= 1.0 {
            let move_angle = Direction::new_from_angle(&Vector3::new(player.angle.x, 0.0, 0.0));

            player.state = Self::Duck { time: Self::DUCK_SCALE, fall: 0.0 };
            player.speed += move_angle.x * Self::DUCK_SPEED;
        }
    }
}

#[derive(Default)]
pub struct Player {
    point: Vector3,
    angle: Vector3,
    speed: Vector3,
    collider: ColliderHandle,
    character: KinematicCharacterController,
    state: PlayerState,
    camera: Camera,
    floor: bool,
}

impl Player {
    const ANGLE_MIN: f32 = -90.0;
    const ANGLE_MAX: f32 = 90.0;
    const CUBE_SHAPE: Vector3 = Vector3::new(0.25, 0.5, 0.25);

    pub fn new(state: &mut State) -> anyhow::Result<Self> {
        state
            .physical
            .new_model(state.asset.get_model("data/level.glb")?)?;

        let collider = state.physical.new_cuboid(Self::CUBE_SHAPE);
        let mollider = state.physical.get_collider_mut(collider).unwrap();
        mollider.set_translation(vector![0.0, 2.0, 0.0]);

        let character = KinematicCharacterController::default();

        Ok(Self {
            point: Vector3::up() * 2.0,
            angle: Vector3::default(),
            speed: Vector3::default(),
            collider,
            character,
            state: PlayerState::default(),
            camera: Camera::default(),
            floor: true,
        })
    }

    fn movement(&mut self, state: &mut State, handle: &mut RaylibHandle) -> anyhow::Result<()> {
        PlayerState::tick(self, state, handle);

        //================================================================

        let movement = state.physical.move_controller(self.collider, self.character, self.speed)?;
        self.point += Vector3::new(movement.translation.x, movement.translation.y, movement.translation.z);
        self.floor  = movement.grounded; 

        Ok(())
    }
}

fn vector_3_rotate_by_axis_angle(value: Vector3, axis: Vector3, mut angle: f32) -> Vector3 {
    // port of raymath's function of the same name.

    let axis = axis.normalized();

    angle /= 2.0;
    let mut a = angle.sin();
    let b = axis.x * a;
    let c = axis.y * a;
    let d = axis.z * a;
    a = angle.cos();
    let w = Vector3::new(b, c, d);

    let mut wv = w.cross(value);

    let mut wwv = w.cross(wv);

    wv.scale(a * 2.0);

    wwv.scale(2.0);

    value + wv + wwv
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
    fn draw_3d(
        &mut self,
        state: &mut State,
        draw: &mut RaylibMode3D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        let model = state.asset.get_model("data/level.glb")?;

        draw.draw_model(model, Vector3::zero(), 1.0, Color::WHITE);
        
        if draw.is_key_down(KeyboardKey::KEY_TAB) {
            state.physical.draw();
        }

        //================================================================

        let mouse = draw.get_mouse_delta();

        self.angle.x -= mouse.x * 0.1;
        self.angle.y += mouse.y * 0.1;
        self.angle.x %= 359.0;
        self.angle.y = self.angle.y.clamp(Self::ANGLE_MIN, Self::ANGLE_MAX);

        //================================================================

        self.camera.blend(&PlayerState::camera(self, draw));

        let direction = Direction::new_from_angle(&self.angle);

        state.camera_3d.position = self.point + self.camera.point;
        state.camera_3d.target   = self.point + self.camera.point + Vector3::new(self.camera.angle.x, self.camera.angle.y, 0.0) + direction.x;
        state.camera_3d.up       = vector_3_rotate_by_axis_angle(direction.y, direction.x, self.camera.angle.z.to_radians());
        state.camera_3d.fovy     = self.camera.scale;

        Ok(())
    }

    fn draw_2d(
        &mut self,
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        draw.draw_circle_v(Vector2::new(1024.0 / 2.0, 768.0 / 2.0), 8.0, Color::BLACK);
        draw.draw_circle_v(Vector2::new(1024.0 / 2.0, 768.0 / 2.0), 4.0, Color::WHITE);

        Ok(())
    }

    fn tick(&mut self, state: &mut State, handle: &mut RaylibHandle) -> anyhow::Result<()> {
        self.movement(state, handle)?;

        Ok(())
    }
}

pub struct Direction {
    pub x: Vector3,
    pub y: Vector3,
    pub z: Vector3,
}

impl Direction {
    pub fn new_from_angle(angle: &Vector3) -> Self {
        // convert to radian.
        let angle = Vector3::new(
            angle.x.to_radians(),
            angle.y.to_radians(),
            angle.z.to_radians(),
        );

        // forward.
        let x = Vector3::new(
            angle.y.cos() * angle.x.sin(),
            angle.y.sin() * -1.0,
            angle.y.cos() * angle.x.cos(),
        );

        // up.
        let y = Vector3::new(
            angle.y.sin() * angle.x.sin(),
            angle.y.cos(),
            angle.y.sin() * angle.x.cos(),
        );

        // right.
        let z = Vector3::new(angle.x.cos(), 0.0, angle.x.sin() * -1.0);

        Self { x, y, z }
    }
}

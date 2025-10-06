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
// after hitting the ground with a slam, if SPACE is hit immediately after,
// - if no movement key is hit, jump up with twice the force
// - otherwise, move forward

use crate::app::*;
use crate::entity::implementation::*;
use crate::helper::*;
use crate::physical::*;
use crate::user::*;
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
    pub point: Vector3,
    pub angle: Vector3,
    #[serde(skip)]
    speed: Vector3,
    #[serde(skip)]
    pub wield: Option<Box<dyn Wield>>,
    #[serde(skip)]
    pub presence: Presence,
    #[serde(skip)]
    state: PlayerState,
    #[serde(skip)]
    view: Target,
    #[serde(skip)]
    floor: bool,
    #[serde(skip)]
    shake: f32,
    #[serde(skip)]
    zoom: f32,
    #[serde(skip)]
    push: f32,
    #[serde(skip)]
    info: EntityInfo,
}

impl Player {
    const ANGLE_MIN: f32 = -90.0;
    const ANGLE_MAX: f32 = 90.00;
    const CUBOID_SCALE: Vector3 = Vector3::new(0.25, 0.5, 0.25);
}

#[typetag::serde]
impl Entity for Player {
    fn get_info(&self) -> &EntityInfo {
        &self.info
    }
    fn get_info_mutable(&mut self) -> &mut EntityInfo {
        &mut self.info
    }

    fn create<'a>(
        &mut self,
        app: &mut App,
        _context: &'a mut Context,
        world: &mut World<'a>,
    ) -> anyhow::Result<()> {
        self.presence = Presence::new_rigid_cuboid_fixed(
            &mut world.scene.physical,
            self.point,
            Vector3::zero(),
            Self::CUBOID_SCALE,
            &self.info,
        )?;

        world.player = Some(self.info.index);

        self.view = Target::new(
            Vector3::up() * 2.0,
            Vector3::default(),
            app.user.video_field,
        );

        Ok(())
    }

    fn draw_3d(
        &mut self,
        app: &mut App,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        if app.user.debug.draw_physical {
            world.scene.physical.draw();
        }

        if !draw.is_cursor_hidden() {
            return Ok(());
        }

        app.user.input_move_x_a.poll(draw);
        app.user.input_move_x_b.poll(draw);
        app.user.input_move_z_a.poll(draw);
        app.user.input_move_z_b.poll(draw);
        app.user.input_jump.poll(draw);
        app.user.input_push.poll(draw);
        app.user.input_pull.poll(draw);

        //================================================================

        let mouse = &draw.get_mouse_delta();

        self.angle.x -= mouse.x * 0.1 * app.user.input_mouse_scale;
        self.angle.y += mouse.y * 0.1 * app.user.input_mouse_scale;
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

        self.view.blend(draw, &PlayerState::view(self, app, draw));

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
        app: &mut App,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        world: &mut World,
    ) -> anyhow::Result<()> {
        if app.user.debug.draw_frame_rate {
            draw.draw_text(&draw.get_fps().to_string(), 8, 8, 32, Color::WHITE);
        }

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

        if let Some(wield) = &mut self.wield {
            wield.draw_2d(app, draw, world)?;
        }

        Ok(())
    }

    fn tick(
        &mut self,
        app: &mut App,
        context: &mut Context,
        world: &mut World,
    ) -> anyhow::Result<()> {
        if !context.handle.is_cursor_hidden() {
            return Ok(());
        }

        // TO-DO fix being able to jump off side of rigid body
        // TO-DO fix snap-to-ground on slope
        PlayerState::tick(self, app, world, &context.handle)?;

        //================================================================

        self.shake = (self.shake - World::TIME_STEP * self.shake * 4.0).max(0.0);
        self.zoom = (self.zoom - World::TIME_STEP * self.zoom * 2.0).max(0.0);

        if let Some(wield) = &mut self.wield {
            wield.tick(app, context, world)?;
        } else {
            let angle = Direction::new_from_angle(&self.angle);

            let cast = world.scene.physical.cast_ray(
                world.scene.camera_3d.position,
                angle.x,
                2.5,
                true,
                Some(self.presence.rigid),
                QueryFilter::default().exclude_sensors(),
            );

            let wrl = { world as *mut World };

            if let Some((collider, _)) = cast
                && let Ok(Some(entity)) = world.entity_from_collider_mutable(collider)
            {
                if app.user.input_pull.get_press() {
                    entity.interact(app, context, unsafe { &mut *wrl }, self)?;
                }
            }

            if app.user.input_push.get_down(&context.handle) {
                self.push = (self.push + World::TIME_STEP * 1.5).min(1.0);
            } else {
                if self.push > 0.0 {
                    let angle = Direction::new_from_angle(&self.angle);

                    let cast = world.scene.physical.cast_ray(
                        world.scene.camera_3d.position,
                        angle.x,
                        2.0,
                        true,
                        Some(self.presence.rigid),
                        QueryFilter::default().exclude_sensors(),
                    );

                    if cast.is_some() {
                        self.shake = self.push * 0.15;

                        // TO-DO past a certain threshold, ignore angle on push jump and just push upward anyway?
                        let boost = angle.x * 5.0 * self.push;

                        if self.speed.y < 0.0 {
                            // allow player to pogo jump off floor if falling down
                            self.speed.x -= boost.x;
                            self.speed.y = -boost.y;
                            self.speed.z -= boost.z;
                        } else if self.angle.y >= 60.0 {
                            self.speed.y += 5.0 * self.push;
                        } else {
                            self.speed -= boost;
                        }
                    } else if !self.floor {
                        if self.angle.y >= 60.0 {
                            self.speed.x = 0.0;
                            self.speed.z = 0.0;
                            self.state = PlayerState::Slam { time: 0.0 };
                        } else {
                            self.zoom = self.push;

                            self.speed += angle.x * 5.0 * self.push;
                        }
                    }
                }

                self.push = 0.0;
            }
        }

        //================================================================

        app.user.input_move_x_a.wipe();
        app.user.input_move_x_b.wipe();
        app.user.input_move_z_a.wipe();
        app.user.input_move_z_b.wipe();
        app.user.input_jump.wipe();
        app.user.input_push.wipe();
        app.user.input_pull.wipe();

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
    const SPEED_AIR_MIN: f32 = 1.0;
    const SPEED_AIR_RISE: f32 = 4.00;
    const SPEED_AIR_FALL: f32 = 8.00;
    const SPEED_JUMP: f32 = 2.75;

    fn get_movement_key(handle: &RaylibHandle, key_a: Input, key_b: Input) -> f32 {
        if key_a.get_down(handle) {
            return Self::SPEED_MAX;
        }

        if key_b.get_down(handle) {
            return -Self::SPEED_MAX;
        }

        0.0
    }

    fn tick(
        player: &mut Player,
        app: &App,
        world: &mut World,
        handle: &RaylibHandle,
    ) -> anyhow::Result<()> {
        match player.state {
            Self::Walk { ref mut jump } => {
                *jump -= *jump * World::TIME_STEP * 4.0;

                let move_angle = Direction::new_from_angle(&Vector3::new(player.angle.x, 0.0, 0.0));
                let move_x = move_angle.x
                    * Self::get_movement_key(
                        handle,
                        app.user.input_move_x_a,
                        app.user.input_move_x_b,
                    );
                let move_z = move_angle.z
                    * Self::get_movement_key(
                        handle,
                        app.user.input_move_z_a,
                        app.user.input_move_z_b,
                    );
                let move_which = move_x + move_z;

                if player.floor {
                    if app.user.input_jump.get_down(handle) {
                        player.speed.y = 2.75;
                        player.floor = false;
                    }
                }

                let controller = KinematicCharacterController::default();

                movement_walk(
                    &mut world.scene.physical,
                    player.presence.collider,
                    controller,
                    move_which,
                    &mut player.point,
                    &mut player.speed,
                    &mut player.floor,
                )
            }
            Self::Slam { ref mut time } => {
                *time += World::TIME_STEP * 4.0;

                player.speed.y -= 8.0_f32.powf(*time + 1.0) * World::TIME_STEP;

                if player.floor {
                    let shake = (player.speed.y.abs() / 64.0).min(1.0) * 0.1;
                    player.shake = shake;
                    player.state = Self::Walk { jump: 0.0 }
                }

                movement_walk(
                    &mut world.scene.physical,
                    player.presence.collider,
                    KinematicCharacterController::default(),
                    player.speed,
                    &mut player.point,
                    &mut player.speed,
                    &mut player.floor,
                )
            }
        }
    }

    fn view(player: &Player, app: &App, draw: &RaylibHandle) -> Target {
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

                Target::new(
                    point
                        + Vector3::new(
                            0.0,
                            Player::CUBOID_SCALE.y - f32::EPSILON + jump.min(0.0),
                            0.0,
                        ),
                    Vector3::new(0.0, jump * 0.1, tilt),
                    app.user.video_field + player.zoom * 25.0,
                )
            }
            Self::Slam { .. } => Target::new(
                Vector3::new(0.0, Player::CUBOID_SCALE.y - f32::EPSILON, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                app.user.video_field + 10.0,
            ),
        }
    }
}

//================================================================

pub trait Wield {
    fn draw_r3d(
        &mut self,
        _app: &mut App,
        _context: &mut Context,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn draw_2d(
        &mut self,
        _app: &mut App,
        _draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn tick(
        &mut self,
        _app: &mut App,
        _context: &mut Context,
        _world: &mut World,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

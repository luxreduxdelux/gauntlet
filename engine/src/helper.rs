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
use crate::asset::*;
use crate::physical::*;
use crate::world::*;

//================================================================

use rapier3d::control::KinematicCharacterController;
use rapier3d::prelude::ColliderHandle;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Serialize, Deserialize, Default)]
pub struct Animation {
    pub name: String,
    pub rate: f32,
    pub frame: f32,
}

impl Animation {
    pub fn new(model: &mut AssetModel, name: &str, rate: f32) -> Self {
        /* TO-DO fix
        model
            .model
            .set_model_animation(model.animation.get_animation(name));
        model.model.set_animation_frame(0);
        */

        Self {
            name: name.to_string(),
            rate,
            frame: 0.0,
        }
    }

    /*
    pub fn update<
        F: FnMut(&mut World, &HashMap<String, serde_json::Value>) -> anyhow::Result<()>,
    >(
        &mut self,
        world: &mut World,
        path: &str,
        call: Option<F>,
    ) -> anyhow::Result<()> {
        let wrl = world as *mut World;
        let model = world.scene.asset.get_model(path)?;

        if let Some(animation) = model.model.get_model_animation() {
            let delta = self.frame + World::TIME_STEP * self.rate;

            if (self.frame as i32) < (delta as i32) {
                let delta = delta as i32 - 1;
                let frame = model.event.map.get(&self.name);
                model.model.set_animation_frame(delta);

                if let Some(frame) = frame
                    && let Some(event) = frame.get(&delta)
                {
                    unsafe {
                        match event {
                            crate::asset::AnimationEvent::Print { data } => println!("{data}"),
                            crate::asset::AnimationEvent::Sound { path } => {
                                (*wrl).scene.asset.get_sound(path)?.sound.play();
                            }
                            crate::asset::AnimationEvent::Custom(data) => {
                                if let Some(mut call) = call {
                                    call(&mut (*wrl), data)?
                                }
                            }
                        }
                    }
                }
            }

            self.frame += World::TIME_STEP * self.rate;
            self.frame %= animation.get_frame_count() as f32 + 1.0;
        }

        Ok(())
    }
    */

    pub fn update(
        &mut self,
        app: &mut App,
        world: &mut World,
        path: &str,
        point: Vector3,
    ) -> anyhow::Result<()> {
        let wrl = world as *mut World;
        let model = world.scene.asset.get_model(path)?;

        // TO-DO fix
        /*
        // TO-DO probably not a good idea to be getting the model animation each frame...fix.
        if let Some(animation) = model.model.get_model_animation() {
            let delta = self.frame + World::TIME_STEP * self.rate;

            if (self.frame as i32) < (delta as i32) {
                let delta = delta as i32 - 1;
                let frame = model.event.map.get(&self.name);
                model.model.set_animation_frame(delta);

                if let Some(frame) = frame
                    && let Some(event) = frame.get(&delta)
                {
                    unsafe {
                        match event {
                            crate::asset::AnimationEvent::Sound { path } => {
                                (*wrl).scene.sound_play(app, path, Some(point))?
                            }
                            crate::asset::AnimationEvent::Custom(_) => {}
                        }
                    }
                }
            }

            self.frame += World::TIME_STEP * self.rate;
            self.frame %= animation.get_frame_count() as f32;
        }
        */

        Ok(())
    }

    pub fn get_bone_data(
        &self,
        model: &AssetModel,
        bone_name: &str,
    ) -> anyhow::Result<Option<(Vector3, Vector4, Vector3)>> {
        /*
        let frame = model
            .animation
            .get_animation(&self.name)
            .ok_or(anyhow::Error::msg(
                "Animation::get_bone_data(): Could not find animation.",
            ))?;

        let mut bone_index = None;

        // find the actual bone index into the pose array.
        for (i, bone) in frame.get_bone_info().iter().enumerate() {
            let name = String::from_utf8(bone.name.iter().map(|&c| c as u8).collect()).unwrap();
            let name = name.trim_matches(char::from(0));
            if name == bone_name {
                bone_index = Some(i);
                break;
            }
        }

        // found bone...
        if let Some(bone_index) = bone_index {
            // get bone data, and return it.
            let frame = frame.get_frame_global_poses(self.frame as usize)[bone_index];

            let (point, angle, scale) = matrix_decompose(&frame);

            return Ok(Some((point, angle, scale)));
        }

        Ok(None)
        */

        todo!()
    }
}

//================================================================

pub struct Direction {
    /// "X", or forward vector.
    pub x: Vector3,
    /// "Y", or up vector.
    pub y: Vector3,
    /// "Z", or right vector.
    pub z: Vector3,
}

impl Direction {
    pub fn draw_debug(
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        point: Vector3,
        angle: Vector3,
    ) {
        let direction = Self::new_from_angle(&angle);

        draw.draw_ray(Ray::new(point, direction.x), Color::RED);
        draw.draw_ray(Ray::new(point, direction.y), Color::GREEN);
        draw.draw_ray(Ray::new(point, direction.z), Color::BLUE);
    }

    pub fn new_from_angle(angle: &Vector3) -> Self {
        // Convert to radian.
        let angle = Vector3::new(
            angle.x.to_radians(),
            angle.y.to_radians(),
            angle.z.to_radians(),
        );

        // Forward.
        let x = Vector3::new(
            angle.y.cos() * angle.x.sin(),
            -angle.y.sin(),
            angle.y.cos() * angle.x.cos(),
        );

        // Up.
        let y = Vector3::new(
            angle.y.sin() * angle.x.sin(),
            angle.y.cos(),
            angle.y.sin() * angle.x.cos(),
        );

        // TO-DO There's probably a better way to do this.
        let y = vector_3_rotate_by_axis_angle(y, x, angle.z);

        // Right.
        let z = Vector3::new(angle.x.cos(), 0.0, -angle.x.sin());

        Self { x, y, z }
    }
}

//================================================================

#[derive(Default)]
pub struct Target {
    pub point: Vector3,
    pub angle: Vector3,
    pub scale: f32,
}

impl Target {
    /// Blending speed for "point".
    const POINT_SPEED: f32 = 16.0;
    /// Blending speed for "angle".
    const ANGLE_SPEED: f32 = 16.0;
    /// Blending speed for "scale".
    const SCALE_SPEED: f32 = 16.0;

    /// Create a new target.
    pub fn new(point: Vector3, angle: Vector3, scale: f32) -> Self {
        Self {
            point,
            angle,
            scale,
        }
    }

    /// Blend this target's data to another target's data.
    pub fn blend(&mut self, handle: &RaylibHandle, target: &Target) {
        let frame = handle.get_frame_time();

        self.point += (target.point - self.point) * frame * Self::POINT_SPEED;
        self.angle += (target.angle - self.angle) * frame * Self::ANGLE_SPEED;
        self.scale += (target.scale - self.scale) * frame * Self::SCALE_SPEED;
    }
}

//================================================================

pub fn movement_walk(
    physical: &mut Physical,
    collider: ColliderHandle,
    character: KinematicCharacterController,
    direction: Vector3,
    point: &mut Vector3,
    speed: &mut Vector3,
    floor: &mut bool,
) -> anyhow::Result<()> {
    const SPEED_MIN: f32 = 0.10;
    const SPEED_MAX: f32 = 8.00;
    const SPEED_RISE: f32 = 4.50;
    const SPEED_FALL: f32 = 8.00;
    const SPEED_AIR_MIN: f32 = 1.0;
    const SPEED_AIR_RISE: f32 = 4.00;
    const SPEED_AIR_FALL: f32 = 8.00;
    const SPEED_JUMP: f32 = 2.75;

    let move_where = direction.normalized();
    let move_speed = direction.length();

    if *floor {
        // on-floor movement.
        if speed.y != 0.0 {
            /*
            // camera fall animation.
            if speed.y <= -2.0 {
                *jump = -0.5
            }
            */

            speed.y = 0.0;
        }

        /*
        if app.user.input_jump.down(handle) {
            speed.y = Self::SPEED_JUMP;
            *jump = 0.5;
        }
        */

        let self_speed = Vector3::new(speed.x, 0.0, speed.z);

        if self_speed.x.abs() >= 0.0 || self_speed.z.abs() >= 0.0 {
            let mut self_length = self_speed.length();

            // TO-DO add edge friction

            if self_length < SPEED_MIN {
                self_length = 1.0 - World::TIME_STEP * (SPEED_MIN / self_length) * SPEED_FALL;
            } else {
                self_length = 1.0 - World::TIME_STEP * SPEED_FALL;
            }

            if self_length < 0.0 {
                speed.x = 0.0;
                speed.z = 0.0;
            } else {
                speed.x *= self_length;
                speed.z *= self_length;
            }
        }

        let self_length = move_speed - (speed.dot(move_where));

        if self_length > 0.0 {
            *speed += move_where * self_length.min(SPEED_RISE * move_speed * World::TIME_STEP);
        }
    } else {
        // in-air movement.
        speed.y -= SPEED_AIR_FALL * World::TIME_STEP;

        let speed_length = if move_speed < SPEED_AIR_MIN {
            move_speed - (speed.dot(move_where))
        } else {
            SPEED_AIR_MIN - (speed.dot(move_where))
        };

        if speed_length > 0.0 {
            *speed += move_where * speed_length.min(SPEED_AIR_RISE * move_speed * World::TIME_STEP);
        }
    }

    let (collision, movement) = physical.move_controller(collider, character, *speed)?;

    *point = Vector3::new(
        point.x + movement.translation.x,
        point.y + movement.translation.y,
        point.z + movement.translation.z,
    );

    // slide off wall.
    if let Some(collision) = collision {
        let normal = Vector3::new(
            collision.hit.normal2.x,
            collision.hit.normal2.y,
            collision.hit.normal2.z,
        );
        *speed -= normal * speed.dot(normal);
    }

    *floor = movement.grounded;

    Ok(())
}

pub fn draw_model_transform(
    draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
    model: &mut AssetModel,
    transform: raylib::math::Matrix,
) {
    model.model.transform = transform.into();

    draw.draw_model(&model.model, Vector3::zero(), 1.0, Color::WHITE);

    model.model.transform = raylib::math::Matrix::identity().into();
}

/// Calculate linear interpolation from "a" to "b" over "time".
pub fn interpolate(a: f32, b: f32, time: f32) -> f32 {
    a + (b - a) * time
}

/// Calculate an ease in-out cubic value.
pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(3.0) / 2.0
    }
}

/// Get the percentage from a range.
pub fn percentage_from_value(input: f32, min: f32, max: f32) -> f32 {
    (input - min) / (max - min)
}

/// Get the value from a range.
pub fn value_from_percentage(input: f32, min: f32, max: f32) -> f32 {
    min + (max - min) * input
}

/// Snap an input value to a grid.
pub fn snap_to_grid(input: f32, grid: f32) -> f32 {
    (input / grid).floor() * grid
}

/// Rotate a Vector3 by an axis angle.
pub fn vector_3_rotate_by_axis_angle(value: Vector3, axis: Vector3, mut angle: f32) -> Vector3 {
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

/// Calculate the sound fall-off over distance and pan relative to a camera.
pub fn calculate_distance_pan(camera: Camera3D, point: Vector3, range: f32) -> (f32, f32) {
    let distance = (point - camera.position).length();
    let distance = (1.0 - (distance / range)).clamp(0.0, 1.0);

    let direction = (point - camera.position).normalized();
    let y = camera
        .up
        .cross(camera.target - camera.position)
        .normalized();
    let pan = (y.dot(direction) + 1.0) / 2.0;

    (distance, pan)
}

/// Throw an error message on screen.
pub fn error_message(text: &str) {
    let e = text.to_string();

    std::thread::spawn(move || {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Fatal Error")
            .set_description(e)
            .show();
    });
}

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
use crate::world::*;

//================================================================

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
        model
            .model
            .set_model_animation(model.animation.get_animation(name));
        model.model.set_animation_frame(0);

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
                                println!("{delta} -> {path}");
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

        Ok(())
    }

    pub fn get_bone_data(
        &self,
        model: &AssetModel,
        bone_name: &str,
    ) -> anyhow::Result<Option<(Vector3, Vector4, Vector3)>> {
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
    }
}

//================================================================

pub struct Direction {
    pub x: Vector3,
    pub y: Vector3,
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
        // convert to radian.
        let angle = Vector3::new(
            angle.x.to_radians(),
            angle.y.to_radians(),
            angle.z.to_radians(),
        );

        // forward.
        let x = Vector3::new(
            angle.y.cos() * angle.x.sin(),
            -angle.y.sin(),
            angle.y.cos() * angle.x.cos(),
        );

        // up.
        let y = Vector3::new(
            angle.y.sin() * angle.x.sin(),
            angle.y.cos(),
            angle.y.sin() * angle.x.cos(),
        );

        // there's probably a better way to do this.
        let y = vector_3_rotate_by_axis_angle(y, x, angle.z);

        // right.
        let z = Vector3::new(angle.x.cos(), 0.0, -angle.x.sin());

        Self { x, y, z }
    }
}

#[derive(Default)]
pub struct View {
    pub point: Vector3,
    pub angle: Vector3,
    pub scale: f32,
}

impl View {
    const BLEND_POINT: f32 = 16.0;
    const BLEND_ANGLE: f32 = 16.0;
    const BLEND_SCALE: f32 = 16.0;

    pub fn new(point: Vector3, angle: Vector3, scale: f32) -> Self {
        Self {
            point,
            angle,
            scale,
        }
    }

    pub fn blend(&mut self, handle: &RaylibHandle, view: &View) {
        let frame = handle.get_frame_time();

        self.point += (view.point - self.point) * frame * Self::BLEND_POINT;
        self.angle += (view.angle - self.angle) * frame * Self::BLEND_ANGLE;
        self.scale += (view.scale - self.scale) * frame * Self::BLEND_SCALE;
    }
}

pub fn interpolate(a: f32, b: f32, time: f32) -> f32 {
    a + (b - a) * time
}

pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(3.0) / 2.0
    }
}

/// Decomposes a transformation matrix into translation, rotation (quaternion), and scale,
/// removing shear (same algorithm and numeric stabilizations as the original C code).
pub fn matrix_decompose(mat: &Matrix) -> (Vector3, Quaternion, Vector3) {
    let eps: f32 = 1e-9;

    // Extract translation
    let translation = Vector3 {
        x: mat.m12,
        y: mat.m13,
        z: mat.m14,
    };

    // Matrix Columns - rotation will be extracted into here.
    // Note: this matches the C code where matColumns[0] = { m0, m4, m8 } etc.
    let mut mat_columns = [
        Vector3 {
            x: mat.m0,
            y: mat.m4,
            z: mat.m8,
        },
        Vector3 {
            x: mat.m1,
            y: mat.m5,
            z: mat.m9,
        },
        Vector3 {
            x: mat.m2,
            y: mat.m6,
            z: mat.m10,
        },
    ];

    // Shear parameters XY, XZ, YZ (extracted and ignored)
    let mut shear = [0.0f32; 3];

    // Normalized scale parameters
    let mut scl = Vector3::default();

    // Max-normalizing helps numerical stability
    let mut stabilizer = eps;
    for i in 0..3 {
        stabilizer = stabilizer.max(mat_columns[i].x.abs());
        stabilizer = stabilizer.max(mat_columns[i].y.abs());
        stabilizer = stabilizer.max(mat_columns[i].z.abs());
    }

    mat_columns[0] = mat_columns[0].scale_by(1.0 / stabilizer);
    mat_columns[1] = mat_columns[1].scale_by(1.0 / stabilizer);
    mat_columns[2] = mat_columns[2].scale_by(1.0 / stabilizer);

    // X scale
    scl.x = mat_columns[0].length();
    if scl.x > eps {
        mat_columns[0] = mat_columns[0].scale_by(1.0 / scl.x);
    }

    // Compute XY shear and make col2 orthogonal
    shear[0] = mat_columns[0].dot(mat_columns[1]);
    mat_columns[1] -= mat_columns[0].scale_by(shear[0]);

    // Y scale
    scl.y = mat_columns[1].length();
    if scl.y > eps {
        mat_columns[1] = mat_columns[1].scale_by(1.0 / scl.y);
        shear[0] /= scl.y; // Correct XY shear
    }

    // Compute XZ and YZ shears and make col3 orthogonal
    shear[1] = mat_columns[0].dot(mat_columns[2]);
    mat_columns[2] -= mat_columns[0].scale_by(shear[1]);
    shear[2] = mat_columns[1].dot(mat_columns[2]);
    mat_columns[2] -= mat_columns[1].scale_by(shear[2]);

    // Z scale
    scl.z = mat_columns[2].length();
    if scl.z > eps {
        mat_columns[2] = mat_columns[2].scale_by(1.0 / scl.z);
        shear[1] /= scl.z; // Correct XZ shear
        shear[2] /= scl.z; // Correct YZ shear
    }

    // Ensure proper handedness (SO(3)) by enforcing determinant = +1
    let cp = mat_columns[1].cross(mat_columns[2]);
    if mat_columns[0].dot(cp) < 0.0 {
        scl = -scl;
        mat_columns[0] = -mat_columns[0];
        mat_columns[1] = -mat_columns[1];
        mat_columns[2] = -mat_columns[2];
    }

    // Set scale (rescale by stabilizer to reverse normalization)
    let scale = scl.scale_by(stabilizer);

    // Build rotation matrix from orthonormal columns (matching C's construction)
    let rotation_matrix = Matrix {
        m0: mat_columns[0].x,
        m1: mat_columns[0].y,
        m2: mat_columns[0].z,
        m3: 0.0,
        m4: mat_columns[1].x,
        m5: mat_columns[1].y,
        m6: mat_columns[1].z,
        m7: 0.0,
        m8: mat_columns[2].x,
        m9: mat_columns[2].y,
        m10: mat_columns[2].z,
        m11: 0.0,
        m12: 0.0,
        m13: 0.0,
        m14: 0.0,
        m15: 1.0,
    };

    let rotation = Vector4::from_matrix(rotation_matrix);

    (translation, rotation, scale)
}

pub fn percentage_from_value(input: f32, min: f32, max: f32) -> f32 {
    (input - min) / (max - min)
}

pub fn value_from_percentage(input: f32, min: f32, max: f32) -> f32 {
    min + (max - min) * input
}

pub fn snap_to_grid(input: f32, grid: f32) -> f32 {
    (input / grid).floor() * grid
}

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

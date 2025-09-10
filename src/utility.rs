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

use raylib::prelude::*;

//================================================================

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

pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(3.0) / 2.0
    }
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

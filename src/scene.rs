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

use crate::asset::*;
use crate::external::r3d::*;
use crate::state::*;
use crate::utility::Direction;

//================================================================

struct Noise {
    point: Option<Vector3>,
    range: f32,
    alias: Option<usize>,
    path: String,
}

// sound/music manager.
// portal visibility manager.
// level geometry manager.
pub struct Scene<'a> {
    pub asset: Asset<'a>,
    pub camera_3d: Camera3D,
    pub camera_2d: Camera2D,
    texture: Option<RenderTexture2D>,
    sound_list: Vec<Noise>,
    music_list: Vec<Noise>,
    pub pause: bool,
}

impl<'a> Scene<'a> {
    pub fn initialize(&mut self, context: &mut Context) -> anyhow::Result<()> {
        self.texture = Some(
            context
                .handle
                .load_render_texture(&context.thread, 1024, 768)?,
        );

        self.camera_3d =
            Camera3D::perspective(Vector3::zero(), Vector3::zero(), Vector3::up(), 90.0);

        Ok(())
    }

    pub fn pause(&mut self) -> anyhow::Result<()> {
        self.pause = true;

        for noise in &self.sound_list {
            let sound = self.asset.get_sound(&noise.path)?;
            sound.sound.pause();

            for alias in &sound.alias {
                alias.pause();
            }
        }

        for noise in &self.music_list {
            let music = self.asset.get_music(&noise.path)?;
            music.pause_stream();
        }

        Ok(())
    }

    pub fn resume(&mut self) -> anyhow::Result<()> {
        self.pause = false;

        for noise in &self.sound_list {
            let sound = self.asset.get_sound(&noise.path)?;
            sound.sound.resume();

            for alias in &sound.alias {
                alias.resume();
            }
        }

        for noise in &self.music_list {
            let music = self.asset.get_music(&noise.path)?;
            music.resume_stream();
        }

        Ok(())
    }

    pub fn play_sound(&mut self, path: &str, point: Option<Vector3>) -> anyhow::Result<()> {
        let sound = self.asset.get_sound(path)?;

        if sound.sound.is_playing() {
            for (i, alias) in sound.alias.iter().enumerate() {
                if !alias.is_playing() {
                    alias.play();

                    self.sound_list.push(Noise {
                        point,
                        range: 16.0,
                        alias: Some(i),
                        path: path.to_string(),
                    });

                    break;
                }
            }
        } else {
            sound.sound.play();

            self.sound_list.push(Noise {
                point,
                range: 16.0,
                alias: None,
                path: path.to_string(),
            });
        }

        Ok(())
    }

    pub fn play_music(&mut self, path: &str, point: Option<Vector3>) -> anyhow::Result<()> {
        let music = self.asset.get_music(path)?;
        music.play_stream();

        self.music_list.push(Noise {
            point,
            range: 16.0,
            alias: None,
            path: path.to_string(),
        });

        Ok(())
    }

    fn calculate_distance_pan(&self, point: Vector3, range: f32) -> (f32, f32) {
        let distance = (point - self.camera_3d.position).length();
        let distance = (1.0 - (distance / range)).clamp(0.0, 1.0);

        let direction = (point - self.camera_3d.position).normalized();
        let y = self
            .camera_3d
            .up
            .cross(self.camera_3d.target - self.camera_3d.position)
            .normalized();
        let pan = (y.dot(direction) + 1.0) / 2.0;

        (distance, pan)
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        if self.pause {
            return Ok(());
        }

        for noise in &self.sound_list {
            if let Some(point) = noise.point {
                let (distance, pan) = self.calculate_distance_pan(point, noise.range);

                let sound = self.asset.get_sound(&noise.path)?;

                if let Some(alias) = noise.alias {
                    let alias = sound.alias.get(alias).unwrap();
                    alias.set_volume(distance);
                    alias.set_pan(pan);
                } else {
                    sound.sound.set_volume(distance);
                    sound.sound.set_pan(pan);
                }
            }
        }

        for noise in &self.music_list {
            let music = self.asset.get_music(&noise.path)?;
            music.update_stream();

            if let Some(point) = noise.point {
                let (distance, pan) = self.calculate_distance_pan(point, noise.range);

                music.set_volume(distance);
                music.set_pan(pan);
            }
        }

        self.sound_list.retain(|noise| {
            let sound = self.asset.get_sound(&noise.path).unwrap();

            if let Some(alias) = noise.alias {
                let alias = sound.alias.get(alias).unwrap();
                alias.is_playing()
            } else {
                sound.sound.is_playing()
            }
        });

        self.music_list.retain(|noise| {
            let music = self.asset.get_music(&noise.path).unwrap();
            music.is_stream_playing()
        });

        Ok(())
    }

    pub fn draw_r3d<F: FnMut(&mut Handle) -> anyhow::Result<()>>(
        &mut self,
        context: &mut Context,
        mut call: F,
    ) -> anyhow::Result<()> {
        if context.handle.is_window_resized() {
            let size = Vector2::new(
                context.handle.get_screen_width() as f32,
                context.handle.get_screen_height() as f32,
            );

            context
                .r3d
                .update_resolution((size.x as i32, size.y as i32));
            self.texture = Some(context.handle.load_render_texture(
                &context.thread,
                size.x as u32,
                size.y as u32,
            )?);
        }

        let texture = self.texture.as_mut().unwrap();
        let mut result = Ok(());

        context.r3d.render_ex(self.camera_3d, texture, |r3d| {
            // scene should be in charge of level geometry rendering...?
            result = call(r3d);
        });

        result
    }

    pub fn draw_3d<
        F: FnMut(
            &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        ) -> anyhow::Result<()>,
    >(
        &mut self,
        context: &mut Context,
        draw: &mut RaylibDrawHandle,
        mut call: F,
    ) -> anyhow::Result<()> {
        let texture = self.texture.as_mut().unwrap();
        let mut draw = draw.begin_texture_mode(&context.thread, texture);
        let mut draw = draw.begin_mode3D(self.camera_3d);

        call(&mut draw)
    }

    pub fn draw_2d<F: FnMut(&mut RaylibMode2D<'_, RaylibDrawHandle<'_>>) -> anyhow::Result<()>>(
        &mut self,
        context: &mut Context,
        draw: &mut RaylibDrawHandle,
        mut call: F,
    ) -> anyhow::Result<()> {
        let texture = self.texture.as_mut().unwrap();
        let mut draw = draw.begin_mode2D(self.camera_2d);

        //let shd = world.scene.asset.get_shader("screen")?;
        //let mut shd = draw_2d.begin_shader_mode(shd);

        draw.draw_texture_pro(
            &texture,
            Rectangle::new(
                0.0,
                0.0,
                texture.texture.width as f32,
                -texture.texture.height as f32,
            ),
            Rectangle::new(
                0.0,
                0.0,
                context.handle.get_screen_width() as f32,
                context.handle.get_screen_height() as f32,
            ),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

        call(&mut draw)
    }
}

impl<'a> Default for Scene<'a> {
    fn default() -> Self {
        Self {
            asset: Asset::default(),
            camera_3d: Camera3D::perspective(
                Vector3::zero(),
                Vector3::zero(),
                Vector3::zero(),
                f32::default(),
            ),
            camera_2d: Camera2D {
                offset: Vector2::zero(),
                target: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            },
            texture: None,
            sound_list: Vec::default(),
            music_list: Vec::default(),
            pause: bool::default(),
        }
    }
}

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

use hashbrown::HashMap;
use rapier3d::prelude::*;
use raylib::prelude::*;

//================================================================

use crate::app::*;
use crate::asset::*;
use crate::physical::*;
use crate::utility::*;

//================================================================

// TO-DO: been told by raylib contributor to use another audio solution for
// spacialization and better audio management, research another library.
pub struct Scene<'a> {
    pub asset: Asset<'a>,
    pub camera_3d: Camera3D,
    pub camera_2d: Camera2D,
    texture: Option<RenderTexture2D>,
    sound_list: Vec<Noise>,
    music_list: Vec<Noise>,
    //pub light_list: Vec<Light>,
    pub room_list: Vec<Room>,
    pub view_list: Vec<View>,
    pub path_list: Vec<Path>,
    pub path_hash: HashMap<(i32, i32, i32), Vec<usize>>,
    pub physical: Physical,
    pub room_rigid: Option<RigidBodyHandle>,
    pub pause: bool,
}

impl<'a> Scene<'a> {
    pub fn path_add(&mut self, point: Vector3) {
        self.path_list.push(Path { point });
        let point = (
            snap_to_grid(point.x, 16.0) as i32,
            snap_to_grid(point.y, 16.0) as i32,
            snap_to_grid(point.z, 16.0) as i32,
        );
        let entry = self.path_hash.entry(point).or_default();
        entry.push(self.path_list.len() - 1);
    }

    fn path_closest(&self, point: Vector3, path_list: &'a [Path]) -> Option<&'a Path> {
        let mut path_away = f32::MAX;
        let mut path_pick = None;

        for path in path_list {
            let away = point.distance_to(path.point);

            if away <= path_away {
                path_away = away;
                path_pick = Some(path)
            }
        }

        path_pick
    }

    //pub fn path_calculate(&mut self, point_a: Vector3, point_b: Vector3) -> Vec<Path> {
    //    let path_o = Vec::with_capacity(self.path_list.len());
    //    let path_c = Vec::with_capacity(self.path_list.len());
    //}

    pub fn room_active(&self, point: Vector3) -> bool {
        if let Some((_, collider)) = self.physical.intersect_point(
            point,
            None,
            QueryFilter::default()
                .exclude_solids()
                .groups(Physical::GROUP_GEOMETRY),
        ) {
            self.room_list[collider.user_data as usize].visible
        } else {
            false
        }
    }

    pub fn room_active_box(&self, point: Vector3, angle: Vector3, shape: Vector3) -> bool {
        if let Some((_, collider)) = self.physical.intersect_cuboid(
            point,
            angle,
            shape,
            None,
            QueryFilter::default()
                .exclude_solids()
                .groups(Physical::GROUP_GEOMETRY),
        ) {
            self.room_list[collider.user_data as usize].visible
        } else {
            false
        }
    }

    pub fn room_active_index(&self, point: Vector3) -> Option<usize> {
        if let Some((_, collider)) = self.physical.intersect_point(
            point,
            None,
            QueryFilter::default()
                .exclude_solids()
                .groups(Physical::GROUP_GEOMETRY),
        ) {
            Some(collider.user_data as usize)
        } else {
            None
        }
    }

    pub fn update_resolution(&mut self, context: &mut Context, scale: f32) -> anyhow::Result<()> {
        let size = Vector2::new(
            context.handle.get_screen_width() as f32 * scale,
            context.handle.get_screen_height() as f32 * scale,
        );

        self.texture = Some(context.handle.load_render_texture(
            &context.thread,
            size.x as u32,
            size.y as u32,
        )?);

        Ok(())
    }

    pub fn initialize(&mut self, app: &App, context: &mut Context) -> anyhow::Result<()> {
        self.texture = Some(context.handle.load_render_texture(
            &context.thread,
            (context.handle.get_screen_width() as f32 * app.user.video_scale) as u32,
            (context.handle.get_screen_height() as f32 * app.user.video_scale) as u32,
        )?);

        self.asset.set_shader(
            context,
            "screen",
            Some("data/shader/base.vs"),
            Some("data/shader/screen.fs"),
        )?;

        self.camera_3d =
            Camera3D::perspective(Vector3::zero(), Vector3::zero(), Vector3::up(), 90.0);

        for (i_v, view) in self.view_list.iter_mut().enumerate() {
            for (i_r, room) in self.room_list.iter_mut().enumerate() {
                let direction = Direction::new_from_angle(&view.angle);
                let direction_f = raylib::math::Ray::new(view.point, direction.x);
                let direction_b = raylib::math::Ray::new(view.point, direction.x * -1.0);

                let model = self.asset.get_model(&room.path)?;
                let bound = model.model.get_model_bounding_box();

                let hit_f = bound.get_ray_collision_box(direction_f);
                let hit_b = bound.get_ray_collision_box(direction_b);

                let hit_f = hit_f.hit && hit_f.distance <= 1.0;
                let hit_b = hit_b.hit && hit_b.distance <= 1.0;

                if hit_f || hit_b {
                    room.view.push(i_v);
                    view.room.push(i_r);
                }
            }
        }

        Ok(())
    }

    pub fn set_pause(&mut self, pause: bool) -> anyhow::Result<()> {
        self.pause = pause;

        for noise in &self.sound_list {
            let sound = self.asset.get_sound(&noise.path)?;
            if pause {
                sound.sound.pause();
            } else {
                sound.sound.resume();
            }

            for alias in &sound.alias {
                if pause {
                    alias.pause();
                } else {
                    alias.resume();
                }
            }
        }

        for noise in &self.music_list {
            let music = self.asset.get_music(&noise.path)?;
            if pause {
                music.pause_stream();
            } else {
                music.resume_stream();
            }
        }

        Ok(())
    }

    pub fn room_add(&mut self, context: &mut Context, path: &str) -> anyhow::Result<()> {
        if self.room_rigid.is_none() {
            self.room_rigid = Some(self.physical.new_rigid_fixed());
        }

        let model = self.asset.set_model(context, path)?;
        let bound = model.model.get_model_bounding_box();

        let collider = self
            .physical
            .new_cuboid((bound.max - bound.min) * 0.5, self.room_rigid);
        self.physical
            .set_collider_point(collider, (bound.min + bound.max) * 0.5)?;
        self.physical.set_collider_sensor(collider, true)?;
        self.physical
            .set_collider_data(collider, self.room_list.len() as u128)?;
        self.physical
            .set_collider_group(collider, Physical::GROUP_GEOMETRY)?;

        self.physical.new_model(&model.model, self.room_rigid)?;

        self.room_list.push(Room {
            point: (bound.min + bound.max) * 0.5,
            angle: Vector3::zero(),
            scale: (bound.max - bound.min) * 0.5,
            bound,
            path: path.to_string(),
            view: Vec::default(),
            visible: false,
            visit: false,
        });

        Ok(())
    }

    pub fn view_add(&mut self, point: Vector3, angle: Vector3) -> anyhow::Result<usize> {
        let mut view = View::default();
        view.point = point;
        view.angle = angle;
        view.visible = false;

        let index = self.view_list.len();

        self.view_list.push(view);

        Ok(index)
    }

    pub fn sound_play(
        &mut self,
        app: &App,
        path: &str,
        point: Option<Vector3>,
    ) -> anyhow::Result<()> {
        let sound = self.asset.get_sound(path)?;

        if sound.sound.is_playing() {
            for (i, alias) in sound.alias.iter().enumerate() {
                if !alias.is_playing() {
                    if let Some(point) = point {
                        let (distance, pan) = calculate_distance_pan(self.camera_3d, point, 8.0);
                        alias.set_volume(distance * app.user.audio_sound);
                        alias.set_pan(pan);
                    } else {
                        alias.set_volume(app.user.audio_sound);
                    }

                    alias.play();

                    self.sound_list.push(Noise {
                        point,
                        range: 8.0,
                        alias: Some(i),
                        path: path.to_string(),
                    });

                    return Ok(());
                }
            }
        }

        // TO-DO cull sound if not even audible?
        if let Some(point) = point {
            let (distance, pan) = calculate_distance_pan(self.camera_3d, point, 8.0);
            sound.sound.set_volume(distance * app.user.audio_sound);
            sound.sound.set_pan(pan);
        } else {
            sound.sound.set_volume(app.user.audio_sound);
        }

        sound.sound.play();

        self.sound_list.push(Noise {
            point,
            range: 8.0,
            alias: None,
            path: path.to_string(),
        });

        Ok(())
    }

    pub fn music_play(&mut self, path: &str, point: Option<Vector3>) -> anyhow::Result<()> {
        let music = self.asset.get_music(path)?;
        music.play_stream();

        self.music_list.push(Noise {
            point,
            range: 8.0,
            alias: None,
            path: path.to_string(),
        });

        Ok(())
    }

    pub fn update(&mut self, app: &App, context: &mut Context) -> anyhow::Result<()> {
        if context.handle.is_window_resized() {
            self.update_resolution(context, app.user.video_scale)?;
        }

        if self.pause {
            return Ok(());
        }

        for noise in &self.sound_list {
            let sound = self.asset.get_sound(&noise.path)?;
            if let Some(point) = noise.point {
                let (distance, pan) = calculate_distance_pan(self.camera_3d, point, noise.range);

                if let Some(alias) = noise.alias {
                    let alias = sound.alias.get(alias).unwrap();
                    alias.set_volume(distance * app.user.audio_sound);
                    alias.set_pan(pan);
                } else {
                    sound.sound.set_volume(distance * app.user.audio_sound);
                    sound.sound.set_pan(pan);
                }
            } else {
                // TO-DO check with raylib if setting the volume each frame is bad or not?
                sound.sound.set_volume(app.user.audio_sound);
            }
        }

        for noise in &self.music_list {
            let music = self.asset.get_music(&noise.path)?;
            music.update_stream();

            if let Some(point) = noise.point {
                let (distance, pan) = calculate_distance_pan(self.camera_3d, point, noise.range);

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
        let scn = { self as *mut Self };
        let texture = self.texture.as_mut().unwrap();
        let mut draw = draw.begin_texture_mode(&context.thread, texture);
        let mut draw = draw.begin_mode3D(self.camera_3d);

        draw.clear_background(Color::BLACK);

        if draw.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            self.physical.draw();
        }

        for room in &mut self.room_list {
            room.visit = false;
            room.visible = false;
        }

        unsafe {
            // HACK: menu view isn't working, i assume it's we don't have any view node?
            if self.view_list.is_empty() {
                for room in &self.room_list {
                    let model = self.asset.get_model(&room.path).unwrap();
                    draw.draw_model(&model.model, Vector3::zero(), 1.0, Color::WHITE);
                }
            } else if let Some(room) = (*scn).room_active_index(self.camera_3d.position) {
                Room::traverse(
                    room,
                    &mut draw,
                    &self.view_list,
                    &mut self.room_list,
                    &mut self.asset,
                    true,
                );
            }
        }

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

        draw.draw_text(&draw.get_fps().to_string(), 8, 8, 32, Color::GREEN);

        call(&mut draw)
    }
}

impl<'a> Default for Scene<'a> {
    fn default() -> Self {
        Self {
            asset: Asset::default(),
            camera_3d: Camera3D::perspective(
                Vector3::default(),
                Vector3::default(),
                Vector3::default(),
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
            //light_list: Vec::default(),
            room_list: Vec::default(),
            view_list: Vec::default(),
            path_list: Vec::default(),
            path_hash: HashMap::default(),
            room_rigid: None,
            physical: Physical::default(),
            pause: bool::default(),
        }
    }
}

//================================================================

#[derive(Default, Debug, Clone)]
pub struct Room {
    pub point: Vector3,
    pub angle: Vector3,
    pub scale: Vector3,
    pub bound: BoundingBox,
    pub path: String,
    pub view: Vec<usize>,
    pub visible: bool,
    pub visit: bool,
}

impl<'a> Room {
    fn traverse(
        room_index: usize,
        draw: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibDrawHandle<'_>>>,
        view_list: &[View],
        room_list: &mut [Room],
        asset: &mut Asset<'a>,
        inside: bool,
    ) {
        let current_room = &mut room_list[room_index];

        if current_room.visit {
            return;
        }

        current_room.visit = true;

        if current_room.is_visible(view_list) || inside {
            current_room.visible = true;

            let model = asset.get_model(&current_room.path).unwrap();
            draw.draw_model(&model.model, Vector3::zero(), 1.0, Color::WHITE);

            let c_r_view = current_room.view.clone();

            for view in &c_r_view {
                for room in &view_list[*view].room {
                    Self::traverse(*room, draw, view_list, room_list, asset, false);
                }
            }
        } else {
            current_room.visible = false;
        }
    }

    fn is_visible(&self, view_list: &[View]) -> bool {
        if self.view.is_empty() {
            return true;
        }

        for view in &self.view {
            if view_list[*view].visible {
                return true;
            }
        }

        false
    }
}

//================================================================

#[derive(Default, Debug, Clone)]
pub struct View {
    pub point: Vector3,
    pub angle: Vector3,
    pub visible: bool,
    pub room: Vec<usize>,
}

//================================================================

struct Noise {
    point: Option<Vector3>,
    range: f32,
    alias: Option<usize>,
    path: String,
}

pub struct Path {
    point: Vector3,
}

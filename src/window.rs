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

// 3D render should have fog, potentially lighting (black light upon grey glass or something)
// add trail effect from far out into cube
// cube should also animate, rotating like a rubik's cube

use crate::asset::*;
use crate::scene::*;
use crate::state::*;
use crate::user::*;
use crate::utility::*;

//================================================================

use raylib::prelude::*;
use std::collections::HashMap;
use std::f32;

//================================================================

pub struct Response {
    pub hover: bool,
    pub press: bool,
    pub release: bool,
    pub widget: Widget,
}

#[derive(Default, Clone, Copy)]
pub struct Widget {
    pub delta: f32,
    pub hover: bool,
}

impl Response {
    fn new(hover: bool, press: bool, release: bool, widget: Widget) -> Self {
        Self {
            hover,
            press,
            release,
            widget,
        }
    }
}

#[derive(Default)]
pub struct Window<'a> {
    widget: HashMap<String, Widget>,
    scene: Scene<'a>,
    point: Vector2,
    mouse: Vector2,
    focus: Option<String>,
    time: f32,
}

impl<'a> Window<'a> {
    const BUTTON_SHAPE_Y: f32 = 32.0;
    const FONT_SPACE: f32 = 1.0;

    pub fn initialize(&mut self, context: &'a mut Context) -> anyhow::Result<()> {
        self.scene
            .asset
            .set_font(context, "data/video/font_label.ttf", 32)?;
        self.scene
            .asset
            .set_font(context, "data/video/font_title.ttf", 56)?;
        self.scene
            .asset
            .set_sound(context, "data/audio/hover.ogg", 0)?;
        self.scene
            .asset
            .set_sound(context, "data/audio/click.ogg", 0)?;
        self.scene
            .asset
            .set_sound(context, "data/audio/back.ogg", 0)?;

        Ok(())
    }

    fn begin(&mut self, handle: &RaylibHandle) {
        self.point = Vector2::new(8.0, 8.0);

        if self.mouse == Vector2::zero() {
            let delta = handle.get_mouse_delta();

            if delta != Vector2::zero() {
                self.mouse = handle.get_mouse_position();
            }
        } else {
            self.mouse = handle.get_mouse_position();
        }
    }
    fn close(&mut self) {}

    fn font_label(&self) -> anyhow::Result<&Font> {
        self.scene.asset.get_font("data/video/font_label.ttf")
    }

    fn font_title(&self) -> anyhow::Result<&Font> {
        self.scene.asset.get_font("data/video/font_title.ttf")
    }

    fn response(
        &mut self,
        handle: &RaylibHandle,
        text: &str,
        shape: Rectangle,
    ) -> anyhow::Result<Response> {
        let focus = if let Some(focus) = &self.focus
            && focus == text
        {
            true
        } else {
            false
        };
        let hover = shape.check_collision_point_rec(self.mouse) || focus;
        let press = handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let release = handle.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);

        let entry = self.widget.entry(text.to_string()).or_default();
        let frame = handle.get_frame_time();

        if self.focus.is_some() && !focus {
            return Ok(Response::new(false, false, false, *entry));
        }

        if hover {
            entry.delta += frame * 8.0;

            if !entry.hover {
                self.scene
                    .asset
                    .get_sound("data/audio/hover.ogg")?
                    .sound
                    .play();

                entry.hover = true;
            }
        } else {
            entry.delta -= frame * 4.0;
            entry.hover = false;
        }

        entry.delta = entry.delta.clamp(0.0, 1.0);

        Ok(Response::new(
            hover,
            hover && press,
            hover && release,
            *entry,
        ))
    }

    pub fn draw<
        T: FnMut(&mut Self, &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>) -> anyhow::Result<()>,
    >(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        mut call: T,
    ) -> anyhow::Result<()> {
        self.begin(draw);

        call(self, draw)?;

        self.close();

        Ok(())
    }

    fn draw_border(
        &self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        shape: Rectangle,
        label: &str,
        delta: f32,
    ) -> anyhow::Result<()> {
        let white = Color::WHITE.lerp(Color::BLACK, delta);
        let black = Color::BLACK.lerp(Color::WHITE, delta);

        draw.draw_rectangle_rec(shape, black);

        let shape = Rectangle::new(
            shape.x + 3.0,
            shape.y + 3.0,
            shape.width - 6.0,
            shape.height - 6.0,
        );

        draw.draw_rectangle_rec(shape, white);

        if !label.is_empty() {
            draw.draw_text_ex(
                self.font_label()?,
                label,
                Vector2::new(shape.x + 4.0, shape.y - 6.0),
                Self::BUTTON_SHAPE_Y,
                Self::FONT_SPACE,
                black,
            );
        }

        Ok(())
    }

    fn draw_border_text(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        shape: Rectangle,
    ) -> anyhow::Result<Response> {
        let font = self.font_label()?;
        let size = font.measure_text(text, Self::BUTTON_SHAPE_Y, Self::FONT_SPACE);
        let size = Rectangle::new(shape.x, shape.y, shape.width + size.x, shape.height);
        let response = self.response(draw, text, size)?;
        let delta = ease_in_out_cubic(response.widget.delta);

        self.draw_border(draw, size, text, delta)?;

        Ok(response)
    }

    pub fn button(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
    ) -> anyhow::Result<Response> {
        let response = self.draw_border_text(
            draw,
            text,
            Rectangle::new(self.point.x, self.point.y, 16.0, Self::BUTTON_SHAPE_Y),
        )?;

        if response.press {
            self.scene
                .asset
                .get_sound("data/audio/click.ogg")?
                .sound
                .play();
        }

        self.point.y += Self::BUTTON_SHAPE_Y + 8.0;

        Ok(response)
    }

    fn toggle(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut bool,
    ) -> anyhow::Result<Response> {
        self.draw_border(
            draw,
            Rectangle::new(
                self.point.x,
                self.point.y,
                Self::BUTTON_SHAPE_Y,
                Self::BUTTON_SHAPE_Y,
            ),
            "",
            if *value { 1.0 } else { 0.0 },
        )?;

        let response = self.draw_border_text(
            draw,
            text,
            Rectangle::new(
                self.point.x + Self::BUTTON_SHAPE_Y + 8.0,
                self.point.y,
                16.0,
                Self::BUTTON_SHAPE_Y,
            ),
        )?;

        if response.press {
            self.scene
                .asset
                .get_sound("data/audio/click.ogg")?
                .sound
                .play();
            *value = !*value;
        }

        self.point.y += Self::BUTTON_SHAPE_Y + 8.0;

        Ok(response)
    }

    pub fn slider(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut f32,
        bound: (f32, f32, f32),
    ) -> anyhow::Result<Response> {
        let key = format!("{:.2}", value);
        let font = self.font_label()?;
        let size = font.measure_text(&key, Self::BUTTON_SHAPE_Y, Self::FONT_SPACE);

        self.draw_border(
            draw,
            Rectangle::new(
                self.point.x,
                self.point.y,
                size.x + 16.0,
                Self::BUTTON_SHAPE_Y,
            ),
            &key,
            1.0,
        )?;

        let response = self.draw_border_text(
            draw,
            text,
            Rectangle::new(
                self.point.x + size.x + 24.0,
                self.point.y,
                16.0,
                Self::BUTTON_SHAPE_Y,
            ),
        )?;

        if response.press {
            self.scene
                .asset
                .get_sound("data/audio/click.ogg")?
                .sound
                .play();
            self.focus = Some(text.to_string());
        }

        if let Some(focus) = &self.focus
            && focus == text
        {
            let delta = draw.get_mouse_delta().x;

            // fix for numerical instability.
            if delta.abs() > 0.0 {
                let min = self.point.x + size.x + 24.0;
                let max = min + 128.0;
                let percent = ((self.mouse.x - min) / (max - min)).clamp(0.0, 1.0);
                let percent = percent * (bound.1 - bound.0) + bound.0;
                let percent = (percent / bound.2).floor() * bound.2;

                *value = percent;
            }

            if draw.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                self.focus = None;
            }
        }

        self.point.y += Self::BUTTON_SHAPE_Y + 8.0;

        Ok(response)
    }

    fn action(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut Input,
    ) -> anyhow::Result<Response> {
        let key = format!("{}", value);

        let font = self.font_label()?;
        let size = font.measure_text(&key, Self::BUTTON_SHAPE_Y, Self::FONT_SPACE);
        let size = (size.x / 7.0).ceil() * 7.0;

        self.draw_border(
            draw,
            Rectangle::new(
                self.point.x,
                self.point.y,
                size + 16.0,
                Self::BUTTON_SHAPE_Y,
            ),
            &key,
            1.0,
        )?;

        let response = self.draw_border_text(
            draw,
            text,
            Rectangle::new(
                self.point.x + size + 24.0,
                self.point.y,
                16.0,
                Self::BUTTON_SHAPE_Y,
            ),
        )?;

        if let Some(focus) = &self.focus
            && focus == text
        {
            if let Some(board) = draw.get_key_pressed() {
                *value = Input::new_board(board as u32);
                self.focus = None;
            }
            if let Some(mouse) = Input::get_mouse_pressed(draw) {
                *value = Input::new_mouse(mouse as u32);
                self.focus = None;
            }
        }

        if response.press {
            self.scene
                .asset
                .get_sound("data/audio/click.ogg")?
                .sound
                .play();
            self.focus = Some(text.to_string());
        }

        self.point.y += Self::BUTTON_SHAPE_Y + 8.0;

        Ok(response)
    }
}

pub enum Layout {
    Intro,
    Main,
    Zoom,
    Begin,
    Setup,
    Close,
}

impl Layout {
    const INITIAL_POINT: Vector2 = Vector2::new(12.0, 84.0);

    fn change_layout(state: &mut State, layout: Option<Self>) {
        state.layout = layout;
        state.window.time = 0.0;
        state.window.widget.clear();
        state.window.mouse = Vector2::zero();
    }

    pub fn draw(
        state: &mut State,
        draw: &mut RaylibDrawHandle<'_>,
        context: &mut Context,
    ) -> anyhow::Result<()> {
        state.window.time += draw.get_frame_time();

        // right-click should return to the last menu.
        // improve slider widget.
        // add scroll widget?

        if draw.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            if let Some(world) = &mut state.world {
                if world.scene.pause {
                    world.scene.resume()?;
                } else {
                    world.scene.pause()?;
                }
            }
        }

        if let Some(layout) = &mut state.layout {
            match layout {
                Layout::Main => Self::main(state, draw),
                Layout::Zoom => Self::zoom(state, context, draw),
                Layout::Begin => Self::begin(state, context, draw),
                Layout::Setup => Self::setup(state, draw),
                Layout::Close => Self::close(state, draw),
                _ => Ok(()),
            }?;
        } else {
            if draw.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                Self::change_layout(state, Some(Layout::Main));
                draw.enable_cursor();
            }
        }

        Ok(())
    }

    fn window_time_scale(window: &Window) -> f32 {
        (window.time * 2.5).min(1.0)
    }

    fn main(state: &mut State, draw: &mut RaylibDrawHandle<'_>) -> anyhow::Result<()> {
        if state.world.is_some() {
            Self::layout_back(state, draw, None)?;
        }

        Self::draw_back(draw, state.world.is_some(), 1.0);

        let mut draw = draw.begin_mode2D(Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        });

        let mut layout = None;

        state.window.draw(&mut draw, |window, draw| {
            Self::draw_head_foot(
                window,
                draw,
                state.world.is_some(),
                "pwrmttl",
                Self::window_time_scale(window),
            )?;

            window.point = Self::INITIAL_POINT;

            if window.button(draw, "begin")?.press {
                layout = Some(Self::Begin);
            };
            if window.button(draw, "setup")?.press {
                layout = Some(Self::Setup);
            };
            if window.button(draw, "close")?.press {
                layout = Some(Self::Close);
            };

            Ok(())
        })?;

        if let Some(layout) = layout {
            Self::change_layout(state, Some(layout));
        }

        Ok(())
    }

    fn zoom(
        state: &mut State,
        context: &mut Context,
        draw: &mut RaylibDrawHandle<'_>,
    ) -> anyhow::Result<()> {
        let time = (state.window.time - 1.5).max(0.0);
        let scale = (1.0 - time * 0.50).max(0.0);
        let black = (1.0 - time * 0.75).max(0.0);
        let scale = ease_in_out_cubic(scale);
        let black = ease_in_out_cubic(black);
        let shape = Vector2::new(
            draw.get_render_width() as f32,
            draw.get_render_height() as f32,
        );

        Self::draw_back(draw, state.world.is_some(), scale);

        let mut draw = draw.begin_mode2D(Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        });

        let header = 1.0 - Self::window_time_scale(&state.window);

        Self::draw_head_foot(
            &mut state.window,
            &mut draw,
            state.world.is_some(),
            "pwrmttl",
            header,
        )?;

        draw.draw_rectangle_rec(
            Rectangle::new(0.0, 0.0, shape.x, shape.y),
            Color::new(0, 0, 0, 0).lerp(Color::BLACK, 1.0 - black),
        );

        if scale == 0.0 {
            state.new_game(context)?;
        }

        Ok(())
    }

    fn layout_back(
        state: &mut State,
        draw: &mut RaylibDrawHandle<'_>,
        layout: Option<Self>,
    ) -> anyhow::Result<()> {
        if draw.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || draw.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            if layout.is_none() {
                draw.disable_cursor();
            }

            Self::change_layout(state, layout);
            state
                .window
                .scene
                .asset
                .get_sound("data/audio/back.ogg")?
                .sound
                .play();
        }

        Ok(())
    }

    fn begin(
        state: &mut State,
        context: &mut Context,
        draw: &mut RaylibDrawHandle<'_>,
    ) -> anyhow::Result<()> {
        if state.world.is_none() {
            Self::change_layout(state, Some(Layout::Zoom));
            draw.disable_cursor();
            // draw zoom for a single frame to avoid flicker on transition from begin -> zoom.
            Self::zoom(state, context, draw)?;
            return Ok(());
        }

        Self::layout_back(state, draw, Some(Layout::Main))?;
        Self::draw_back(draw, state.world.is_some(), 1.0);

        let mut layout = None;
        let mut accept = false;

        {
            let mut draw = draw.begin_mode2D(Camera2D {
                offset: Vector2::zero(),
                target: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            });

            state.window.draw(&mut draw, |window, draw| {
                Self::draw_head_foot(
                    window,
                    draw,
                    state.world.is_some(),
                    "begin",
                    Self::window_time_scale(window),
                )?;

                window.point = Self::INITIAL_POINT;

                if window.button(draw, "accept")?.press {
                    accept = true;
                };
                if window.button(draw, "return")?.press {
                    layout = Some(Self::Main);
                };

                Ok(())
            })?;
        }

        if let Some(layout) = layout {
            Self::change_layout(state, Some(layout));
        }

        if accept {
            state.new_game(context)?;
        }

        Ok(())
    }

    fn setup(state: &mut State, draw: &mut RaylibDrawHandle<'_>) -> anyhow::Result<()> {
        Self::layout_back(state, draw, Some(Layout::Main))?;
        Self::draw_back(draw, state.world.is_some(), 1.0);

        let mut draw = draw.begin_mode2D(Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        });

        let mut layout = None;

        state.window.draw(&mut draw, |window, draw| {
            Self::draw_head_foot(
                window,
                draw,
                state.world.is_some(),
                "setup",
                Self::window_time_scale(window),
            )?;

            window.point = Self::INITIAL_POINT;

            window.toggle(draw, "play tutorial", &mut state.user.tutorial)?;
            if window
                .toggle(draw, "screen full", &mut state.user.screen_full)?
                .press
            {
                if draw.is_window_fullscreen() {
                    draw.set_window_size(1024, 768);
                } else {
                    draw.set_window_size(1920, 1080);
                }

                draw.toggle_fullscreen();
            }
            window.slider(
                draw,
                "screen field",
                &mut state.user.screen_field,
                (60.0, 120.0, 1.0),
            )?;
            window.slider(
                draw,
                "screen shake",
                &mut state.user.screen_shake,
                (0.0, 2.0, 0.1),
            )?;
            window.slider(
                draw,
                "screen tilt",
                &mut state.user.screen_tilt,
                (0.0, 2.0, 0.1),
            )?;
            if window
                .slider(
                    draw,
                    "screen rate",
                    &mut state.user.screen_rate,
                    (30.0, 300.0, 1.0),
                )?
                .release
            {
                draw.set_target_fps(state.user.screen_rate as u32);
            }
            window.slider(
                draw,
                "mouse speed",
                &mut state.user.mouse_speed,
                (0.0, 2.0, 0.1),
            )?;
            window.slider(
                draw,
                "sound volume",
                &mut state.user.volume_sound,
                (0.0, 1.0, 0.1),
            )?;
            window.slider(
                draw,
                "music volume",
                &mut state.user.volume_music,
                (0.0, 1.0, 0.1),
            )?;
            window.action(draw, "move x+", &mut state.user.move_x_a)?;
            window.action(draw, "move x-", &mut state.user.move_x_b)?;
            window.action(draw, "move z+", &mut state.user.move_z_a)?;
            window.action(draw, "move z-", &mut state.user.move_z_b)?;
            window.action(draw, "jump", &mut state.user.jump)?;
            window.action(draw, "duck", &mut state.user.duck)?;
            window.action(draw, "fire a", &mut state.user.fire_a)?;
            window.action(draw, "fire b", &mut state.user.fire_b)?;

            if window.button(draw, "return")?.press {
                layout = Some(Self::Main);
            };

            Ok(())
        })?;

        if let Some(layout) = layout {
            Self::change_layout(state, Some(layout));
        }

        Ok(())
    }

    fn close(state: &mut State, draw: &mut RaylibDrawHandle<'_>) -> anyhow::Result<()> {
        Self::layout_back(state, draw, Some(Layout::Main))?;
        Self::draw_back(draw, state.world.is_some(), 1.0);

        let mut draw = draw.begin_mode2D(Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        });

        let mut layout = None;

        state.window.draw(&mut draw, |window, draw| {
            Self::draw_head_foot(
                window,
                draw,
                state.world.is_some(),
                "close",
                Self::window_time_scale(window),
            )?;

            window.point = Self::INITIAL_POINT;

            if window.button(draw, "accept")?.press {
                state.close = true;
            };
            if window.button(draw, "return")?.press {
                layout = Some(Self::Main);
            };

            Ok(())
        })?;

        if let Some(layout) = layout {
            Self::change_layout(state, Some(layout));
        }

        Ok(())
    }

    fn draw_back(handle: &mut RaylibDrawHandle, in_game: bool, scale: f32) {
        if in_game {
            return;
        }

        let time = handle.get_time() as f32 * 0.5;
        let x = time.sin() * 8.0 * scale;
        let z = time.cos() * 8.0 * scale;

        let mut draw = handle.begin_mode3D(Camera3D::perspective(
            Vector3::new(x, 6.0 * scale, z),
            Vector3::zero(),
            Vector3::up(),
            90.0,
        ));

        draw.draw_cube(Vector3::zero(), 4.0, 4.0, 4.0, Color::BLACK);

        for r in 0..8 {
            let p = r as f32 / 8.0;

            for i in 0..16 {
                let t = time * (2.0 + 4.0 * p);
                let j = (i as f32 / 8.0) * f32::consts::PI;
                let x = j.sin() * (6.0 + 24.0 * p);
                let y = t.sin() * (2.0 + 4.00 * p) - (8.0 * p);
                let z = j.cos() * (6.0 + 24.0 * p);

                draw.draw_cube(
                    Vector3::new(x, y, z),
                    1.0,
                    6.0,
                    1.0,
                    Color::new(127, 127, 127, 127),
                );
            }
        }
    }

    fn draw_head_foot(
        window: &Window,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        in_game: bool,
        text: &str,
        scale: f32,
    ) -> anyhow::Result<()> {
        let screen_size = Vector2::new(
            draw.get_render_width() as f32,
            draw.get_render_height() as f32,
        );
        let scale = ease_in_out_cubic(scale);
        let head = Rectangle::new(0.0, -72.0 * (1.0 - scale), screen_size.x, 72.0);
        let foot = Rectangle::new(0.0, screen_size.y - 72.0 * scale, screen_size.x, 72.0);
        let full = Rectangle::new(0.0, 0.0, screen_size.x, screen_size.y);

        if in_game {
            draw.draw_rectangle_rec(full, Color::new(0, 0, 0, 127));
        }
        draw.draw_rectangle_rec(head, Color::BLACK);
        draw.draw_rectangle_rec(foot, Color::BLACK);

        let font = window.font_title()?;

        let sin_a = ((draw.get_time() as f32 * 2.0).sin() * 4.0).min(0.0);
        let sin_b = (draw.get_time() as f32 * 4.0).sin().max(0.0);

        draw.draw_text_ex(
            font,
            text,
            Vector2::new(16.0, 8.0 + head.y),
            56.0,
            4.0,
            Color::GRAY.lerp(Color::BLACK, sin_b),
        );

        draw.draw_text_ex(
            font,
            text,
            Vector2::new(16.0 + sin_a, 8.0 + head.y + sin_a),
            56.0,
            4.0,
            Color::WHITE,
        );

        let font = window.font_label()?;

        draw.draw_text_ex(
            font,
            State::VERSION,
            Vector2::new(16.0, 16.0 + foot.y),
            32.0,
            4.0,
            Color::WHITE,
        );

        Ok(())
    }
}

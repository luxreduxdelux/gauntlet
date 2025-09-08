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

use crate::asset::*;
use crate::state::State;
use crate::utility::*;

use raylib::prelude::*;
use std::collections::HashMap;

pub struct Response {
    pub hover: bool,
    pub click: bool,
    pub widget: Widget,
}

#[derive(Default, Clone, Copy)]
pub struct Widget {
    pub delta: f32,
}

impl Response {
    fn new(hover: bool, click: bool, widget: Widget) -> Self {
        Self {
            hover,
            click,
            widget,
        }
    }
}

#[derive(Default)]
pub struct Window {
    widget: HashMap<String, Widget>,
    asset: Asset,
    point: Vector2,
    mouse: Vector2,
}

impl Window {
    const BUTTON_SHAPE_Y: f32 = 32.0;
    const FONT_SPACE: f32 = 1.0;

    pub fn initialize(
        &mut self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> anyhow::Result<()> {
        self.asset.set_font(handle, thread, "data/font_large.ttf")?;

        Ok(())
    }

    fn begin(&mut self, handle: &RaylibHandle) {
        self.point = Vector2::new(8.0, 8.0);
        self.mouse = handle.get_mouse_position();
    }
    fn close(&mut self) {}

    fn font(&self) -> anyhow::Result<&Font> {
        self.asset.get_font("data/font_large.ttf")
    }

    fn response(&mut self, handle: &RaylibHandle, text: &str, shape: Rectangle) -> Response {
        let hover = shape.check_collision_point_rec(self.mouse);
        let click = handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        let entry = self.widget.entry(text.to_string()).or_default();
        let frame = handle.get_frame_time();

        if hover {
            entry.delta += frame * 6.0;
        } else {
            entry.delta -= frame * 6.0;
        }

        entry.delta = entry.delta.clamp(0.0, 1.0);

        Response::new(hover, hover && click, *entry)
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

    pub fn button(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
    ) -> anyhow::Result<Response> {
        let font = self.font()?;
        let size = font.measure_text(text, Self::BUTTON_SHAPE_Y, Self::FONT_SPACE);
        let mut size = Rectangle::new(
            self.point.x,
            self.point.y,
            size.x + 8.0,
            Self::BUTTON_SHAPE_Y,
        );
        let response = self.response(draw, text, size);
        let delta = ease_in_out_cubic(response.widget.delta);
        let white = Color::WHITE.lerp(Color::BLACK, delta);
        let black = Color::BLACK.lerp(Color::WHITE, delta);

        size.x += delta * 8.0;
        draw.draw_rectangle_rec(size, black);

        draw.draw_text_ex(
            self.font()?,
            text,
            self.point + Vector2::new(4.0 + delta * 8.0, 0.0),
            Self::BUTTON_SHAPE_Y,
            Self::FONT_SPACE,
            white,
        );

        self.point.y += Self::BUTTON_SHAPE_Y + 4.0;

        Ok(response)
    }
}

#[derive(Default)]
pub enum Layout {
    Intro,
    #[default]
    Main,
    Begin,
    Setup,
    Close,
}

impl Layout {
    pub fn draw(
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        match state.layout {
            Layout::Main => Self::main(state, draw),
            Layout::Begin => Self::begin(state, draw),
            Layout::Setup => Self::setup(state, draw),
            Layout::Close => Self::close(state, draw),
            _ => Ok(()),
        }
    }

    fn draw_head_foot(
        window: &Window,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
    ) -> anyhow::Result<()> {
        let screen_size = Vector2::new(
            draw.get_render_width() as f32,
            draw.get_render_height() as f32,
        );
        let head = Rectangle::new(0.0, 0.0, screen_size.x, 64.0);
        let foot = Rectangle::new(0.0, screen_size.y - 64.0, screen_size.x, 64.0);

        draw.draw_rectangle_rec(head, Color::BLACK);
        draw.draw_rectangle_rec(foot, Color::BLACK);

        let font = window.font()?;

        draw.draw_text_ex(font, text, Vector2::new(8.0, -4.0), 64.0, 1.0, Color::WHITE);

        Ok(())
    }

    fn main(
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        state.window.draw(draw, |window, draw| {
            Self::draw_head_foot(window, draw, "pwrmttl")?;

            window.point = Vector2::new(8.0, 72.0);

            if window.button(draw, "begin")?.click {
                state.layout = Self::Begin;
            };
            if window.button(draw, "setup")?.click {
                state.layout = Self::Setup;
            };
            if window.button(draw, "close")?.click {
                state.layout = Self::Close;
            };

            Ok(())
        })
    }

    fn begin(
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn setup(
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        state.window.draw(draw, |window, draw| {
            Self::draw_head_foot(window, draw, "setup")?;

            window.point = Vector2::new(8.0, 72.0);

            if window.button(draw, "return")?.click {
                state.layout = Self::Main;
            };

            Ok(())
        })
    }

    fn close(
        state: &mut State,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        state.window.draw(draw, |window, draw| {
            Self::draw_head_foot(window, draw, "close")?;

            window.point = Vector2::new(8.0, 72.0);

            if window.button(draw, "accept")?.click {
                state.close = true;
            };
            if window.button(draw, "return")?.click {
                state.layout = Self::Main;
            };

            Ok(())
        })
    }
}

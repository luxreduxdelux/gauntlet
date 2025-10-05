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
use crate::scene::*;
use crate::user::*;
use crate::utility::*;

//================================================================

use hashbrown::HashMap;
use raylib::prelude::*;
use std::f32;
use std::fmt::Display;

//================================================================

pub struct Logger {
    pub active: bool,
    buffer: String,
    activity: Vec<String>,
    suggest: Vec<LoggerSuggest>,
    history: Vec<LoggerLine>,
    command: HashMap<String, LoggerCommand>,
    scroll: f32,
}

struct LoggerCommand {
    info: String,
    call: Box<dyn FnMut(&mut App, &mut Context, Vec<&str>) -> anyhow::Result<()>>,
}

struct LoggerSuggest {
    name: String,
    info: String,
}

struct LoggerLine {
    text: String,
    kind: LoggerKind,
    time: f32,
}

enum LoggerKind {
    History,
    Command,
    Warning,
    Failure,
}

impl LoggerKind {
    fn color(&self) -> Color {
        match self {
            Self::History => Color::new(127, 127, 127, 255),
            Self::Command => Color::new(255, 255, 255, 255),
            Self::Warning => Color::new(255, 127, 0, 255),
            Self::Failure => Color::new(255, 0, 0, 255),
        }
    }
}

impl Logger {
    fn history_check(&mut self) {
        if self.history.len() > 32 {
            self.history.remove(0);
        }
    }

    pub fn print_history(&mut self, text: &str) {
        let time = unsafe { ffi::GetTime() as f32 };

        self.history.push(LoggerLine {
            text: format!("({time:.2}) {text}"),
            kind: LoggerKind::History,
            time: 4.0,
        });

        self.history_check();
    }

    pub fn print_command(&mut self, text: &str) {
        let time = unsafe { ffi::GetTime() as f32 };

        self.history.push(LoggerLine {
            text: format!("({time:.2}) {text}"),
            kind: LoggerKind::Command,
            time: 4.0,
        });

        self.history_check();
    }

    pub fn print_warning(&mut self, text: &str) {
        let time = unsafe { ffi::GetTime() as f32 };

        self.history.push(LoggerLine {
            text: format!("({time:.2}) {text}"),
            kind: LoggerKind::Warning,
            time: 4.0,
        });

        self.history_check();
    }

    pub fn print_failure(&mut self, text: &str) {
        let time = unsafe { ffi::GetTime() as f32 };

        self.history.push(LoggerLine {
            text: format!("({time:.2}) {text}"),
            kind: LoggerKind::Failure,
            time: 4.0,
        });

        self.history_check();
    }

    fn draw(
        &mut self,
        app: &mut App,
        context: &mut Context,
        draw: &mut RaylibDrawHandle<'_>,
    ) -> anyhow::Result<()> {
        if self.active {
            self.draw_active(app, context, draw)
        } else {
            self.draw_hidden(app, context, draw)
        }
    }

    fn complete_suggest(&mut self) {
        if !self.suggest.is_empty() {
            let mut select = None;

            for (i, entry) in self.suggest.iter().enumerate() {
                if entry.name.starts_with(&self.buffer) {
                    select = Some(i);
                }
            }

            if let Some(index) = select
                && let Some(suggest) = self.suggest.get(index + 1)
            {
                self.buffer = suggest.name.clone();
            } else {
                self.buffer = self.suggest[0].name.clone();
            }
        }
    }

    fn complete_activity(&mut self, direction: i32) {
        if !self.activity.is_empty() {
            let mut select = None;

            if !self.buffer.is_empty() {
                for (i, entry) in self.activity.iter().enumerate() {
                    if entry.starts_with(&self.buffer) {
                        select = Some(i);
                        break;
                    }
                }
            }

            if let Some(index) = select {
                let index = (index as i32 + direction).rem_euclid(self.activity.len() as i32);

                self.buffer = self.activity[index as usize].clone();
            } else {
                self.buffer = self.activity.last().unwrap().clone();
            }
        }
    }

    fn build_suggest(&mut self) {
        self.suggest.clear();

        if !self.buffer.is_empty() {
            for (entry, command) in &self.command {
                if entry.starts_with(&self.buffer) {
                    self.suggest.push(LoggerSuggest {
                        name: entry.to_string(),
                        info: command.info.clone(),
                    });
                }
            }
        }
    }

    fn attach_character(&mut self, character: char) {
        self.buffer.push(character);
        self.build_suggest();
    }

    fn detach_character(&mut self) {
        self.buffer.pop();
        self.build_suggest();
    }

    fn wipe_character(&mut self) {
        self.buffer.clear();
        self.build_suggest();
    }

    fn draw_active(
        &mut self,
        app: &mut App,
        context: &mut Context,
        draw: &mut RaylibDrawHandle<'_>,
    ) -> anyhow::Result<()> {
        if let Some(key) = draw.get_key_pressed() {
            match key {
                KeyboardKey::KEY_ENTER => {
                    let text = self.buffer.clone();
                    self.print_history(&text);
                    self.activity.push(text);
                    self.evaluate(app, context)?;
                    self.wipe_character();
                }
                KeyboardKey::KEY_TAB => {
                    self.complete_suggest();
                }
                KeyboardKey::KEY_UP => {
                    self.complete_activity(1 * -1);
                }
                KeyboardKey::KEY_DOWN => {
                    self.complete_activity(1);
                }
                KeyboardKey::KEY_BACKSPACE => {
                    self.detach_character();
                }
                KeyboardKey::KEY_F3 => {
                    self.active = false;
                }
                _ => {}
            }
        }

        if let Some(character) = draw.get_char_pressed() {
            self.attach_character(character);
        }

        let scale = Vector2::new(
            draw.get_screen_width() as f32,
            draw.get_screen_height() as f32 * 0.5,
        );

        draw.draw_rectangle_rec(
            Rectangle::new(0.0, 0.0, scale.x, scale.y),
            Color::new(0, 0, 0, 192),
        );

        let font = app.window.font_label()?;

        let delta = draw.get_mouse_wheel_move();

        self.scroll = (self.scroll + delta).max(0.0);

        unsafe {
            ffi::BeginScissorMode(0, 0, scale.x as i32, (scale.y - 40.0) as i32);
        }

        for (i, line) in self.history.iter().rev().enumerate() {
            Window::font_draw(
                draw,
                font,
                &line.text,
                Vector2::new(
                    8.0,
                    scale.y - 40.0 - 32.0 * (i + 1) as f32 + 32.0 * self.scroll,
                ),
                line.kind.color(),
            );
        }

        unsafe {
            ffi::EndScissorMode();
        }

        for (i, line) in self.suggest.iter().enumerate() {
            let point = Vector2::new(0.0, scale.y + 32.0 * i as f32);

            draw.draw_rectangle_rec(
                Rectangle::new(point.x, point.y, scale.x, 32.0),
                Color::new(0, 0, 0, 127),
            );

            Window::font_draw(
                draw,
                font,
                &line.name,
                point + Vector2::new(8.0, 0.0),
                Color::WHITE,
            );

            let measure = Window::font_measure(font, &line.info);

            Window::font_draw(
                draw,
                font,
                &line.info,
                point + Vector2::new(scale.x - measure.x - 8.0, 0.0),
                Color::new(192, 192, 192, 255),
            );
        }

        Window::font_draw(
            draw,
            font,
            &self.buffer,
            Vector2::new(8.0, scale.y - 40.0),
            Color::WHITE,
        );

        Ok(())
    }

    fn draw_hidden(
        &mut self,
        app: &mut App,
        context: &mut Context,
        draw: &mut RaylibDrawHandle<'_>,
    ) -> anyhow::Result<()> {
        if draw.is_key_pressed(KeyboardKey::KEY_F3) {
            self.active = true;
        }

        let font = app.window.font_label()?;

        for (i, line) in self.history.iter_mut().rev().enumerate() {
            if i >= 4 || line.time == 0.0 {
                break;
            }

            line.time = (line.time - context.handle.get_frame_time()).max(0.0);

            Window::font_draw(
                draw,
                font,
                &line.text,
                Vector2::new(8.0, 8.0 + 32.0 * i as f32),
                line.kind.color(),
            );
        }

        Ok(())
    }

    fn evaluate(&mut self, app: &mut App, context: &mut Context) -> anyhow::Result<()> {
        if !self.buffer.is_empty() {
            let split: Vec<&str> = self.buffer.split(" ").collect();

            if let Some(entry) = split.first() {
                if let Some(command) = self.command.get_mut(*entry) {
                    (command.call)(app, context, split)?;
                } else {
                    self.print_failure(&format!("Unknown command \"{entry}\"."));
                }
            }
        }

        Ok(())
    }

    fn register_command<
        F: FnMut(&mut App, &mut Context, Vec<&str>) -> anyhow::Result<()> + 'static,
    >(
        table: &mut HashMap<String, LoggerCommand>,
        entry: &str,
        info: &str,
        call: F,
    ) {
        table.insert(
            entry.to_string(),
            LoggerCommand {
                info: info.to_string(),
                call: Box::new(call),
            },
        );
    }

    //================================================================

    fn find(app: &mut App, _: &mut Context, token: Vec<&str>) -> anyhow::Result<()> {
        if let Some(token) = token.get(1) {
            let mut result = Vec::with_capacity(app.window.logger.command.len());

            for (entry, command) in app.window.logger.command.iter() {
                if entry.contains(token) {
                    result.push(format!("{entry}: {}", command.info));
                }
            }

            result
                .iter()
                .for_each(|x| app.window.logger.print_command(&x));
        } else {
            app.window.logger.print_failure("Usage: find {sub-string}");
        }

        Ok(())
    }

    fn clear(app: &mut App, _: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.window.logger.history.clear();

        Ok(())
    }

    fn close(app: &mut App, _: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.close = true;

        Ok(())
    }

    fn reset(app: &mut App, context: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        *app = App::default();
        app.initialize(context)?;

        Ok(())
    }

    fn reset_world(app: &mut App, context: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.new_world(context)?;
        app.window.logger.active = false;

        Ok(())
    }

    fn debug_draw_physical(app: &mut App, _: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.user.debug_draw_physical = !app.user.debug_draw_physical;

        Ok(())
    }

    fn debug_draw_entity(app: &mut App, _: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.user.debug_draw_entity = !app.user.debug_draw_entity;

        Ok(())
    }

    fn debug_frame_rate(app: &mut App, _: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.user.debug_frame_rate = !app.user.debug_frame_rate;

        Ok(())
    }

    fn debug_light_edit(app: &mut App, _: &mut Context, _: Vec<&str>) -> anyhow::Result<()> {
        app.user.debug_light_edit = !app.user.debug_light_edit;

        Ok(())
    }
}

impl Default for Logger {
    #[rustfmt::skip]
    fn default() -> Self {
        let mut command = HashMap::default();
        Self::register_command(&mut command, "find",                "Find a command by sub-string.",       Self::find);
        Self::register_command(&mut command, "clear",               "Clear logger history.",               Self::clear);
        Self::register_command(&mut command, "close",               "Close Gauntlet Complex.",             Self::close);
        Self::register_command(&mut command, "reset",               "Reset the app state.",                Self::reset);
        Self::register_command(&mut command, "reset_world",         "Reset the world state.",              Self::reset_world);
        Self::register_command(&mut command, "debug_draw_physical", "Draw the world physical simulation.", Self::debug_draw_physical);
        Self::register_command(&mut command, "debug_draw_entity",   "Draw point entity.",                  Self::debug_draw_entity);
        Self::register_command(&mut command, "debug_frame_rate",    "Draw the current frame rate.",        Self::debug_frame_rate);
        Self::register_command(&mut command, "debug_light_edit",    "Draw the light editor.",              Self::debug_light_edit);

        Self {
            active: false,
            buffer: Default::default(),
            activity: Default::default(),
            suggest: Default::default(),
            history: Default::default(),
            command,
            scroll: Default::default(),
        }
    }
}

//================================================================

#[derive(Default)]
pub struct Window<'a> {
    widget: HashMap<usize, Widget>,
    scene: Scene<'a>,
    point: Vector2,
    index: usize,
    pub logger: Logger,
    device: Device,
    focus: Option<usize>,
    view: Option<(Rectangle, f32)>,
    time: f32,
    glyph_kind: GlyphKind,
}

impl<'a> Window<'a> {
    const BUTTON_SHAPE_Y: f32 = 32.0;
    const FONT_SCALE: f32 = 32.0;
    const FONT_SPACE: f32 = 1.0;

    pub fn set_device(&mut self, device: Device) {
        self.device = device;
    }

    #[rustfmt::skip]
    pub fn initialize(&mut self, app: &App, context: &'a mut Context) -> anyhow::Result<()> {
        let glyph_kind = ["play_station", "nintendo", "xbox"];
        let glyph_list = [
            "button_d.png",
            "button_l.png",
            "button_r.png",
            "button_u.png",
            "l_bumper.png",
            "l_stick_click.png",
            "l_stick.png",
            "l_trigger.png",
            "pad_d.png",
            "pad_l.png",
            "pad_r.png",
            "pad_u.png",
            "middle_r.png",
            "r_bumper.png",
            "r_stick_click.png",
            "r_stick.png",
            "r_trigger.png",
            "middle_l.png",
        ];

        for kind in glyph_kind {
            for list in glyph_list {
                self.scene.asset.set_texture(context, &format!("data/video/glyph/{kind}/{list}"))?;
            }
        }

        self.scene.initialize(app, context)?;

        Room::new(&mut self.scene, context, "data/video/menu.glb")?;

        self.scene.link()?;

        self.scene.asset.set_texture(context, "data/video/glyph/mouse/button_l.png")?;
        self.scene.asset.set_texture(context, "data/video/glyph/mouse/button_m.png")?;
        self.scene.asset.set_texture(context, "data/video/glyph/mouse/button_r.png")?;
        self.scene.asset.set_texture(context, "data/video/glyph/mouse/wheel_u.png")?;
        self.scene.asset.set_texture(context, "data/video/glyph/mouse/wheel_d.png")?;

        self.scene.asset.set_font(context, "data/video/font_label.ttf", 32)?;
        self.scene.asset.set_font(context, "data/video/font_title.ttf", 56)?;
        self.scene.asset.set_sound(context, "data/audio/hover.ogg", 0)?;
        self.scene.asset.set_sound(context, "data/audio/click.ogg", 0)?;
        self.scene.asset.set_sound(context, "data/audio/back.ogg", 0)?;

        Ok(())
    }

    fn begin(&mut self) {
        self.point = Vector2::new(8.0, 8.0);
        self.index = usize::default();
    }

    fn close(&mut self, handle: &mut RaylibHandle) {
        self.device.update_index(handle, self.index);
        self.device = self.device.poll_change(handle);
    }

    pub fn font_label(&self) -> anyhow::Result<&Font> {
        self.scene.asset.get_font("data/video/font_label.ttf")
    }

    fn font_measure(font: &Font, text: &str) -> Vector2 {
        font.measure_text(text, Self::FONT_SCALE, Self::FONT_SPACE)
    }

    pub fn font_draw(
        draw: &mut RaylibDrawHandle<'_>,
        font: &Font,
        text: &str,
        point: Vector2,
        color: Color,
    ) {
        draw.draw_text_ex(font, text, point, Self::FONT_SCALE, Self::FONT_SPACE, color);
    }

    /// Begin a new UI frame.
    pub fn draw<
        T: FnMut(&mut App, &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>) -> anyhow::Result<()>,
    >(
        app: &mut App<'a>,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        mut call: T,
    ) -> anyhow::Result<()> {
        app.window.glyph_kind = app.user.video_glyph;

        app.window.begin();

        call(app, draw)?;

        app.window.close(draw);

        Ok(())
    }

    pub fn draw_input(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        input: &Input,
        point: Vector2,
    ) -> anyhow::Result<()> {
        match input {
            Input::Board { .. } => {
                Window::font_draw(
                    draw,
                    self.font_label()?,
                    &format!("{}", input),
                    point,
                    Color::WHITE,
                );
            }
            Input::Mouse { key, .. } => {
                let key = Input::to_mouse(*key);

                let texture = match key {
                    MouseButton::MOUSE_BUTTON_LEFT => "button_l.png",
                    MouseButton::MOUSE_BUTTON_RIGHT => "button_r.png",
                    MouseButton::MOUSE_BUTTON_MIDDLE => "button_m.png",
                    MouseButton::MOUSE_BUTTON_SIDE => "button_l.png",
                    MouseButton::MOUSE_BUTTON_EXTRA => "button_l.png",
                    MouseButton::MOUSE_BUTTON_FORWARD => "button_l.png",
                    MouseButton::MOUSE_BUTTON_BACK => "button_l.png",
                };

                let texture = self
                    .scene
                    .asset
                    .get_texture(&format!("data/video/glyph/mouse/{texture}"))?;

                draw.draw_texture_ex(texture, point, 0.0, 0.35, Color::WHITE);
            }
            Input::Pad { key, .. } => {
                let key = Input::to_pad(*key);

                let texture = match key {
                    GamepadButton::GAMEPAD_BUTTON_UNKNOWN => "pad_u.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP => "pad_u.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT => "pad_r.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN => "pad_d.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT => "pad_l.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP => "button_u.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT => "button_r.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN => "button_d.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT => "button_l.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1 => "l_bumper.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_2 => "l_trigger.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1 => "r_bumper.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2 => "r_trigger.png",
                    GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT => "middle_l.png",
                    GamepadButton::GAMEPAD_BUTTON_MIDDLE => "middle.png",
                    GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT => "middle_r.png",
                    GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB => "l_stick_click.png",
                    GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB => "r_stick_click.png",
                };

                let texture = self.scene.asset.get_texture(&format!(
                    "data/video/glyph/{}/{texture}",
                    self.glyph_kind.folder_name()
                ))?;

                draw.draw_texture_ex(texture, point, 0.0, 0.35, Color::WHITE);
            }
        }

        Ok(())
    }

    fn check_visibility(&mut self, handle: &RaylibHandle, shape: Rectangle) -> bool {
        if let Some((view, scroll)) = &mut self.view {
            if view.check_collision_recs(&shape) {
                true
            } else if self.device.hover(handle, self.index, Rectangle::default()) {
                *scroll = (view.y + *scroll) - shape.y;
                true
            } else {
                self.index += 1;
                self.point.y += Self::BUTTON_SHAPE_Y + 4.0;
                false
            }
        } else {
            true
        }
    }

    pub fn button(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
    ) -> anyhow::Result<Response> {
        let size = Self::font_measure_box(
            self.font_label()?,
            text,
            Rectangle::new(self.point.x, self.point.y, 24.0, Self::BUTTON_SHAPE_Y),
        )?;

        if !self.check_visibility(draw, size) {
            return Ok(Response::default());
        }

        let response = Response::new_from_window(draw, self, size);
        let color = if response.hover {
            (Color::WHITE, Color::BLACK)
        } else {
            (Color::BLACK, Color::WHITE)
        };

        //================================================================

        draw.draw_rectangle_rec(size, color.0);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            self.point + Vector2::new(4.0, -2.0),
            color.1,
        );

        if response.hover {
            Device::draw_glyph_response(
                self,
                draw,
                Vector2::new(8.0, 0.0),
                "Accept",
                DeviceResponse::Accept,
            )?;
        }

        //================================================================

        self.index += 1;
        self.point.y += Self::BUTTON_SHAPE_Y + 4.0;

        Ok(response)
    }

    pub fn toggle(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut bool,
    ) -> anyhow::Result<Response> {
        let size = Self::font_measure_box(
            self.font_label()?,
            text,
            Rectangle::new(self.point.x, self.point.y, 24.0, Self::BUTTON_SHAPE_Y),
        )?;

        if !self.check_visibility(draw, size) {
            return Ok(Response::default());
        }

        //================================================================

        draw.draw_rectangle_rec(size, Color::BLACK);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            self.point + Vector2::new(4.0, -2.0),
            Color::WHITE,
        );

        //================================================================

        let size_a = Rectangle::new(
            size.x + size.width + 4.0,
            size.y,
            Self::BUTTON_SHAPE_Y,
            Self::BUTTON_SHAPE_Y,
        );
        let size_b = Rectangle::new(
            size_a.x + 4.0,
            size_a.y + 4.0,
            size_a.width - 8.0,
            size_a.height - 8.0,
        );
        let response = Response::new_from_window(draw, self, size_a);
        let color = if response.hover {
            (Color::WHITE, Color::BLACK)
        } else {
            (Color::BLACK, Color::WHITE)
        };
        if response.accept() {
            *value = !*value;
        }

        draw.draw_rectangle_rec(size_a, color.0);
        if response.hover {
            Device::draw_glyph_response(
                self,
                draw,
                Vector2::new(8.0, 0.0),
                "Modify",
                DeviceResponse::Accept,
            )?;
        }

        if *value {
            draw.draw_rectangle_rec(size_b, color.1);
        }

        //================================================================

        self.index += 1;
        self.point.y += Self::BUTTON_SHAPE_Y + 4.0;

        Ok(response)
    }

    pub fn slider(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut f32,
        bound: (f32, f32),
        step: f32,
    ) -> anyhow::Result<Response> {
        let size = Self::font_measure_box(
            self.font_label()?,
            text,
            Rectangle::new(self.point.x, self.point.y, 24.0, Self::BUTTON_SHAPE_Y),
        )?;

        if !self.check_visibility(draw, size) {
            return Ok(Response::default());
        }

        //================================================================

        draw.draw_rectangle_rec(size, Color::BLACK);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            self.point + Vector2::new(4.0, -2.0),
            Color::WHITE,
        );

        let size_a = Rectangle::new(
            size.x + size.width + 4.0,
            size.y,
            128.0,
            Self::BUTTON_SHAPE_Y,
        );
        let size_b = Rectangle::new(
            size_a.x + 4.0,
            size_a.y + 4.0,
            (size_a.width - 8.0) * percentage_from_value(*value, bound.0, bound.1),
            size_a.height - 8.0,
        );
        let size_c = Rectangle::new(
            size_a.x + size_a.width * 0.5 - 32.0,
            size_a.y + 8.0,
            64.0,
            size_a.height - 16.0,
        );

        //================================================================

        let response = Response::new_from_window(draw, self, size_a);
        let color = if response.hover {
            (Color::WHITE, Color::BLACK)
        } else {
            (Color::BLACK, Color::WHITE)
        };

        if let Some((DeviceResponse::Accept, true)) = response.device
            && self.device.is_mouse()
        {
            self.focus = Some(self.index)
        }

        if response.focus {
            if let Some((DeviceResponse::Accept, false)) = response.device {
                if self.device.is_mouse() {
                    self.focus = None
                }
            } else {
                let delta = draw.get_mouse_delta().x;

                if delta.abs() > 0.0 {
                    let point = draw.get_mouse_position();
                    let point = percentage_from_value(point.x, size_a.x, size_a.x + size_a.width)
                        .clamp(0.0, 1.0);
                    let end = value_from_percentage(point, bound.0, bound.1);
                    let end = snap_to_grid(end, step);
                    *value = end;
                }

                if response.side_a() {
                    *value -= step;
                } else if response.side_b() {
                    *value += step;
                }

                *value = (*value).clamp(bound.0, bound.1);
            }
        } else if !self.device.is_mouse() {
            if response.side_a() {
                *value -= step;
            } else if response.side_b() {
                *value += step;
            }

            *value = (*value).clamp(bound.0, bound.1);
        }

        //================================================================

        draw.draw_rectangle_rec(size_a, color.0);
        draw.draw_rectangle_rec(size_b, color.1);
        draw.draw_rectangle_rec(size_c, Color::BLACK);
        let text = &format!("{:.2}", value);
        let measure = Self::font_measure(self.font_label()?, text);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            Vector2::new(
                size_c.x + size_c.width * 0.5 - measure.x * 0.5,
                size_c.y - 10.0,
            ),
            Color::WHITE,
        );
        if response.hover {
            let push = if self.device.is_mouse() {
                Device::draw_glyph_response(
                    self,
                    draw,
                    Vector2::new(8.0, 0.0),
                    if response.focus { "" } else { "Modify" },
                    DeviceResponse::Accept,
                )?;

                40.0
            } else {
                0.0
            };

            if !self.device.is_mouse() || (self.device.is_mouse() && response.focus) {
                Device::draw_glyph_response(
                    self,
                    draw,
                    Vector2::new(8.0 + push, 0.0),
                    "Modify",
                    DeviceResponse::SideA,
                )?;
            }
        }

        self.index += 1;
        self.point.y += Self::BUTTON_SHAPE_Y + 4.0;

        Ok(response)
    }

    fn font_measure_box(font: &Font, text: &str, shape: Rectangle) -> anyhow::Result<Rectangle> {
        let size = Self::font_measure(font, text);
        Ok(Rectangle::new(
            shape.x,
            shape.y,
            shape.width + size.x,
            shape.height,
        ))
    }

    pub fn switch<T: PartialEq + Copy + Display>(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut T,
        bound: &[T],
    ) -> anyhow::Result<Response> {
        let size = Self::font_measure_box(
            self.font_label()?,
            text,
            Rectangle::new(self.point.x, self.point.y, 24.0, Self::BUTTON_SHAPE_Y),
        )?;

        if !self.check_visibility(draw, size) {
            return Ok(Response::default());
        }

        //================================================================

        draw.draw_rectangle_rec(size, Color::BLACK);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            self.point + Vector2::new(4.0, -2.0),
            Color::WHITE,
        );

        let size_a = Rectangle::new(
            size.x + size.width + 4.0,
            size.y,
            128.0,
            Self::BUTTON_SHAPE_Y,
        );
        let size_b = Rectangle::new(
            size_a.x + 4.0,
            size_a.y + 4.0,
            size_a.width - 8.0,
            size_a.height - 8.0,
        );

        //================================================================

        let response = Response::new_from_window(draw, self, size_a);
        let color = if response.hover {
            (Color::WHITE, Color::BLACK)
        } else {
            (Color::BLACK, Color::WHITE)
        };

        let side_a = !self.device.is_mouse() && response.side_a();
        let side_b = (!self.device.is_mouse() && response.side_b()) || response.accept();

        if side_a {
            let mut pick = bound.last();

            for (i, choice) in bound.iter().enumerate() {
                if *choice == *value && i > 0 {
                    pick = bound.get(i - 1);
                    break;
                }
            }

            if let Some(pick) = pick {
                *value = *pick;
            }
        }

        if side_b {
            let mut pick = bound.first();

            for (i, choice) in bound.iter().enumerate() {
                if *choice == *value {
                    if let Some(choice) = bound.get(i + 1) {
                        pick = Some(choice)
                    }
                    break;
                }
            }

            if let Some(pick) = pick {
                *value = *pick;
            }
        }

        //================================================================

        draw.draw_rectangle_rec(size_a, color.0);
        draw.draw_rectangle_rec(size_b, color.0);
        let text = &*value.to_string();
        let measure = Self::font_measure(self.font_label()?, text);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            Vector2::new(
                size_b.x + size_b.width * 0.5 - measure.x * 0.5,
                size_b.y - 6.0,
            ),
            color.1,
        );
        if response.hover {
            if self.device.is_mouse() {
                Device::draw_glyph_response(
                    self,
                    draw,
                    Vector2::new(8.0, 0.0),
                    "Modify",
                    DeviceResponse::Accept,
                )?;
            } else {
                Device::draw_glyph_response(
                    self,
                    draw,
                    Vector2::new(8.0, 0.0),
                    "Modify",
                    DeviceResponse::SideA,
                )?;
            }
        }

        self.index += 1;
        self.point.y += Self::BUTTON_SHAPE_Y + 4.0;

        Ok(response)
    }

    pub fn action(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        text: &str,
        value: &mut Input,
    ) -> anyhow::Result<Response> {
        let size = Self::font_measure_box(
            self.font_label()?,
            text,
            Rectangle::new(self.point.x, self.point.y, 24.0, Self::BUTTON_SHAPE_Y),
        )?;

        if !self.check_visibility(draw, size) {
            return Ok(Response::default());
        }

        //================================================================

        draw.draw_rectangle_rec(size, Color::BLACK);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            self.point + Vector2::new(4.0, -2.0),
            Color::WHITE,
        );

        let size_a = Rectangle::new(
            size.x + size.width + 4.0,
            size.y,
            128.0,
            Self::BUTTON_SHAPE_Y,
        );
        let size_b = Rectangle::new(
            size_a.x + 4.0,
            size_a.y + 4.0,
            size_a.width - 8.0,
            size_a.height - 8.0,
        );

        //================================================================

        let response = Response::new_from_window(draw, self, size_a);
        let color = if response.hover {
            (Color::WHITE, Color::BLACK)
        } else {
            (Color::BLACK, Color::WHITE)
        };

        if response.focus {
            if let Some(board) = draw.get_key_pressed() {
                *value = Input::new_board(board);
                self.focus = None;
            }

            if let Some(mouse) = Input::get_mouse_pressed(draw) {
                *value = Input::new_mouse(mouse);
                self.focus = None;
            }

            if let Some(pad) = Input::get_gamepad_button_pressed(draw, 0) {
                *value = Input::new_pad(pad);
                self.focus = None;
            }
        } else if let Some((DeviceResponse::Accept, true)) = response.device {
            self.focus = Some(self.index)
        }

        //================================================================

        draw.draw_rectangle_rec(size_a, color.0);
        draw.draw_rectangle_rec(size_b, color.0);
        self.draw_input(
            draw,
            value,
            Vector2::new(size_b.x + size_b.width * 0.5, size_b.y - 6.0),
        )?;
        if response.hover && !response.focus {
            Device::draw_glyph_response(
                self,
                draw,
                Vector2::new(8.0, 0.0),
                "Modify",
                DeviceResponse::Accept,
            )?;
        }

        /*
        let text = &*value.to_string();
        let measure = Self::font_measure(self.font_label()?, text);
        Self::font_draw(
            draw,
            self.font_label()?,
            text,
            Vector2::new(
                size_b.x + size_b.width * 0.5 - measure.x * 0.5,
                size_b.y - 6.0,
            ),
            color.1,
        );
        */

        self.index += 1;
        self.point.y += Self::BUTTON_SHAPE_Y + 4.0;

        Ok(response)
    }

    pub fn scroll<
        F: FnMut(&mut Self, &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>) -> anyhow::Result<()>,
    >(
        &mut self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        shape: Vector2,
        mut call: F,
    ) -> anyhow::Result<()> {
        let size = Rectangle::new(self.point.x, self.point.y, shape.x, shape.y);
        let form = self.point.y;
        let response = Response::new_from_window(draw, self, size);
        let scroll = { self.widget.entry(self.index).or_default().scroll };
        let index = self.index;

        //================================================================

        self.point.y += scroll;

        let full = self.point.y;

        //draw.draw_rectangle_rec(size, Color::RED);

        self.view = Some((size, 0.0));

        call(self, draw)?;

        let full = self.point.y - full;

        if self.device.is_mouse() {
            let entry = self.widget.entry(index).or_default();

            if response.side_a() {
                entry.scroll -= Self::BUTTON_SHAPE_Y + 4.0;
            }

            if response.side_b() {
                entry.scroll += Self::BUTTON_SHAPE_Y + 4.0;
            }

            let min = (-full + 4.0 + shape.y).min(0.0);
            let max = (-full + 4.0 + shape.y).max(0.0);

            entry.scroll = entry.scroll.clamp(min, max);
        } else {
            let entry = self.widget.entry(index).or_default();

            entry.scroll += self.view.as_ref().unwrap().1;
        }

        self.view = None;

        //================================================================

        self.point.y = form + shape.y + 4.0;

        Ok(())
    }
}

//================================================================

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

    pub fn set_layout(app: &mut App, handle: &mut RaylibHandle, layout: Option<Self>) {
        if layout.is_none() {
            handle.disable_cursor();
        }

        app.window.index = usize::default();
        app.layout = layout;
        app.window.time = 0.0;
        app.window.widget.clear();
    }

    pub fn draw(
        app: &mut App,
        draw: &mut RaylibDrawHandle<'_>,
        context: &mut Context,
    ) -> anyhow::Result<()> {
        app.window.time += draw.get_frame_time();

        if app.window.device.escape(draw)
            && let Some(world) = &mut app.world
        {
            world.scene.set_pause(world.scene.pause)?;
        }

        let pause = app.layout.is_some();

        unsafe {
            let app_ref = app as *mut App;
            let ctx_ref = context as *mut Context;

            if pause {
                app.window.scene.camera_3d = Camera3D::perspective(
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(1.0, 1.0, 0.0),
                    Vector3::up(),
                    90.0,
                );

                app.window.scene.update(&*app_ref, context)?;

                app.window.scene.draw_3d(&mut *context, draw, |draw| {
                    //
                    Ok(())
                })?;

                app.window.scene.draw_2d(&mut *context, draw, |draw| {
                    let app = &mut *app_ref;
                    let context = &mut *ctx_ref;

                    if let Some(layout) = &app.layout {
                        match layout {
                            Layout::Main => Self::main(app, draw),
                            Layout::Begin => Self::begin(app, context, draw),
                            Layout::Setup => Self::setup(app, draw),
                            Layout::Close => Self::close(app, draw),
                            _ => Ok(()),
                        }?;
                    }

                    Ok(())
                })?;
            } else if app.window.device.escape(draw) {
                Self::set_layout(app, draw, Some(Layout::Main));
                draw.enable_cursor();
            }

            if app.user.developer {
                app.window.logger.draw(&mut *app_ref, context, draw)?;
            }
        }

        Ok(())
    }

    fn main(
        app: &mut App,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        if app.world.is_some() {
            Self::layout_back(app, draw, None)?;
        }

        let mut layout = None;

        Window::draw(app, draw, |app, draw| {
            app.window.point = Self::INITIAL_POINT;

            if app.window.button(draw, "begin")?.accept() {
                layout = Some(Self::Begin);
            };
            if app.window.button(draw, "setup")?.accept() {
                layout = Some(Self::Setup);
            };
            if app.window.button(draw, "close")?.accept() {
                layout = Some(Self::Close);
            };

            Ok(())
        })?;

        if let Some(layout) = layout {
            Self::set_layout(app, draw, Some(layout));
        }

        Ok(())
    }

    fn layout_back(
        app: &mut App,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        layout: Option<Self>,
    ) -> anyhow::Result<()> {
        if let Some(response) = app.window.device.response(draw)
            && app.window.focus.is_none()
            && let (DeviceResponse::Cancel, true) = response
        {
            if layout.is_none() {
                draw.disable_cursor();
            }

            Self::set_layout(app, draw, layout);
            app.window
                .scene
                .asset
                .get_sound("data/audio/back.ogg")?
                .sound
                .play();
        }

        Ok(())
    }

    fn begin(
        _app: &mut App,
        _context: &mut Context,
        _draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[rustfmt::skip]
    fn setup(app: &mut App, draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>) -> anyhow::Result<()> {
        Self::layout_back(app, draw, Some(Layout::Main))?;

        let mut layout = None;

        Window::draw(app, draw, |app, draw| {
            app.window.point = Self::INITIAL_POINT;

            let y = draw.get_screen_height() as f32 - 200.0;

            app.window.scroll(draw, Vector2::new(768.0, y), |window, draw| {
                window.toggle(draw, "play tutorial", &mut app.user.tutorial)?;
                if window
                    .toggle(draw, "screen full", &mut app.user.video_full)?
                    .accept()
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
                    &mut app.user.video_field,
                    (60.0, 120.0), 1.0,
                )?;
                window.slider(
                    draw,
                    "screen shake",
                    &mut app.user.video_shake,
                    (0.0, 2.0),  0.1,
                )?;
                window.slider(
                    draw,
                    "screen tilt",
                    &mut app.user.video_tilt,
                    (0.0, 2.0,),  0.1,
                )?;
                /*
                if app.window
                    .slider(
                        draw,
                        "screen rate",
                        &mut app.user.screen_rate,
                        (30.0, 300.0), 1.0,
                    )?
                    .release()
                {
                    draw.set_target_fps(app.user.screen_rate as u32);
                }
                */

                window.slider(
                    draw,
                    "mouse speed",
                    &mut app.user.input_mouse_scale,
                    (0.0, 2.0), 0.1,
                )?;
                window.slider(
                    draw,
                    "sound volume",
                    &mut app.user.audio_sound,
                    (0.0, 1.0), 0.1
                )?;
                window.slider(
                    draw,
                    "music volume",
                    &mut app.user.audio_music,
                    (0.0, 1.0), 0.1
                )?;

                window.switch(draw, "glyph kind", &mut app.user.video_glyph, &[
                    GlyphKind::PlayStation,
                    GlyphKind::Xbox,
                    GlyphKind::Nintendo,
                ])?;
                window.switch(draw, "language", &mut app.user.video_locale, &[
                    LocaleKind::English,
                    LocaleKind::Spanish,
                ])?;

                window.action(draw, "move x+", &mut app.user.input_move_x_a)?;
                window.action(draw, "move x-", &mut app.user.input_move_x_b)?;
                window.action(draw, "move z+", &mut app.user.input_move_z_a)?;
                window.action(draw, "move z-", &mut app.user.input_move_z_b)?;
                window.action(draw, "jump", &mut app.user.input_jump)?;
                window.action(draw, "push", &mut app.user.input_push)?;
                window.action(draw, "pull", &mut app.user.input_pull)?;

                Ok(())
            })?;

            if app.window.button(draw, "return")?.accept() {
                layout = Some(Self::Main);
            };

            Ok(())
        })?;

        if let Some(layout) = layout {
            Self::set_layout(app, draw, Some(layout));
        }

        Ok(())
    }

    fn close(
        _app: &mut App,
        _draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

//================================================================

#[derive(Default, Debug, Clone, Copy)]
pub struct Widget {
    pub delta: f32,
    pub hover: bool,
    pub scroll: f32,
}

//================================================================

#[derive(PartialEq, Copy, Clone)]
pub enum Device {
    Board {
        lock: bool,
        index: usize,
    },
    Mouse {
        lock: bool,
    },
    Pad {
        lock: bool,
        index: usize,
        stick: f32,
    },
}

impl Default for Device {
    fn default() -> Self {
        Self::Mouse { lock: false }
    }
}

impl Device {
    const BOARD_DEVICE_RESPONSE: [(DeviceResponse, KeyboardKey); 4] = [
        (DeviceResponse::Accept, KeyboardKey::KEY_ENTER),
        (DeviceResponse::Cancel, KeyboardKey::KEY_ESCAPE),
        (DeviceResponse::SideA, KeyboardKey::KEY_LEFT),
        (DeviceResponse::SideB, KeyboardKey::KEY_RIGHT),
    ];
    const MOUSE_DEVICE_RESPONSE: [(DeviceResponse, MouseButton); 2] = [
        (DeviceResponse::Accept, MouseButton::MOUSE_BUTTON_LEFT),
        (DeviceResponse::Cancel, MouseButton::MOUSE_BUTTON_RIGHT),
    ];
    const PAD_DEVICE_RESPONSE: [(DeviceResponse, GamepadButton); 4] = [
        (
            DeviceResponse::Accept,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
        ),
        (
            DeviceResponse::Cancel,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
        ),
        (
            DeviceResponse::SideA,
            GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT,
        ),
        (
            DeviceResponse::SideB,
            GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
        ),
    ];

    fn lock(&self) -> bool {
        match self {
            Device::Board { lock, .. } => *lock,
            Device::Mouse { lock, .. } => *lock,
            Device::Pad { lock, .. } => *lock,
        }
    }

    fn escape(&self, handle: &RaylibHandle) -> bool {
        match self {
            Device::Board { .. } => handle.is_key_pressed(KeyboardKey::KEY_ESCAPE),
            Device::Mouse { .. } => handle.is_key_pressed(KeyboardKey::KEY_ESCAPE),
            Device::Pad { .. } => {
                handle.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT)
            }
        }
    }

    fn draw_glyph_response(
        window: &mut Window,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        point: Vector2,
        label: &str,
        device_response: DeviceResponse,
    ) -> anyhow::Result<()> {
        let point = Vector2::new(point.x, point.y + draw.get_screen_height() as f32 - 64.0);

        let mut draw_call = |window: &mut Window,
                             point: Vector2,
                             texture: &str,
                             label: &str|
         -> anyhow::Result<()> {
            let texture = window.scene.asset.get_texture(texture)?;

            draw.draw_texture_ex(texture, point, 0.0, 0.5, Color::WHITE);

            Window::font_draw(
                draw,
                window.font_label()?,
                label,
                point + Vector2::new(56.0, 8.0),
                Color::WHITE,
            );

            Ok(())
        };

        match window.device {
            Device::Board { .. } => {
                let key = match device_response {
                    DeviceResponse::Accept => "ENTER",
                    DeviceResponse::Cancel => "ESCAPE",
                    _ => "<- | ->",
                };

                Window::font_draw(
                    draw,
                    window.font_label()?,
                    &format!("[{key}] {label}"),
                    point + Vector2::new(0.0, 8.0),
                    Color::WHITE,
                );
            }
            Device::Mouse { .. } => match device_response {
                DeviceResponse::Accept => {
                    draw_call(window, point, "data/video/glyph/mouse/button_l.png", label)?;
                }
                DeviceResponse::Cancel => {
                    draw_call(window, point, "data/video/glyph/mouse/button_r.png", label)?;
                }
                _ => {
                    draw_call(window, point, "data/video/glyph/mouse/wheel_u.png", "")?;
                    draw_call(
                        window,
                        point + Vector2::new(40.0, 0.0),
                        "data/video/glyph/mouse/wheel_d.png",
                        label,
                    )?;
                }
            },
            Device::Pad { .. } => match device_response {
                DeviceResponse::Accept => {
                    draw_call(
                        window,
                        point,
                        "data/video/glyph/play_station/button_d.png",
                        label,
                    )?;
                }
                DeviceResponse::Cancel => {
                    draw_call(
                        window,
                        point,
                        "data/video/glyph/play_station/button_r.png",
                        label,
                    )?;
                }
                _ => {
                    draw_call(window, point, "data/video/glyph/play_station/pad_l.png", "")?;
                    draw_call(
                        window,
                        point + Vector2::new(56.0, 0.0),
                        "data/video/glyph/play_station/pad_r.png",
                        label,
                    )?;
                }
            },
        }

        Ok(())
    }

    fn is_board(&self) -> bool {
        matches!(self, Self::Board { .. })
    }

    fn is_mouse(&self) -> bool {
        matches!(self, Self::Mouse { .. })
    }

    fn is_pad(&self) -> bool {
        matches!(self, Self::Pad { .. })
    }

    fn poll_change(&self, handle: &mut RaylibHandle) -> Self {
        if self.lock() {
            return *self;
        }

        let mut new_device = None;

        if handle.get_key_pressed().is_some() {
            new_device = Some(Self::Board {
                lock: false,
                index: usize::default(),
            })
        }

        if matches!(self, Self::Board { .. }) {
            let delta = handle.get_mouse_delta();

            if delta.length() != 0.0 {
                new_device = Some(Self::Mouse { lock: false })
            }
        } else if Input::get_mouse_pressed(handle).is_some() {
            new_device = Some(Self::Mouse { lock: false })
        }

        if handle.get_gamepad_button_pressed().is_some() {
            new_device = Some(Self::Pad {
                lock: false,
                index: usize::default(),
                stick: f32::default(),
            })
        }

        if let Some(n_d) = new_device
            && std::mem::discriminant(&n_d) != std::mem::discriminant(self)
        {
            if matches!(n_d, Self::Mouse { .. }) {
                handle.enable_cursor();
            } else {
                handle.disable_cursor();
            }

            n_d
        } else {
            *self
        }
    }

    fn hover(&self, handle: &RaylibHandle, widget_index: usize, widget_shape: Rectangle) -> bool {
        match self {
            Device::Board { index, .. } => *index == widget_index,
            Device::Mouse { .. } => {
                widget_shape.check_collision_point_rec(handle.get_mouse_position())
            }
            Device::Pad { index, .. } => *index == widget_index,
        }
    }

    fn update_index(&mut self, handle: &mut RaylibHandle, bound: usize) {
        match self {
            Device::Board { index, .. } => {
                if handle.is_key_pressed(KeyboardKey::KEY_UP)
                    || handle.is_key_pressed_repeat(KeyboardKey::KEY_UP)
                {
                    if *index > 0 {
                        *index -= 1;
                    } else {
                        *index = bound - 1;
                    }
                }

                if handle.is_key_pressed(KeyboardKey::KEY_DOWN)
                    || handle.is_key_pressed_repeat(KeyboardKey::KEY_DOWN)
                {
                    *index += 1;
                }

                *index %= bound;
            }
            Device::Mouse { .. } => {}
            Device::Pad { index, stick, .. } => {
                let stick_state =
                    handle.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);

                if stick_state < -0.1 && *stick >= -0.1 {
                    if *index > 0 {
                        *index -= 1;
                    } else {
                        *index = bound - 1;
                    }
                }

                if stick_state > 0.1 && *stick <= 0.1 {
                    *index += 1;
                }

                *stick = stick_state;

                if handle.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
                    if *index > 0 {
                        *index -= 1;
                    } else {
                        *index = bound - 1;
                    }
                }

                if handle.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN)
                {
                    *index += 1;
                }

                *index %= bound;
            }
        }
    }

    fn response(&self, handle: &RaylibHandle) -> Option<(DeviceResponse, bool)> {
        match self {
            Device::Board { .. } => {
                for (response, key) in Self::BOARD_DEVICE_RESPONSE {
                    if handle.is_key_pressed(key) || handle.is_key_pressed_repeat(key) {
                        return Some((response, true));
                    }

                    if handle.is_key_released(key) {
                        return Some((response, false));
                    }
                }

                None
            }
            Device::Mouse { .. } => {
                for (response, key) in Self::MOUSE_DEVICE_RESPONSE {
                    if handle.is_mouse_button_pressed(key) {
                        return Some((response, true));
                    }

                    if handle.is_mouse_button_released(key) {
                        return Some((response, false));
                    }
                }

                let delta = handle.get_mouse_wheel_move();

                if delta > 0.0 {
                    return Some((DeviceResponse::SideB, true));
                } else if delta < 0.0 {
                    return Some((DeviceResponse::SideA, true));
                }

                None
            }
            Device::Pad { .. } => {
                for (response, key) in Self::PAD_DEVICE_RESPONSE {
                    if handle.is_gamepad_button_pressed(0, key) {
                        return Some((response, true));
                    }

                    if handle.is_gamepad_button_released(0, key) {
                        return Some((response, false));
                    }

                    // TO-DO return SideA/SideB with left-stick?
                }

                None
            }
        }
    }
}

//================================================================

#[derive(PartialEq)]
pub enum DeviceResponse {
    Accept,
    Cancel,
    SideA,
    SideB,
}

//================================================================

#[derive(Default)]
pub struct Response {
    pub hover: bool,
    pub focus: bool,
    pub device: Option<(DeviceResponse, bool)>,
    pub widget: Widget,
}

impl Response {
    pub fn accept(&self) -> bool {
        matches!(self.device, Some((DeviceResponse::Accept, true)))
    }

    pub fn cancel(&self) -> bool {
        matches!(self.device, Some((DeviceResponse::Cancel, true)))
    }

    pub fn side_a(&self) -> bool {
        matches!(self.device, Some((DeviceResponse::SideA, true)))
    }

    pub fn side_b(&self) -> bool {
        matches!(self.device, Some((DeviceResponse::SideB, true)))
    }

    fn new_from_window(handle: &RaylibHandle, window: &mut Window, shape: Rectangle) -> Self {
        let focus = if let Some(focus) = window.focus
            && focus == window.index
        {
            true
        } else {
            false
        };
        let hover = window.device.hover(handle, window.index, shape);
        let hover = (hover && window.focus.is_none()) || focus;
        let device = if hover {
            window.device.response(handle)
        } else {
            None
        };

        let widget = window.widget.entry(window.index).or_default();

        if hover {
            if !widget.hover {
                widget.hover = true;
            }
        } else if widget.hover {
            widget.hover = false;
        }

        Self {
            hover,
            focus,
            device,
            widget: *widget,
        }
    }
}

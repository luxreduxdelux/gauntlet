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
use serde::{Deserialize, Serialize};
use std::fmt::Display;

//================================================================

#[derive(Serialize, Deserialize)]
pub struct User {
    pub developer: bool,
    pub tutorial: bool,
    pub video_glyph: GlyphKind,
    pub video_locale: LocaleKind,
    pub video_full: bool,
    pub video_field: f32,
    pub video_shake: f32,
    pub video_scale: f32,
    pub video_tilt: f32,
    pub video_rate: f32,
    pub video_brightness: f32,
    pub video_contrast: f32,
    pub video_cross: bool,
    pub audio_sound: f32,
    pub audio_music: f32,
    pub input_move_x_a: Input,
    pub input_move_x_b: Input,
    pub input_move_z_a: Input,
    pub input_move_z_b: Input,
    pub input_jump: Input,
    pub input_push: Input,
    pub input_pull: Input,
    pub input_info: Input,
    pub input_mouse_scale: f32,
    #[serde(skip)]
    pub debug_draw_physical: bool,
    #[serde(skip)]
    pub debug_draw_entity: bool,
    #[serde(skip)]
    pub debug_frame_rate: bool,
    #[serde(skip)]
    pub debug_light_edit: bool,
}

impl User {
    const PATH_FILE: &'static str = "user.json";
}

impl Default for User {
    fn default() -> Self {
        if let Ok(file) = std::fs::read_to_string(Self::PATH_FILE)
            && let Ok(data) = serde_json::from_str(&file)
        {
            data
        } else {
            Self {
                developer: true,
                tutorial: true,
                video_glyph: GlyphKind::PlayStation,
                video_locale: LocaleKind::English,
                video_full: false,
                video_field: 90.0,
                video_shake: 1.0,
                video_scale: 1.0,
                video_tilt: 1.0,
                video_rate: 60.0,
                video_brightness: 1.0,
                video_contrast: 1.0,
                video_cross: true,
                audio_sound: 1.0,
                audio_music: 1.0,
                input_move_x_a: Input::new_board(KeyboardKey::KEY_W),
                input_move_x_b: Input::new_board(KeyboardKey::KEY_S),
                input_move_z_a: Input::new_board(KeyboardKey::KEY_A),
                input_move_z_b: Input::new_board(KeyboardKey::KEY_D),
                input_jump: Input::new_board(KeyboardKey::KEY_SPACE),
                input_push: Input::new_mouse(MouseButton::MOUSE_BUTTON_LEFT),
                input_pull: Input::new_mouse(MouseButton::MOUSE_BUTTON_RIGHT),
                input_info: Input::new_board(KeyboardKey::KEY_TAB),
                input_mouse_scale: 1.0,
                debug_draw_physical: false,
                debug_draw_entity: false,
                debug_frame_rate: false,
                debug_light_edit: false,
            }
        }
    }
}

impl Drop for User {
    fn drop(&mut self) {
        let data = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(Self::PATH_FILE, data).unwrap();
    }
}

//================================================================

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Input {
    Board {
        key: u32,
        #[serde(skip)]
        press: bool,
        #[serde(skip)]
        release: bool,
    },
    Mouse {
        key: u32,
        #[serde(skip)]
        press: bool,
        #[serde(skip)]
        release: bool,
    },
    Pad {
        key: u32,
        #[serde(skip)]
        press: bool,
        #[serde(skip)]
        release: bool,
    },
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Board { key, .. } => {
                let key = Self::to_board(*key);

                match key {
                    KeyboardKey::KEY_NULL => "Null",
                    KeyboardKey::KEY_APOSTROPHE => "'",
                    KeyboardKey::KEY_COMMA => ",",
                    KeyboardKey::KEY_MINUS => "-",
                    KeyboardKey::KEY_PERIOD => ".",
                    KeyboardKey::KEY_SLASH => "/",
                    KeyboardKey::KEY_ZERO => "0",
                    KeyboardKey::KEY_ONE => "1",
                    KeyboardKey::KEY_TWO => "2",
                    KeyboardKey::KEY_THREE => "3",
                    KeyboardKey::KEY_FOUR => "4",
                    KeyboardKey::KEY_FIVE => "5",
                    KeyboardKey::KEY_SIX => "6",
                    KeyboardKey::KEY_SEVEN => "7",
                    KeyboardKey::KEY_EIGHT => "8",
                    KeyboardKey::KEY_NINE => "9",
                    KeyboardKey::KEY_SEMICOLON => ";",
                    KeyboardKey::KEY_EQUAL => "=",
                    KeyboardKey::KEY_A => "A",
                    KeyboardKey::KEY_B => "B",
                    KeyboardKey::KEY_C => "C",
                    KeyboardKey::KEY_D => "D",
                    KeyboardKey::KEY_E => "E",
                    KeyboardKey::KEY_F => "F",
                    KeyboardKey::KEY_G => "G",
                    KeyboardKey::KEY_H => "H",
                    KeyboardKey::KEY_I => "I",
                    KeyboardKey::KEY_J => "J",
                    KeyboardKey::KEY_K => "K",
                    KeyboardKey::KEY_L => "L",
                    KeyboardKey::KEY_M => "M",
                    KeyboardKey::KEY_N => "N",
                    KeyboardKey::KEY_O => "O",
                    KeyboardKey::KEY_P => "P",
                    KeyboardKey::KEY_Q => "Q",
                    KeyboardKey::KEY_R => "R",
                    KeyboardKey::KEY_S => "S",
                    KeyboardKey::KEY_T => "T",
                    KeyboardKey::KEY_U => "U",
                    KeyboardKey::KEY_V => "V",
                    KeyboardKey::KEY_W => "W",
                    KeyboardKey::KEY_X => "X",
                    KeyboardKey::KEY_Y => "Y",
                    KeyboardKey::KEY_Z => "Z",
                    KeyboardKey::KEY_LEFT_BRACKET => "{",
                    KeyboardKey::KEY_BACKSLASH => "\\",
                    KeyboardKey::KEY_RIGHT_BRACKET => "}",
                    KeyboardKey::KEY_GRAVE => "`",
                    KeyboardKey::KEY_SPACE => "Space",
                    KeyboardKey::KEY_ESCAPE => "Escape",
                    KeyboardKey::KEY_ENTER => "Enter",
                    KeyboardKey::KEY_TAB => "Tabulation",
                    KeyboardKey::KEY_BACKSPACE => "Backspace",
                    KeyboardKey::KEY_INSERT => "Insert",
                    KeyboardKey::KEY_DELETE => "Delete",
                    KeyboardKey::KEY_RIGHT => "Right",
                    KeyboardKey::KEY_LEFT => "Left",
                    KeyboardKey::KEY_DOWN => "Down",
                    KeyboardKey::KEY_UP => "Up",
                    KeyboardKey::KEY_PAGE_UP => "Page Up",
                    KeyboardKey::KEY_PAGE_DOWN => "Page Down",
                    KeyboardKey::KEY_HOME => "Home",
                    KeyboardKey::KEY_END => "End",
                    KeyboardKey::KEY_CAPS_LOCK => "Case Lock",
                    KeyboardKey::KEY_SCROLL_LOCK => "Scroll Lock",
                    KeyboardKey::KEY_NUM_LOCK => "Number Lock",
                    KeyboardKey::KEY_PRINT_SCREEN => "Print Screen",
                    KeyboardKey::KEY_PAUSE => "Pause",
                    KeyboardKey::KEY_F1 => "F1",
                    KeyboardKey::KEY_F2 => "F2",
                    KeyboardKey::KEY_F3 => "F3",
                    KeyboardKey::KEY_F4 => "F4",
                    KeyboardKey::KEY_F5 => "F5",
                    KeyboardKey::KEY_F6 => "F6",
                    KeyboardKey::KEY_F7 => "F7",
                    KeyboardKey::KEY_F8 => "F8",
                    KeyboardKey::KEY_F9 => "F9",
                    KeyboardKey::KEY_F10 => "F10",
                    KeyboardKey::KEY_F11 => "F11",
                    KeyboardKey::KEY_F12 => "F12",
                    KeyboardKey::KEY_LEFT_SHIFT => "L. Shift",
                    KeyboardKey::KEY_LEFT_CONTROL => "L. Control",
                    KeyboardKey::KEY_LEFT_ALT => "L. Alternate",
                    KeyboardKey::KEY_LEFT_SUPER => "L. Super",
                    KeyboardKey::KEY_RIGHT_SHIFT => "R. Shift",
                    KeyboardKey::KEY_RIGHT_CONTROL => "R. Control",
                    KeyboardKey::KEY_RIGHT_ALT => "R. Alternate",
                    KeyboardKey::KEY_RIGHT_SUPER => "R. Super",
                    KeyboardKey::KEY_KB_MENU => "Menu",
                    KeyboardKey::KEY_KP_0 => "Pad 0",
                    KeyboardKey::KEY_KP_1 => "Pad 1",
                    KeyboardKey::KEY_KP_2 => "Pad 2",
                    KeyboardKey::KEY_KP_3 => "Pad 3",
                    KeyboardKey::KEY_KP_4 => "Pad 4",
                    KeyboardKey::KEY_KP_5 => "Pad 5",
                    KeyboardKey::KEY_KP_6 => "Pad 6",
                    KeyboardKey::KEY_KP_7 => "Pad 7",
                    KeyboardKey::KEY_KP_8 => "Pad 8",
                    KeyboardKey::KEY_KP_9 => "Pad 9",
                    KeyboardKey::KEY_KP_DECIMAL => "Pad .",
                    KeyboardKey::KEY_KP_DIVIDE => "Pad /",
                    KeyboardKey::KEY_KP_MULTIPLY => "Pad *",
                    KeyboardKey::KEY_KP_SUBTRACT => "Pad -",
                    KeyboardKey::KEY_KP_ADD => "Pad +",
                    KeyboardKey::KEY_KP_ENTER => "Pad Enter",
                    KeyboardKey::KEY_KP_EQUAL => "Pad =",
                    KeyboardKey::KEY_BACK => "Back",
                    KeyboardKey::KEY_MENU => "Menu",
                    KeyboardKey::KEY_VOLUME_UP => "Volume Up",
                    KeyboardKey::KEY_VOLUME_DOWN => "Volume Down",
                }
            }
            _ => "",
        };

        f.write_str(string)
    }
}

impl Input {
    pub fn new_board(key: KeyboardKey) -> Input {
        Input::Board {
            key: key as u32,
            press: false,
            release: false,
        }
    }

    pub fn new_mouse(key: MouseButton) -> Input {
        Input::Mouse {
            key: key as u32,
            press: false,
            release: false,
        }
    }

    pub fn new_pad(key: GamepadButton) -> Input {
        Input::Pad {
            key: key as u32,
            press: false,
            release: false,
        }
    }

    pub fn get_mouse_pressed(handle: &RaylibHandle) -> Option<MouseButton> {
        let list = [
            MouseButton::MOUSE_BUTTON_LEFT,
            MouseButton::MOUSE_BUTTON_RIGHT,
            MouseButton::MOUSE_BUTTON_MIDDLE,
            MouseButton::MOUSE_BUTTON_SIDE,
            MouseButton::MOUSE_BUTTON_EXTRA,
            MouseButton::MOUSE_BUTTON_FORWARD,
            MouseButton::MOUSE_BUTTON_BACK,
        ];

        list.into_iter()
            .find(|&button| handle.is_mouse_button_pressed(button))
    }

    pub fn get_gamepad_button_pressed(handle: &RaylibHandle, index: i32) -> Option<GamepadButton> {
        let list = [
            GamepadButton::GAMEPAD_BUTTON_UNKNOWN,
            GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP,
            GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
            GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN,
            GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT,
            GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1,
            GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_2,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2,
            GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT,
            GamepadButton::GAMEPAD_BUTTON_MIDDLE,
            GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT,
            GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB,
            GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB,
        ];

        list.into_iter()
            .find(|&button| handle.is_gamepad_button_pressed(index, button))
    }

    pub fn to_board(value: u32) -> KeyboardKey {
        unsafe { std::mem::transmute(value) }
    }

    pub fn to_mouse(value: u32) -> MouseButton {
        unsafe { std::mem::transmute(value) }
    }

    pub fn to_pad(value: u32) -> GamepadButton {
        unsafe { std::mem::transmute(value) }
    }

    pub fn poll(&mut self, handle: &RaylibHandle) {
        match self {
            Input::Board {
                key,
                press,
                release,
            } => {
                if handle.is_key_pressed(Self::to_board(*key)) {
                    *press = true;
                }

                if handle.is_key_released(Self::to_board(*key)) {
                    *release = true;
                }
            }
            Input::Mouse {
                key,
                press,
                release,
            } => {
                if handle.is_mouse_button_pressed(Self::to_mouse(*key)) {
                    *press = true;
                }

                if handle.is_mouse_button_released(Self::to_mouse(*key)) {
                    *release = true;
                }
            }
            Input::Pad {
                key,
                press,
                release,
            } => {
                if handle.is_gamepad_button_pressed(0, Self::to_pad(*key)) {
                    *press = true;
                }

                if handle.is_gamepad_button_released(0, Self::to_pad(*key)) {
                    *release = true;
                }
            }
        }
    }

    pub fn wipe(&mut self) {
        match self {
            Input::Board { press, release, .. } => {
                *press = false;
                *release = false;
            }
            Input::Mouse { press, release, .. } => {
                *press = false;
                *release = false;
            }
            Input::Pad { press, release, .. } => {
                *press = false;
                *release = false;
            }
        }
    }

    pub fn up(&self, handle: &RaylibHandle) -> bool {
        match self {
            Input::Board { key, .. } => handle.is_key_up(Self::to_board(*key)),
            Input::Mouse { key, .. } => handle.is_mouse_button_up(Self::to_mouse(*key)),
            Input::Pad { key, .. } => handle.is_gamepad_button_up(0, Self::to_pad(*key)),
        }
    }

    pub fn down(&self, handle: &RaylibHandle) -> bool {
        match self {
            Input::Board { key, .. } => handle.is_key_down(Self::to_board(*key)),
            Input::Mouse { key, .. } => handle.is_mouse_button_down(Self::to_mouse(*key)),
            Input::Pad { key, .. } => handle.is_gamepad_button_down(0, Self::to_pad(*key)),
        }
    }

    pub fn press(&self) -> bool {
        match self {
            Input::Board { press, .. } => *press,
            Input::Mouse { press, .. } => *press,
            Input::Pad { press, .. } => *press,
        }
    }

    pub fn release(&mut self) -> bool {
        match self {
            Input::Board { release, .. } => *release,
            Input::Mouse { release, .. } => *release,
            Input::Pad { release, .. } => *release,
        }
    }
}

//================================================================

#[derive(Default, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum GlyphKind {
    #[default]
    PlayStation,
    Xbox,
    Nintendo,
}

impl GlyphKind {
    pub fn folder_name(&self) -> &str {
        match self {
            GlyphKind::PlayStation => "play_station",
            GlyphKind::Xbox => "xbox",
            GlyphKind::Nintendo => "nintendo",
        }
    }
}

impl Display for GlyphKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::PlayStation => "PlayStation",
            Self::Xbox => "Xbox",
            Self::Nintendo => "Nintendo",
        };

        f.write_str(string)
    }
}

//================================================================

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum LocaleKind {
    English,
    Spanish,
}

impl Display for LocaleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::English => "English",
            Self::Spanish => "Spanish",
        };

        f.write_str(string)
    }
}

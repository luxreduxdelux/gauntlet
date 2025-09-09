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

#[derive(Serialize, Deserialize)]
pub struct Setting {
    pub screen_full: bool,
    pub screen_field: f32,
    pub screen_shake: f32,
    pub screen_tilt: f32,
    pub screen_rate: f32,
    pub mouse_speed: f32,
    pub volume_sound: f32,
    pub volume_music: f32,
    pub move_x_a: Input,
    pub move_x_b: Input,
    pub move_z_a: Input,
    pub move_z_b: Input,
    pub jump: Input,
    pub duck: Input,
    pub fire_a: Input,
    pub fire_b: Input,
}

impl Setting {
    const PATH_FILE: &'static str = "setting.json";
}

impl Default for Setting {
    fn default() -> Self {
        if let Ok(file) = std::fs::read_to_string(Self::PATH_FILE)
            && let Ok(data) = serde_json::from_str(&file)
        {
            data
        } else {
            Self {
                screen_full: false,
                screen_field: 90.0,
                screen_shake: 1.0,
                screen_tilt: 1.0,
                screen_rate: 60.0,
                mouse_speed: 1.0,
                volume_sound: 1.0,
                volume_music: 1.0,
                move_x_a: Input::new_board(KeyboardKey::KEY_W as u32),
                move_x_b: Input::new_board(KeyboardKey::KEY_S as u32),
                move_z_a: Input::new_board(KeyboardKey::KEY_A as u32),
                move_z_b: Input::new_board(KeyboardKey::KEY_D as u32),
                jump: Input::new_board(KeyboardKey::KEY_SPACE as u32),
                duck: Input::new_mouse(MouseButton::MOUSE_BUTTON_EXTRA as u32),
                fire_a: Input::new_mouse(MouseButton::MOUSE_BUTTON_LEFT as u32),
                fire_b: Input::new_mouse(MouseButton::MOUSE_BUTTON_RIGHT as u32),
            }
        }
    }
}

impl Drop for Setting {
    fn drop(&mut self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            std::fs::write(Self::PATH_FILE, data);
        }
    }
}

// there is a build failure on raylib-rs 5.5.1 when using the "serde" feature
// that supposedly does implement Serialize for KeyboardKey/etc; but it does not
// compile successfully as of 5/9/2025.
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
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Board { key, .. } => {
                let key = Self::to_board(*key);

                match key {
                    KeyboardKey::KEY_NULL => "null",
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
                    KeyboardKey::KEY_A => "a",
                    KeyboardKey::KEY_B => "b",
                    KeyboardKey::KEY_C => "c",
                    KeyboardKey::KEY_D => "d",
                    KeyboardKey::KEY_E => "e",
                    KeyboardKey::KEY_F => "f",
                    KeyboardKey::KEY_G => "g",
                    KeyboardKey::KEY_H => "h",
                    KeyboardKey::KEY_I => "i",
                    KeyboardKey::KEY_J => "j",
                    KeyboardKey::KEY_K => "k",
                    KeyboardKey::KEY_L => "l",
                    KeyboardKey::KEY_M => "m",
                    KeyboardKey::KEY_N => "n",
                    KeyboardKey::KEY_O => "o",
                    KeyboardKey::KEY_P => "p",
                    KeyboardKey::KEY_Q => "q",
                    KeyboardKey::KEY_R => "r",
                    KeyboardKey::KEY_S => "s",
                    KeyboardKey::KEY_T => "t",
                    KeyboardKey::KEY_U => "u",
                    KeyboardKey::KEY_V => "v",
                    KeyboardKey::KEY_W => "w",
                    KeyboardKey::KEY_X => "x",
                    KeyboardKey::KEY_Y => "y",
                    KeyboardKey::KEY_Z => "z",
                    KeyboardKey::KEY_LEFT_BRACKET => "{",
                    KeyboardKey::KEY_BACKSLASH => "\\",
                    KeyboardKey::KEY_RIGHT_BRACKET => "}",
                    KeyboardKey::KEY_GRAVE => "`",
                    KeyboardKey::KEY_SPACE => "space",
                    KeyboardKey::KEY_ESCAPE => "escape",
                    KeyboardKey::KEY_ENTER => "enter",
                    KeyboardKey::KEY_TAB => "tabulation",
                    KeyboardKey::KEY_BACKSPACE => "backspace",
                    KeyboardKey::KEY_INSERT => "insert",
                    KeyboardKey::KEY_DELETE => "delete",
                    KeyboardKey::KEY_RIGHT => "right",
                    KeyboardKey::KEY_LEFT => "left",
                    KeyboardKey::KEY_DOWN => "down",
                    KeyboardKey::KEY_UP => "up",
                    KeyboardKey::KEY_PAGE_UP => "page up",
                    KeyboardKey::KEY_PAGE_DOWN => "page down",
                    KeyboardKey::KEY_HOME => "home",
                    KeyboardKey::KEY_END => "end",
                    KeyboardKey::KEY_CAPS_LOCK => "case lock",
                    KeyboardKey::KEY_SCROLL_LOCK => "scroll lock",
                    KeyboardKey::KEY_NUM_LOCK => "number lock",
                    KeyboardKey::KEY_PRINT_SCREEN => "print screen",
                    KeyboardKey::KEY_PAUSE => "pause",
                    KeyboardKey::KEY_F1 => "f1",
                    KeyboardKey::KEY_F2 => "f2",
                    KeyboardKey::KEY_F3 => "f3",
                    KeyboardKey::KEY_F4 => "f4",
                    KeyboardKey::KEY_F5 => "f5",
                    KeyboardKey::KEY_F6 => "f6",
                    KeyboardKey::KEY_F7 => "f7",
                    KeyboardKey::KEY_F8 => "f8",
                    KeyboardKey::KEY_F9 => "f9",
                    KeyboardKey::KEY_F10 => "f10",
                    KeyboardKey::KEY_F11 => "f11",
                    KeyboardKey::KEY_F12 => "f12",
                    KeyboardKey::KEY_LEFT_SHIFT => "l. shift",
                    KeyboardKey::KEY_LEFT_CONTROL => "l. control",
                    KeyboardKey::KEY_LEFT_ALT => "l. alternate",
                    KeyboardKey::KEY_LEFT_SUPER => "l. super",
                    KeyboardKey::KEY_RIGHT_SHIFT => "r. shift",
                    KeyboardKey::KEY_RIGHT_CONTROL => "r. control",
                    KeyboardKey::KEY_RIGHT_ALT => "r. alternate",
                    KeyboardKey::KEY_RIGHT_SUPER => "r. super",
                    KeyboardKey::KEY_KB_MENU => "menu",
                    KeyboardKey::KEY_KP_0 => "pad 0",
                    KeyboardKey::KEY_KP_1 => "pad 1",
                    KeyboardKey::KEY_KP_2 => "pad 2",
                    KeyboardKey::KEY_KP_3 => "pad 3",
                    KeyboardKey::KEY_KP_4 => "pad 4",
                    KeyboardKey::KEY_KP_5 => "pad 5",
                    KeyboardKey::KEY_KP_6 => "pad 6",
                    KeyboardKey::KEY_KP_7 => "pad 7",
                    KeyboardKey::KEY_KP_8 => "pad 8",
                    KeyboardKey::KEY_KP_9 => "pad 9",
                    KeyboardKey::KEY_KP_DECIMAL => "pad .",
                    KeyboardKey::KEY_KP_DIVIDE => "pad /",
                    KeyboardKey::KEY_KP_MULTIPLY => "pad *",
                    KeyboardKey::KEY_KP_SUBTRACT => "pad -",
                    KeyboardKey::KEY_KP_ADD => "pad +",
                    KeyboardKey::KEY_KP_ENTER => "pad enter",
                    KeyboardKey::KEY_KP_EQUAL => "pad =",
                    KeyboardKey::KEY_BACK => "back",
                    KeyboardKey::KEY_MENU => "menu",
                    KeyboardKey::KEY_VOLUME_UP => "volume up",
                    KeyboardKey::KEY_VOLUME_DOWN => "volume down",
                }
            }
            Self::Mouse { key, .. } => {
                let key = Self::to_mouse(*key);

                match key {
                    MouseButton::MOUSE_BUTTON_LEFT => "mouse l.",
                    MouseButton::MOUSE_BUTTON_RIGHT => "mouse r.",
                    MouseButton::MOUSE_BUTTON_MIDDLE => "mouse middle",
                    MouseButton::MOUSE_BUTTON_SIDE => "mouse side",
                    MouseButton::MOUSE_BUTTON_EXTRA => "mouse extra",
                    MouseButton::MOUSE_BUTTON_FORWARD => "mouse forward",
                    MouseButton::MOUSE_BUTTON_BACK => "mouse back",
                }
            }
        };

        f.write_str(string)
    }
}

impl Input {
    pub fn new_board(key: u32) -> Input {
        Input::Board {
            key,
            press: false,
            release: false,
        }
    }

    pub fn new_mouse(key: u32) -> Input {
        Input::Mouse {
            key,
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

        for button in list {
            if handle.is_mouse_button_pressed(button) {
                return Some(button);
            }
        }

        None
    }

    fn to_board(value: u32) -> KeyboardKey {
        unsafe { std::mem::transmute(value) }
    }

    fn to_mouse(value: u32) -> MouseButton {
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
        }
    }

    pub fn up(&self, handle: &RaylibHandle) -> bool {
        match self {
            Input::Board { key, .. } => handle.is_key_up(Self::to_board(*key)),
            Input::Mouse { key, .. } => handle.is_mouse_button_up(Self::to_mouse(*key)),
        }
    }

    pub fn down(&self, handle: &RaylibHandle) -> bool {
        match self {
            Input::Board { key, .. } => handle.is_key_down(Self::to_board(*key)),
            Input::Mouse { key, .. } => handle.is_mouse_button_down(Self::to_mouse(*key)),
        }
    }

    pub fn press(&self) -> bool {
        match self {
            Input::Board { press, .. } => *press,
            Input::Mouse { press, .. } => *press,
        }
    }

    pub fn release(&mut self) -> bool {
        match self {
            Input::Board { release, .. } => *release,
            Input::Mouse { release, .. } => *release,
        }
    }
}

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

#[derive(Serialize, Deserialize)]
pub struct Setting {
    pub screen_field: u32,
    pub screen_shake: f32,
    pub screen_tilt: f32,
    pub screen_full: bool,
    pub screen_sync: bool,
    pub screen_rate: u32,
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
                screen_field: 90,
                screen_shake: 1.0,
                screen_tilt: 1.0,
                screen_full: false,
                screen_sync: false,
                screen_rate: 60,
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

impl Input {
    fn new_board(key: u32) -> Input {
        Input::Board {
            key,
            press: false,
            release: false,
        }
    }

    fn new_mouse(key: u32) -> Input {
        Input::Mouse {
            key,
            press: false,
            release: false,
        }
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

/* ui/mod.rs
 *
 * Copyright 2023 Anton Klimanov
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * 	http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::emulator;

pub mod window;
pub mod code_view_pane;
pub mod debug_pane;
pub mod line_builder_pane;
pub mod stack_view;
pub mod memory_view;
pub mod command_view;

pub trait PlainCommandRepr {
    fn from_command(_: &emulator::Command) -> Self;
    fn get_words(&self) -> [u8; 10];
}

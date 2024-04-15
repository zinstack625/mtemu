/* utils.rs
 *
 * Copyright 2024 Anton Klimanov
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

use std::{sync::Arc, cell::RefCell};

use crate::emulator;
use crate::emulator::MT1804Emulator;

pub type EmulatorStored = Arc<RefCell<Option<emulator::OriginalImplementation>>>;

pub fn get_commands(emul: EmulatorStored) -> Vec<emulator::Command> {
    let Some(ref emul) = *emul.as_ref().borrow() else { return Vec::new() };
    let mut commands = Vec::<emulator::Command>::with_capacity(emul.commands_count());
    for i in 0..emul.commands_count() {
        commands.push(emul.get_command(i));
    }
    commands
}

pub fn get_calls(emul: EmulatorStored) -> Vec<emulator::Call> {
    let Some(ref emul) = *emul.as_ref().borrow() else { return Vec::new() };
    let mut calls = Vec::<emulator::Call>::with_capacity(emul.call_count());
    for i in 0..emul.call_count() {
        calls.push(emul.get_call(i));
    }
    calls
}

pub fn get_libcalls(emul: EmulatorStored) -> Vec<emulator::LibCall> {
    let Some(ref emul) = *emul.as_ref().borrow() else { return Vec::new() };
    emul.get_map_calls()
}

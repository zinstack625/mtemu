/* emulator/mod.rs
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


use libc;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Command {
    isOffset: i32,
    number_: i32,
    words: *mut i32,
    words_len: libc::size_t,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            isOffset: i32::default(),
            number_: i32::default(),
            words: std::ptr::null_mut(),
            words_len: usize::default(),
        }
    }
}

impl Command {
    pub fn new(pos: i32, words: &mut [i32]) -> Command {
        Command {
            isOffset: 0,
            number_: pos,
            words: words.as_mut_ptr(),
            words_len: words.len(),
        }
    }
    pub fn get_num(&self) -> usize {
        self.number_ as usize
    }
    pub fn get_words(&self) -> Option<Vec<u32>> {
        if self.words == std::ptr::null_mut() {
            return None;
        }
        let words = std::ptr::slice_from_raw_parts(self.words, self.words_len);
        unsafe { Some((&*words).iter().map(|elem| { *elem as u32 }).collect()) }
    }
}

#[repr(C)]
pub struct Call {
    address_: i32,
    comment_: *mut libc::c_char,
}

#[repr(C)]
pub struct Emulator {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "engine")]
extern "C" {
    fn create_emulator() -> *mut Emulator;
    fn destroy_emulator(_: *mut Emulator);
    fn emulator_reset(_: *mut Emulator);
    fn emulator_get_command(_: *mut Emulator, _: i32) -> Command;
    fn emulator_add_command(_: *mut Emulator, _: i32, _: Command) -> u8;
    fn emulator_update_command(_: *mut Emulator, _: i32, _: Command) -> u8;
    fn emulator_last_command(_: *mut Emulator) -> Command;
    fn emulator_remove_command(_: *mut Emulator, _: i32) -> *mut i32;
    fn emulator_commands_count(_: *mut Emulator) -> i32;
    fn emulator_executed_command(_: *mut Emulator) -> Command;
    fn emulator_exec_one(_: *mut Emulator) -> usize;
    fn emulator_exec_one_call(_: *mut Emulator) -> usize;
    fn emulator_exec_all(_: *mut Emulator) -> usize;
    fn emulator_get_next_index(_: *mut Emulator) -> i32;
    fn emulator_get_prev_index(_: *mut Emulator) -> i32;
    fn emulator_get_call_index(_: *mut Emulator) -> i32;
    fn emulator_get_pc(_: *mut Emulator) -> i32;
    fn emulator_set_pc(_: *mut Emulator, _: i32) -> i32;
    fn emulator_get_sp(_: *mut Emulator) -> i32;
    fn emulator_set_sp(_: *mut Emulator, _: i32) -> i32;
    fn emulator_get_stack_value(_: *mut Emulator, _: i32) -> i32;
    fn emulator_get_stack_length(_: *mut Emulator) -> i32;
    fn emulator_get_mp(_: *mut Emulator) -> i32;
    fn emulator_get_port(_: *mut Emulator) -> i32;
    fn emulator_get_mem_value(_: *mut Emulator) -> i32;
    fn emulator_get_reg_q(_: *mut Emulator) -> i32;
    fn emulator_get_reg_value(_: *mut Emulator, _: i32) -> i32;
    fn emulator_get_f(_: *mut Emulator) -> i32;
    fn emulator_get_y(_: *mut Emulator) -> i32;
    fn emulator_get_prev_reg_q(_: *mut Emulator) -> i32;
    fn emulator_get_prev_reg_a(_: *mut Emulator) -> i32;
    fn emulator_get_prev_reg_b(_: *mut Emulator) -> i32;
    fn emulator_get_r(_: *mut Emulator) -> i32;
    fn emulator_get_s(_: *mut Emulator) -> i32;
    fn emulator_get_z(_: *mut Emulator) -> i32;
    fn emulator_get_f3(_: *mut Emulator) -> i32;
    fn emulator_get_c4(_: *mut Emulator) -> i32;
    fn emulator_get_ovr(_: *mut Emulator) -> i32;
    fn emulator_get_g(_: *mut Emulator) -> i32;
    fn emulator_get_p(_: *mut Emulator) -> i32;
    fn emulator_add_call(_: *mut Emulator, _: i32, _: Call);
    fn emulator_get_call(_: *mut Emulator, _: i32) -> Call;
    fn emulator_update_call(_: *mut Emulator, _: i32, _: Call);
    fn emulator_remove_call(_: *mut Emulator, _: i32);
    fn emulator_calls_count(_: *mut Emulator) -> i32;
    fn emulator_last_call(_: *mut Emulator) -> Call;
    fn emulator_open_raw(_: *mut Emulator, _: *mut u8, _: libc::size_t) -> u8;
    fn emulator_export_raw(_: *mut Emulator, _: *mut *mut u8, _: *mut libc::size_t);
    fn command_get_name(_: *mut Emulator, _: Command) -> *mut libc::c_char;
    fn command_get_jump_name(_: *mut Emulator, _: Command) -> *mut libc::c_char;
    fn free_obj(_: *mut libc::c_void);
}

pub struct Port {}

trait SerialPort {}

pub trait MT1804Emulator {
    fn reset(&mut self);
    fn get_command(&self, index: usize) -> Command;
    fn add_command(&mut self, index: usize, cmd: &Command);
    fn update_command(&mut self, index: usize, cmd: &Command);
    fn last_command(&self) -> Command;
    fn remove_command(&mut self, index: usize);
    fn move_command(&mut self, index: usize, new_pos: usize);
    fn commands_count(&self) -> usize;
    fn executed_command(&self) -> Command;
    fn exec_one(&mut self);
    fn exec_one_call(&mut self);
    fn exec_all(&mut self);
    fn get_next_index(&self) -> usize;
    fn get_prev_index(&self) -> usize;
    fn get_call_index(&self) -> usize;
    fn get_pc(&self) -> usize;
    fn set_pc(&mut self, index: usize);
    fn get_sp(&self) -> usize;
    fn set_sp(&mut self, index: usize);
    fn get_stack(&self) -> Vec<i32>;
    fn get_mp(&self) -> usize;
    fn get_port(&self) -> usize;
    fn get_mem_value(&self) -> usize;
    fn get_reg_q(&self) -> u8;
    fn get_reg(&self, index: usize) -> u8;
    fn get_f(&self) -> u8;
    fn get_y(&self) -> u8;
    fn get_prev_reg_q(&self) -> u8;
    fn get_prev_reg_a(&self) -> u8;
    fn get_prev_reg_b(&self) -> u8;
    fn get_r(&self) -> u8;
    fn get_s(&self) -> u8;
    fn get_z(&self) -> u8;
    fn get_f3(&self) -> u8;
    fn get_c4(&self) -> u8;
    fn get_ovr(&self) -> u8;
    fn get_g(&self) -> u8;
    fn get_p(&self) -> u8;
    fn add_call(&mut self, index: usize, call: Call);
    fn get_call(&self, index: usize) -> Call;
    fn update_call(&mut self, index: usize, call: Call);
    fn remove_call(&mut self, index: usize);
    fn call_count(&self) -> usize;
    fn last_call(&self) -> Call;
    fn open_raw(&mut self, bytes: &[u8]);
    fn export_raw(&self) -> Vec<u8>;
    fn command_get_name(&self, cmd: Command) -> String;
    fn command_get_jump_name(&self, cmd: Command) -> String;
    fn get_state(&self) -> State;
}

#[derive(Default, Debug)]
pub struct OriginalImplementation {
    inst: Option<*mut Emulator>,
}

impl OriginalImplementation {
    pub fn new() -> Self {
        unsafe {
            Self {
                inst: Some(create_emulator()),
            }
        }
    }

}

impl MT1804Emulator for OriginalImplementation {
    fn reset(&mut self) {
        unsafe {
            emulator_reset(self.inst.as_mut().unwrap().to_owned());
        }
    }

    fn get_command(&self, index: usize) -> Command {
        unsafe { emulator_get_command(self.inst.as_ref().unwrap().to_owned(), index as i32) }
    }

    fn add_command(&mut self, index: usize, cmd: &Command) {
        unsafe {
            emulator_add_command(
                self.inst.as_mut().unwrap().to_owned(),
                index as i32,
                cmd.clone(),
            );
        }
    }

    fn update_command(&mut self, index: usize, cmd: &Command) {
        unsafe {
            emulator_update_command(
                self.inst.as_mut().unwrap().to_owned(),
                index as i32,
                cmd.clone(),
            );
        }
    }

    fn last_command(&self) -> Command {
        unsafe { emulator_last_command(self.inst.as_ref().unwrap().to_owned()) }
    }

    fn remove_command(&mut self, index: usize) {
        unsafe { emulator_remove_command(self.inst.as_mut().unwrap().to_owned(), index as i32); }
    }

    fn move_command(&mut self, _index: usize, _new_pos: usize) {
        todo!()
    }

    fn commands_count(&self) -> usize {
        unsafe { emulator_commands_count(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn executed_command(&self) -> Command {
        unsafe { emulator_executed_command(self.inst.as_ref().unwrap().to_owned()) }
    }

    fn exec_one(&mut self) {
        unsafe { emulator_exec_one(self.inst.as_mut().unwrap().to_owned()); }
    }

    fn exec_one_call(&mut self) {
        unsafe { emulator_exec_one_call(self.inst.as_mut().unwrap().to_owned()); }
    }

    fn exec_all(&mut self) {
        unsafe { emulator_exec_all(self.inst.as_mut().unwrap().to_owned()); }
    }

    fn get_next_index(&self) -> usize {
        unsafe { emulator_get_next_index(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn get_prev_index(&self) -> usize {
        unsafe { emulator_get_prev_index(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn get_call_index(&self) -> usize {
        unsafe { emulator_get_call_index(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn get_pc(&self) -> usize {
        unsafe { emulator_get_pc(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn set_pc(&mut self, pc: usize) {
        unsafe { emulator_set_pc(self.inst.as_mut().unwrap().to_owned(), pc as i32); }
    }

    fn get_sp(&self) -> usize {
        unsafe { emulator_get_sp(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn set_sp(&mut self, sp: usize) {
        unsafe { emulator_set_sp(self.inst.as_mut().unwrap().to_owned(), sp as i32); }
    }

    fn get_stack(&self) -> Vec<i32> {
        let stack_len = unsafe { emulator_get_stack_length(self.inst.as_ref().unwrap().to_owned()) as usize };
        let mut stack = Vec::<i32>::new();
        let mut cur_stack_pos: i32 = 0;
        stack.resize_with(stack_len, || {
            let val = unsafe { emulator_get_stack_value(self.inst.as_ref().unwrap().to_owned(), cur_stack_pos) };
            cur_stack_pos += 1;
            val
        });
        stack
    }

    fn get_mp(&self) -> usize {
        unsafe { emulator_get_mp(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn get_port(&self) -> usize {
        unsafe { emulator_get_port(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn get_mem_value(&self) -> usize {
        unsafe { emulator_get_mem_value(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn get_reg_q(&self) -> u8 {
        unsafe { emulator_get_reg_q(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_reg(&self, index: usize) -> u8 {
        unsafe { emulator_get_reg_value(self.inst.as_ref().unwrap().to_owned(), index as i32) as u8 }
    }

    fn get_f(&self) -> u8 {
        unsafe { emulator_get_f(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_y(&self) -> u8 {
        unsafe { emulator_get_y(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_prev_reg_q(&self) -> u8 {
        unsafe { emulator_get_prev_reg_q(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_prev_reg_a(&self) -> u8 {
        unsafe { emulator_get_prev_reg_a(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_prev_reg_b(&self) -> u8 {
        unsafe { emulator_get_prev_reg_b(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_r(&self) -> u8 {
        unsafe { emulator_get_r(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_s(&self) -> u8 {
        unsafe { emulator_get_s(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_z(&self) -> u8 {
        unsafe { emulator_get_z(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_f3(&self) -> u8 {
        unsafe { emulator_get_f3(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_c4(&self) -> u8 {
        unsafe { emulator_get_c4(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_ovr(&self) -> u8 {
        unsafe { emulator_get_ovr(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_g(&self) -> u8 {
        unsafe { emulator_get_g(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn get_p(&self) -> u8 {
        unsafe { emulator_get_p(self.inst.as_ref().unwrap().to_owned()) as u8 }
    }

    fn add_call(&mut self, index: usize, call: Call) {
        unsafe { emulator_add_call(self.inst.as_mut().unwrap().to_owned(), index as i32, call); }
    }

    fn get_call(&self, index: usize) -> Call {
        unsafe { emulator_get_call(self.inst.as_ref().unwrap().to_owned(), index as i32) }
    }

    fn update_call(&mut self, index: usize, call: Call) {
        unsafe { emulator_update_call(self.inst.as_mut().unwrap().to_owned(), index as i32, call); }
    }

    fn remove_call(&mut self, index: usize) {
        unsafe { emulator_remove_call(self.inst.as_mut().unwrap().to_owned(), index as i32); }
    }

    fn call_count(&self) -> usize {
        unsafe { emulator_calls_count(self.inst.as_ref().unwrap().to_owned()) as usize }
    }

    fn last_call(&self) -> Call {
        unsafe { emulator_last_call(self.inst.as_ref().unwrap().to_owned()) }
    }

    fn open_raw(&mut self, bytes: &[u8]) {
        let mut bytes_copy = bytes.to_owned();
        unsafe {
            emulator_open_raw(
                self.inst.as_mut().unwrap().to_owned(),
                bytes_copy.as_mut_ptr(),
                bytes.len(),
            );
        }
    }
    fn export_raw(&self) -> Vec<u8> {
        let mut bytes: *mut u8 = std::ptr::null_mut();
        let mut bytes_cnt: libc::size_t = 0;
        unsafe { emulator_export_raw(self.inst.as_ref().unwrap().to_owned(), &mut bytes, &mut bytes_cnt); }
        let mut bytes_cpy = Vec::<u8>::with_capacity(bytes_cnt);
        for i in 0..bytes_cnt {
            unsafe { bytes_cpy.push(bytes.add(i).read()); }
        }
        unsafe { libc::free(bytes as *mut libc::c_void); }
        bytes_cpy
    }
    fn command_get_name(&self, cmd: Command) -> String {
        let name = unsafe { command_get_name(self.inst.as_ref().unwrap().to_owned(), cmd) };
        let res = String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(name).to_bytes() }).to_string();
        unsafe {
            free_obj(name as *mut libc::c_void);
        }
        res
    }

    fn command_get_jump_name(&self, cmd: Command) -> String {
        let name = unsafe { command_get_jump_name(self.inst.as_ref().unwrap().to_owned(), cmd) };
        let res = String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(name).to_bytes() }).to_string();
        unsafe {
            free_obj(name as *mut libc::c_void);
        }
        res
    }
    fn get_state(&self) -> State {
        State {
            program_counter: self.get_pc(),
            stack_pointer: self.get_sp(),
            // stack_value: self.get_stack_value(),
            multiplexor_value: self.get_mp(),
            // port_value: self.get_port(),
            // mem_value: self.get_mem_value(),
            registers: (0..16)
                .into_iter()
                .map(|ind| { self.get_reg(ind) })
                .chain([self.get_reg_q()].into_iter())
                .collect(),
            flags: [
                self.get_ovr(),
                self.get_c4(),
                self.get_f3(),
                self.get_z(),
                self.get_g(),
                self.get_p()
            ],
            func_output: self.get_f(),
            func_value: self.get_y(),
        }
    }
}

impl std::ops::Drop for OriginalImplementation {
    fn drop(&mut self) {
        if self.inst.is_none() {
            return;
        }
        unsafe {
            destroy_emulator(self.inst.as_mut().unwrap().clone());
        }
        self.inst = None
    }
}

#[derive(Clone, Default, Debug)]
pub struct State {
    pub program_counter: usize,
    pub stack_pointer: usize,
    // pub stack_value: usize,
    pub multiplexor_value: usize,
    // pub port_value: usize,
    // pub mem_value: usize,
    pub func_output: u8,
    pub func_value: u8,
    pub registers: Vec<u8>,
    pub flags: [u8; 6],
}

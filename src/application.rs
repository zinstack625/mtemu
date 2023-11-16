/* application.rs
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


use std::cell::RefCell;
use std::io::BufWriter;
use std::io::prelude::*;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::Arc;

use gtk::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::emulator;
use crate::emulator::MT1804Emulator;
use crate::ui;
use crate::ui::window::MtemuWindow;
use crate::ui::PlainCommandRepr;

mod imp {
    use std::{cell::RefCell, sync::Arc, rc::Rc};
    use gtk::{glib::{once_cell::sync::Lazy, MainContext}, SingleSelection, MultiSelection};

    use crate::{emulator, ui};

    use super::*;

    #[derive(Default)]
    pub struct MtemuApplication {
        emulator: Arc<RefCell<Option<Box<dyn crate::emulator::MT1804Emulator>>>>,
        pub stack_window: RefCell<Option<u32>>,
        pub memory_window: RefCell<Option<u32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MtemuApplication {
        const NAME: &'static str = "MtemuApplication";
        type Type = super::MtemuApplication;
        type ParentType = adw::Application;
    }
    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedCommands")]
    pub struct BoxedCommands(pub Rc<Vec<emulator::Command>>);

    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedCommand")]
    pub struct BoxedCommand(pub Rc<emulator::Command>);

    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedState")]
    pub struct BoxedState(pub Rc<emulator::State>);

    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedStack")]
    pub struct BoxedStack(pub Rc<Vec<u32>>);

    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedMemory")]
    pub struct BoxedMemory(pub Rc<Vec<u32>>);

    impl ObjectImpl for MtemuApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.open-file", &["<primary>o"]);
            obj.set_accels_for_action("app.save-file", &["<primary>s"]);
            obj.set_accels_for_action("app.copy-commands", &["<primary>c"]);
            obj.set_accels_for_action("app.cut-commands", &["<primary>x"]);
            obj.set_accels_for_action("app.paste-commands", &["<primary>v"]);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![glib::subclass::Signal::builder("commands-appeared")
                     .param_types([BoxedCommands::static_type()])
                     .build(),
                     glib::subclass::Signal::builder("state-changed")
                     .param_types([BoxedState::static_type()])
                     .build(),
                     glib::subclass::Signal::builder("command-changed")
                     .param_types([BoxedCommand::static_type()])
                     .build(),
                     glib::subclass::Signal::builder("stack-changed")
                     .param_types([BoxedStack::static_type()])
                     .build(),
                     glib::subclass::Signal::builder("memory-changed")
                     .param_types([BoxedMemory::static_type()])
                     .build()
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl ApplicationImpl for MtemuApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = MtemuWindow::new(&*application);
                window.upcast()
            };
            // Ask the window manager/compositor to present the window
            window.present();
            self.connect_command_appeared();
            self.connect_state_changed();
            self.connect_command_changed();
            self.connect_repr_changed();
            self.connect_stack_changed();
            self.connect_memory_changed();
            self.handle_debug_buttons();
            self.handle_builder_selection_change();
            self.handle_edit_buttons();
            self.handle_code_list_selection_change();
        }
    }

    impl GtkApplicationImpl for MtemuApplication {}
    impl AdwApplicationImpl for MtemuApplication {}
    impl MtemuApplication {
        pub fn set_emulator(&self, emul: Box<dyn crate::emulator::MT1804Emulator>) {
            self.emulator.replace(Some(emul));
        }
        pub fn get_emulator(&self) -> Arc<RefCell<Option<Box<dyn crate::emulator::MT1804Emulator>>>> {
            return self.emulator.clone()
        }
        fn connect_repr_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let code_cmd_list = window.imp().code_view_pane.clone();
            window.imp().instr_repr_sw.connect_closure("state-set", false, glib::closure_local!(move |_: gtk::Switch, state: bool| {
                match state {
                    true => {
                        code_cmd_list.show_binary();
                        code_cmd_list.hide_human();
                    }
                    false => {
                        code_cmd_list.show_human();
                        code_cmd_list.hide_binary();
                    }
                }
                false
            }));
        }
        fn connect_command_appeared(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let code_cmd_list = window.imp().code_view_pane.clone();
            app.connect_closure("commands-appeared", false, glib::closure_local!(move |app: super::MtemuApplication, cmds: BoxedCommands| {
                let model = cmds.0
                    .iter()
                    .map(|cmd| { crate::ui::code_view_pane::CommandRepr::from_command(&app, &cmd) })
                    .collect::<gio::ListStore>();
                code_cmd_list.imp().instance_model(model);
                app.imp().handle_code_list_selection_change();
            }));
        }
        fn connect_state_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let debug_pane = window.imp().debug_pane.clone();
            let code_cmd_list = window.imp().code_view_pane.clone();
            app.connect_closure("state-changed", false, glib::closure_local!(move |_: super::MtemuApplication, state: BoxedState| {
                debug_pane.renew_state(&state.0);
                let Some(selection) = code_cmd_list.imp().code_list.model() else { return };
                selection.select_item(state.0.program_counter as u32, true);
            }));
        }
        fn connect_command_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let _line_builder_pane = window.imp().line_builder_pane.clone();
            let cmd_editor = window.imp().code_view_pane.imp().instruction_editor.clone();
            let line_builder_pane = window.imp().line_builder_pane.clone();
            app.connect_closure("command-changed", false, glib::closure_local!(move |_: super::MtemuApplication, cmd: BoxedCommand| {
                // TODO: it's fixed somehow? need to keep a close eye on this
                line_builder_pane.renew_command(&ui::line_builder_pane::CommandRepr::from_command(&cmd.0));
                cmd_editor.renew_command(&ui::code_view_pane::editor::CommandRepr::from_command(&cmd.0));
            }));
        }
        fn connect_stack_changed(&self) {
            let app_clone = self.obj().clone();
            self.obj().connect_closure("stack-changed", false, glib::closure_local!(move |_: super::MtemuApplication, stack: BoxedStack| {
                let Some(ref stack_id) = *app_clone.imp().stack_window.borrow() else { return };
                let Some(window) = app_clone.window_by_id(*stack_id) else { return };
                let Ok(window) = window.downcast::<ui::stack_view::StackWindow>() else { return };

                let stack_repr = stack.0.iter().enumerate().map(|(ind, val)| {
                    ui::stack_view::StackValueRepr::new(ind as u32, *val as u32)
                }).collect::<gio::ListStore>();
                window.set_stack(stack_repr);
            }));
        }
        fn connect_memory_changed(&self) {
            let app_clone = self.obj().clone();
            self.obj().connect_closure("memory-changed", false, glib::closure_local!(move |_: super::MtemuApplication, memory: BoxedMemory| {
                let Some(ref memory_id) = *app_clone.imp().memory_window.borrow() else { return };
                let Some(window) = app_clone.window_by_id(*memory_id) else { return };
                let Ok(window) = window.downcast::<ui::memory_view::MemoryWindow>() else { return };
                let memory_repr = memory.0.iter().enumerate().map(|(ind, val)| {
                    ui::memory_view::MemoryValueRepr::new(ind as u32, *val as u32)
                }).collect::<gio::ListStore>();
                window.set_memory(memory_repr);
            }));
        }
        fn handle_code_list_selection_change(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let cmd_list = window.imp().code_view_pane.clone();
            let emul = self.get_emulator();
            let closure = glib::closure_local!(move |selection: MultiSelection, _: u32, _: u32| {
                let emul_clone = emul.clone();
                let cmd_cnt = {
                    let Some(ref emul) = *emul_clone.borrow() else { return };
                    emul.commands_count()
                };
                let Some(selected) = ({
                    let mut selected = None;
                    for i in 0..cmd_cnt {
                        if selection.is_selected(i as u32) {
                            selected = Some(i);
                        }
                    }
                    selected
                }) else { return };
                let emul_clone = emul.clone();
                let cmd = BoxedCommand({
                    let Some(ref emul) = *emul_clone.borrow() else { return };
                    Rc::new(emul.get_command(selected))
                });
                app.emit_by_name::<()>("command-changed", &[&cmd]);
            });
            let Some(code_list_model) = cmd_list.imp().code_list.model() else { return };
            code_list_model.connect_closure("selection-changed", false, closure);
        }
        fn handle_builder_selection_change(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let cmd_builder = window.imp().line_builder_pane.imp();
            let cmd_builder_clone = cmd_builder.obj().clone();
            let cmd_editor = window.imp().code_view_pane.imp().instruction_editor.clone();
            let closure = move || {
                let mut new_words = cmd_builder_clone
                    .get_command()
                    .get_words()
                    .into_iter()
                    .map(|word| { word as i32 })
                    .collect::<Vec<i32>>();
                let cur_cmd = cmd_editor.get_codes();
                new_words[7] = cur_cmd[7] as i32;
                new_words[8] = cur_cmd[8] as i32;
                new_words[9] = cur_cmd[9] as i32;
                let cmd = emulator::Command::new(0, new_words.as_mut_slice());
                app.emit_by_name::<()>("command-changed", &[&BoxedCommand(Rc::new(cmd))]);
            };
            let closure_clone = closure.clone();
            let sel_closure = glib::closure_local!(move |_: SingleSelection, _: u32, _: u32| {
               closure_clone();
            });
            let closure_clone = closure.clone();
            let toggle_closure = glib::closure_local!(move |_: gtk::CheckButton| {
                closure_clone();
            });
            let Some(model) = cmd_builder.jump_type.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.alu_instr_type.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.pointer_type.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.interface_type.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.pointer_size.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.op_type.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.load_type.model() else { return };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            cmd_builder.m0_select.connect_closure("toggled", false, toggle_closure.clone());
            cmd_builder.m1_select.connect_closure("toggled", false, toggle_closure.clone());
        }
        fn handle_debug_buttons(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let button_view = window.imp().debug_pane.imp().stepping_view.imp();
            let app_clone = app.clone();
            button_view.step_button.connect_clicked(move |_| {
                let emul = app_clone.get_emulator();
                let prev_cmd = {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    let cmd_count = emul.commands_count();
                    let mut pc = emul.get_pc();
                    // just give me the goddamn exception...
                    if pc == usize::MAX {
                        pc = 0;
                    }
                    if cmd_count == 0 || pc >= cmd_count {
                        // TODO: emit rom end
                        return;
                    }
                    let prev_cmd = emul.get_command(pc);
                    emul.exec_one();
                    prev_cmd
                };
                let state = BoxedState({
                    let Some(ref emul) = *emul.borrow() else { todo!() };
                    Rc::new(emul.get_state())
                });
                let command = BoxedCommand({
                    let emul = app_clone.get_emulator();
                    let Some(ref emul) = *emul.borrow() else { todo!() };
                    Rc::new(emul.get_command(state.0.program_counter))
                });
                app_clone.emit_by_name::<()>("state-changed", &[&state]);
                app_clone.emit_by_name::<()>("command-changed", &[&command]);
                let Some(words) = prev_cmd.get_words() else { return };
                match words[3] {
                    4..=6 | 9 | 10 => {
                        let stack = Rc::new({
                            let Some(ref emul) = *emul.borrow() else { return };
                            emul.get_stack()
                        }.into_iter().map(|val| { val as u32 }).collect::<Vec<u32>>());
                        app_clone.emit_by_name::<()>("stack-changed", &[&BoxedStack(stack)]);
                    }
                    _ => {}
                }
                match words[6] {
                    12 => {
                        let memory = Rc::new({
                           let Some(ref emul) = *emul.borrow() else { return };
                            emul.get_mem().into_iter().map(|val| val as u32).collect::<Vec<u32>>()
                        });
                        app_clone.emit_by_name::<()>("memory-changed", &[&BoxedMemory(memory)]);
                    }
                    _ => {}
                }
            });
            let app_clone = app.clone();
            button_view.reset_button.connect_clicked(move |_| {
                let emul = app_clone.get_emulator();
                {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    emul.reset();
                }
                let state = BoxedState({
                    let Some(ref emul) = *emul.borrow() else { todo!() };
                    // pc returned by engine is set to -1
                    // very hacky and breaks ui but works
                    // because it breaks ui (kinda), we need to
                    // overwrite that
                    let mut state = emul.get_state();
                    state.program_counter = 0;
                    Rc::new(state)
                });
                app_clone.emit_by_name::<()>("state-changed", &[&state]);

                let stack = Rc::new({
                    let Some(ref emul) = *emul.borrow() else { return };
                    emul.get_stack()
                }.into_iter().map(|val| { val as u32 }).collect::<Vec<u32>>());
                app_clone.emit_by_name::<()>("stack_changed", &[&BoxedStack(stack)]);

                let command = BoxedCommand({
                    let Some(ref emul) = *emul.borrow() else { todo!() };
                    if emul.commands_count() == 0 {
                        return;
                    }
                    Rc::new(emul.get_command(state.0.program_counter))
                });
                app_clone.emit_by_name::<()>("command-changed", &[&command]);
            });
            let app_clone = app.clone();
            button_view.run_button.connect_clicked(move |button: &gtk::Button| {
                let context = MainContext::default();
                let button_clone = button.clone();
                context.spawn_local(glib::clone!(@weak app_clone => async move {
                    button_clone.set_sensitive(false);
                    for _ in 0..200 {
                        let emul = app_clone.get_emulator();
                        let prev_cmd = {
                            let Some(ref mut emul) = *emul.borrow_mut() else { return };
                            let cmd_count = emul.commands_count();
                            let mut pc = emul.get_pc();
                            // just give me the goddamn exception...
                            if pc == usize::MAX {
                                pc = 0;
                            }
                            if cmd_count == 0 || pc >= cmd_count {
                                // TODO: emit rom end
                                button_clone.set_sensitive(true);
                                return;
                            }
                            let prev_cmd = emul.get_command(pc);
                            emul.exec_one();
                            prev_cmd
                        };
                        let Some(words) = prev_cmd.get_words() else { return };
                        match words[3] {
                            4..=6 | 9 | 10 => {
                                let stack = Rc::new({
                                    let Some(ref emul) = *emul.borrow() else { return };
                                    emul.get_stack()
                                }.into_iter().map(|val| { val as u32 }).collect::<Vec<u32>>());
                                app_clone.emit_by_name::<()>("stack_changed", &[&BoxedStack(stack)]);
                            }
                            _ => {}
                        }
                        match words[6] {
                            12 => {
                                let memory = Rc::new({
                                    let Some(ref emul) = *emul.borrow() else { return };
                                    emul.get_mem().into_iter().map(|val| val as u32).collect::<Vec<u32>>()
                                });
                                app_clone.emit_by_name::<()>("memory-changed", &[&BoxedMemory(memory)]);
                            }
                            _ => {}
                        }
                        let state = BoxedState({
                            let Some(ref emul) = *emul.borrow() else { todo!() };

                            Rc::new(emul.get_state())
                        });
                        app_clone.emit_by_name::<()>("state-changed", &[&state]);
                        let command = BoxedCommand({
                            let Some(ref emul) = *emul.borrow() else { todo!() };
                            if emul.commands_count() <= state.0.program_counter {
                                // TODO: emit rom end
                                button_clone.set_sensitive(true);
                                return;
                            }
                            Rc::new(emul.get_command(state.0.program_counter))
                        });
                        app_clone.emit_by_name::<()>("command-changed", &[&command]);

                        glib::timeout_future(std::time::Duration::from_millis(20)).await;
                    }
                    button_clone.set_sensitive(true);
                }));
            });
        }
        fn handle_edit_buttons(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let app_clone = app.clone();
            let cmd_view = window.imp().code_view_pane.clone();
            let cmd_view_clone = cmd_view.clone();
            cmd_view.imp().add_button.connect_closure("clicked", false, glib::closure_local!(move |_: gtk::Button| {
                let mut cmd_words = cmd_view_clone.get_codes().into_iter().map(|word: u8| { word as i32 }).collect::<Vec<i32>>();
                let emul = app_clone.get_emulator();
                let cur_cmd_cnt = {
                    let Some(ref emul) = *emul.borrow() else { return };
                    emul.commands_count()
                };
                let selection = cmd_view_clone.imp().code_list.model();
                let position = match selection {
                    None => 0,
                    Some(selection) => {
                        let mut first_selected: Option<u32> = None;
                        for i in 0..cur_cmd_cnt {
                            if selection.is_selected(i as u32) {
                                first_selected = Some((i + 1) as u32);
                                break
                            }
                        }
                        first_selected
                    }.unwrap_or(0)
                };
                {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    emul.add_command(position as usize, &emulator::Command::new(position as i32, &mut cmd_words));
                }
                let commands = BoxedCommands({
                    let Some(ref emul) = *emul.borrow() else { return };
                    let mut commands = Vec::<crate::emulator::Command>::with_capacity(emul.commands_count());
                    for i in 0..emul.commands_count() {
                        commands.push(emul.get_command(i));
                    }
                    Rc::new(commands)
                });
                app_clone.emit_by_name::<()>("commands-appeared", &[&commands]);
            }));
            let app_clone = app.clone();
            let cmd_view_clone = cmd_view.clone();
            cmd_view.imp().delete_button.connect_closure("clicked", false, glib::closure_local!(move |_: gtk::Button| {
                let emul = app_clone.get_emulator();
                let cur_cmd_cnt = {
                    let Some(ref emul) = *emul.borrow() else { return };
                    emul.commands_count()
                };
                let selection = cmd_view_clone.imp().code_list.model();
                let Some(position) = (match selection {
                    None => None,
                    Some(selection) => {
                        let mut selected = Vec::new();
                        for i in 0..cur_cmd_cnt {
                            if selection.is_selected(i as u32) {
                                selected.push(i as usize);
                            }
                        }
                        Some(selected)
                    }
                }) else { return };
                {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    for i in position.into_iter().enumerate() {
                        emul.remove_command(i.1 - i.0);
                    }
                }
                let commands = BoxedCommands({
                    let Some(ref emul) = *emul.borrow() else { return };
                    let mut commands = Vec::<crate::emulator::Command>::with_capacity(emul.commands_count());
                    for i in 0..emul.commands_count() {
                        commands.push(emul.get_command(i));
                    }
                    Rc::new(commands)
                });
                app_clone.emit_by_name::<()>("commands-appeared", &[&commands]);
            }));
            let app_clone = app.clone();
            let cmd_view_clone = cmd_view.clone();
            cmd_view.imp().update_button.connect_closure("clicked", false, glib::closure_local!(move |_: gtk::Button| {
                let mut cmd_words = cmd_view_clone.get_codes().into_iter().map(|word: u8| { word as i32 }).collect::<Vec<i32>>();
                let emul = app_clone.get_emulator();
                let cur_cmd_cnt = {
                    let Some(ref emul) = *emul.borrow() else { return };
                    emul.commands_count()
                };
                let selection = cmd_view_clone.imp().code_list.model();
                let Some(position) = (match selection {
                    None => None,
                    Some(selection) => {
                        let mut selected = Vec::new();
                        for i in 0..cur_cmd_cnt {
                            if selection.is_selected(i as u32) {
                                selected.push(i as usize);
                            }
                        }
                        Some(selected)
                    }
                }) else { return };
                {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    for i in position.into_iter() {
                        emul.remove_command(i);
                        emul.add_command(i, &emulator::Command::new(i as i32, &mut cmd_words))
                    }
                }
                let commands = BoxedCommands({
                    let Some(ref emul) = *emul.borrow() else { return };
                    let mut commands = Vec::<crate::emulator::Command>::with_capacity(emul.commands_count());
                    for i in 0..emul.commands_count() {
                        commands.push(emul.get_command(i));
                    }
                    Rc::new(commands)
                });
                app_clone.emit_by_name::<()>("commands-appeared", &[&commands]);
            }));

        }
    }
}

glib::wrapper! {
    pub struct MtemuApplication(ObjectSubclass<imp::MtemuApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MtemuApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags, emul: Box<dyn crate::emulator::MT1804Emulator>) -> Self {
        let app: MtemuApplication = glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build();
        app.set_emulator(emul);
        app
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let open_file_action = gio::ActionEntry::builder("open-file")
            .activate(move |app: &Self, _, _| app.show_open_file())
            .build();
        let save_file_action = gio::ActionEntry::builder("save-file")
            .activate(move |app: &Self, _, _| app.show_save_file())
            .build();
        let show_debug_action = gio::ActionEntry::builder("show-debug")
            .activate(move |app: &Self, _, _| app.toggle_debug_pane())
            .build();
        let show_builder_action = gio::ActionEntry::builder("show-builder")
            .activate(move |app: &Self, _, _| app.toggle_builder_pane())
            .build();
        let show_stack_action = gio::ActionEntry::builder("show-stack")
            .activate(move |app: &Self, _, _| app.toggle_stack())
            .build();
        let show_memory_action = gio::ActionEntry::builder("show-memory")
            .activate(move |app: &Self, _, _| app.toggle_memory())
            .build();
        let cut_commands_action = gio::ActionEntry::builder("cut-commands")
            .activate(move |app: &Self, _, _| app.cut_commands())
            .build();
        let paste_commands_action = gio::ActionEntry::builder("paste-commands")
            .activate(move |app: &Self, _, _| app.paste_commands())
            .build();
        let copy_commands_action = gio::ActionEntry::builder("copy-commands")
            .activate(move |app: &Self, _, _| app.copy_commands())
            .build();
        self.add_action_entries([quit_action,
                                 about_action,
                                 open_file_action,
                                 save_file_action,
                                 show_debug_action,
                                 show_builder_action,
                                 show_stack_action,
                                 show_memory_action,
                                 cut_commands_action,
                                 paste_commands_action,
                                 copy_commands_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name("mtemu")
            .application_icon("org.bmstu.mtemu")
            .developer_name("Anton Klimanov")
            .version(VERSION)
            .developers(vec!["Anton Klimanov"])
            .copyright("© 2023 Anton Klimanov")
            .build();

        about.present();
    }

    fn show_open_file(&self) {
        let window = self.active_window().unwrap();
        let open_file = gtk::FileDialog::new();
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.mte");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        open_file.set_filters(Some(&filters));
        let emul = self.get_emulator();
        let obj = self.clone();
        open_file.open(Some(&window.clone()), gio::Cancellable::NONE, move |res| {
            let Ok(file) = res else { return };
            let path = file.path().expect("Unable to get file path");
            let file = std::fs::File::open(path).expect("Cannot open file");
            let mut reader = BufReader::new(file);
            let mut bytes = Vec::<u8>::new();
            let _ = reader.read_to_end(&mut bytes);
            {
                let Some(ref mut emul) = *emul.borrow_mut() else { return };
                emul.open_raw(&bytes);
                emul.reset();
            }
            let commands = imp::BoxedCommands({
                let Some(ref emul) = *emul.borrow() else { return };
                let mut commands = Vec::<crate::emulator::Command>::with_capacity(emul.commands_count());
                for i in 0..emul.commands_count() {
                    commands.push(emul.get_command(i));
                }
                Rc::new(commands)
            });
            obj.emit_by_name::<()>("commands-appeared", &[&commands]);
        });
    }
    fn show_save_file(&self) {
        let window = self.active_window().unwrap();
        let open_file = gtk::FileDialog::new();
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.mte");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        open_file.set_filters(Some(&filters));
        let emul = self.get_emulator();
        open_file.save(Some(&window.clone()), gio::Cancellable::NONE, move |res| {
            let Ok(file) = res else { return };
            let path = file.path().expect("Unable to get file path");
            let file = std::fs::File::create(path).expect("Cannot create file");
            let mut writer = BufWriter::new(file);
            let bytes = {
                let Some(ref emul) = *emul.borrow() else { return };
                emul.export_raw()
            };
            let _ = writer.write_all(&bytes);
        });
    }
    fn toggle_debug_pane(&self) {
        let Some(window) = self.active_window().and_downcast::<MtemuWindow>() else { return };
        let visible = window.imp().debug_pane.property::<bool>("visible");
        window.imp().debug_pane.set_property("visible", !visible);
    }
    fn toggle_builder_pane(&self) {
        let Some(window) = self.active_window().and_downcast::<MtemuWindow>() else { return };
        let visible = window.imp().line_builder_pane.property::<bool>("visible");
        window.imp().line_builder_pane.set_property("visible", !visible);
    }
    fn toggle_stack(&self) {
        if let Some(stack_id) = *self.imp().stack_window.borrow() {
            if let Some(window) = self.window_by_id(stack_id) {
                self.remove_window(&window);
                window.destroy();
                return;
            }
        }
        let stack_window = {
            let window = ui::stack_view::StackWindow::new(self);
            self.add_window(&window);
            self.imp().stack_window.replace(Some(window.id()));
            window
        };
        stack_window.present();
        let emul = self.get_emulator();
        let stack = Rc::new({
            let Some(ref emul) = *emul.borrow() else { return };
            emul.get_stack()
        }.into_iter().map(|val| { val as u32 }).collect::<Vec<u32>>());
        self.emit_by_name::<()>("stack-changed", &[&imp::BoxedStack(stack)]);
    }
    fn toggle_memory(&self) {
        if let Some(memory_id) = *self.imp().memory_window.borrow() {
            if let Some(window) = self.window_by_id(memory_id) {
                self.remove_window(&window);
                window.destroy();
                return;
            }
        }
        let memory_window = {
            let window = ui::memory_view::MemoryWindow::new(self);
            self.add_window(&window);
            self.imp().memory_window.replace(Some(window.id()));
            window
        };
        memory_window.present();
        let emul = self.get_emulator();
        let memory = Rc::new({
            let Some(ref emul) = *emul.borrow() else { return };
            emul.get_mem().into_iter().map(|val| val as u32).collect::<Vec<u32>>()
        });
        self.emit_by_name::<()>("memory-changed", &[&imp::BoxedMemory(memory)]);
    }

    fn cut_commands(&self) {
        let window = self.active_window().unwrap();
        let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
        let Some(model) = window.imp().code_view_pane.imp().code_list.model() else { return };
        let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else { return };
        let emul = self.get_emulator();
        let mut cut_commands = Vec::<emulator::Command>::with_capacity(model.n_items() as usize);
        for ind in 0..model.n_items() {
            if model.is_selected(ind) {
                let cmd = {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    let cmd = emul.get_command(ind as usize);
                    emul.remove_command(ind as usize);
                    cmd
                };
                cut_commands.push(cmd);
            }
        }
        let new_commands = imp::BoxedCommands({
            let Some(ref emul) = *emul.borrow() else { return };
            let new_cmd_cnt = emul.commands_count();
            let mut cmds = Vec::<emulator::Command>::with_capacity(new_cmd_cnt);
            for i in 0..new_cmd_cnt {
                cmds.push(emul.get_command(i));
            }
            Rc::new(cmds)
        });
        self.emit_by_name::<()>("commands-appeared", &[&new_commands]);
        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| { Some(disp.clipboard()) }) else { return };
        clipboard.set_text(&String::from_iter(
            cut_commands
                .into_iter()
                .map(|cmd| {
                    let words = cmd.get_words().expect("Failed getting words!");
                    format!(
                        "{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}\n",
                        words[0], words[1], words[2], words[3], words[4],
                        words[5], words[6], words[7], words[8], words[9]
                    )
                })
        ));
    }

    fn copy_commands(&self) {
        let window = self.active_window().unwrap();
        let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
        let Some(model) = window.imp().code_view_pane.imp().code_list.model() else { return };
        let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else { return };
        let emul = self.get_emulator();
        let mut cut_commands = Vec::<emulator::Command>::with_capacity(model.n_items() as usize);
        for ind in 0..model.n_items() {
            if model.is_selected(ind) {
                let cmd = {
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    let cmd = emul.get_command(ind as usize);
                    cmd
                };
                cut_commands.push(cmd);
            }
        }
        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| { Some(disp.clipboard()) }) else { return };
        clipboard.set_text(&String::from_iter(
            cut_commands
                .into_iter()
                .map(|cmd| {
                    let words = cmd.get_words().expect("Failed getting words!");
                    format!(
                        "{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}\n",
                        words[0], words[1], words[2], words[3], words[4],
                        words[5], words[6], words[7], words[8], words[9]
                    )
                })
        ));
    }

    fn paste_commands(&self) {
        let window = self.active_window().unwrap();
        let app = self.clone();

        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| { Some(disp.clipboard()) }) else { return };
        clipboard.read_text_async(gio::Cancellable::NONE, move |result| {
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let Some(model) = window.imp().code_view_pane.imp().code_list.model() else { return };
            let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else { return };
            let selected = {
                let mut selected = None;
                for ind in 0..model.n_items() {
                    if model.is_selected(ind) {
                        selected = Some(ind);
                        break
                    }
                }
                selected
            }.unwrap_or(0);
            let Ok(data) = result else { return };
            let Some(string) = data else { return };
            let cmd_words = string
                .lines()
                .filter_map(|line| {
                    if line.len() != 40 || i32::from_str_radix(line, 2).is_err() {
                        return None;
                    }
                    let mut words = Vec::<i32>::with_capacity(10);
                    for i in (0..=line.len()-4).step_by(4) {
                        words.push(i32::from_str_radix(&line[i..i+4], 2).unwrap_or(0));
                    }
                    Some(words)
                })
                .collect::<Vec<Vec<i32>>>();
            let emul = app.get_emulator();
            let new_commands = imp::BoxedCommands({
                let Some(ref mut emul) = *emul.borrow_mut() else { return };
                cmd_words
                    .into_iter()
                    .enumerate()
                    .for_each(|(ind, words)| {
                        let mut words_boxed = Box::new(words);
                        let cmd = emulator::Command::new(0, words_boxed.as_mut());
                        // +1 is needed to insert command after selected, not before
                        emul.add_command(selected as usize + ind + 1, &cmd);
                        drop(words_boxed);
                    });
                let new_cmd_cnt = emul.commands_count();
                let mut cmds = Vec::<emulator::Command>::with_capacity(new_cmd_cnt);
                for i in 0..new_cmd_cnt {
                    cmds.push(emul.get_command(i));
                }
                Rc::new(cmds)
            });
            app.emit_by_name::<()>("commands-appeared", &[&new_commands]);
        });
    }

    pub fn set_emulator(&self, emul: Box<dyn crate::emulator::MT1804Emulator>) {
        self.imp().set_emulator(emul);
    }
    pub fn get_emulator(&self) -> Arc<RefCell<Option<Box<dyn crate::emulator::MT1804Emulator>>>> {
        self.imp().get_emulator()
    }
}

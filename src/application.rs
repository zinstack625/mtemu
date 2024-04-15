/* application.rs
 *
 * Copyright 2023-2024 Anton Klimanov
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

use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::rc::Rc;

use adw::subclass::prelude::*;
use gtk::glib::GString;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::emulator;
use crate::ui::command_view;
use crate::ui::memory_view;
use crate::ui::stack_view;
use crate::ui::window::MtemuWindow;
use crate::ui::PlainCommandRepr;
use crate::utils;
use crate::utils::get_calls;
use crate::utils::get_commands;
use crate::utils::get_libcalls;

use crate::emulator::MT1804Emulator;

mod imp {
    use gtk::{
        glib::{closure_local, once_cell::sync::Lazy, MainContext},
        MultiSelection, SingleSelection,
    };
    use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};

    use crate::{
        emulator::{self, LibCall},
        ui,
        utils::{self, *},
    };

    use super::*;

    pub struct MtemuApplication {
        emulator: EmulatorStored,
        pub stack_window: RefCell<Option<u32>>,
        pub memory_window: RefCell<Option<u32>>,
        pub commands_window: RefCell<Option<u32>>,
        settings: gio::Settings,
        undo_stack: RefCell<VecDeque<EmulatorStored>>,
    }

    impl Default for MtemuApplication {
        fn default() -> Self {
            Self {
                emulator: Default::default(),
                stack_window: Default::default(),
                memory_window: Default::default(),
                commands_window: Default::default(),
                settings: gio::Settings::new("org.bmstu.mtemu"),
                undo_stack: Default::default(),
            }
        }
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
    #[shared_boxed_type(name = "BoxedCall")]
    pub struct BoxedCall(pub Rc<emulator::Call>);

    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedCalls")]
    pub struct BoxedCalls(pub Rc<Vec<emulator::Call>>);

    #[derive(glib::SharedBoxed, Clone, Debug)]
    #[shared_boxed_type(name = "BoxedLibCalls")]
    pub struct BoxedLibCalls(pub Rc<Vec<emulator::LibCall>>);

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
            obj.set_accels_for_action("app.undo", &["<primary>z"]);
            self.undo_stack
                .borrow_mut()
                .reserve(self.settings.uint("backtrace-steps") as usize);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![
                    glib::subclass::Signal::builder("commands-appeared")
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
                        .build(),
                    glib::subclass::Signal::builder("calls-appeared")
                        .param_types([BoxedCalls::static_type()])
                        .build(),
                    glib::subclass::Signal::builder("callslib-appeared")
                        .param_types([BoxedLibCalls::static_type()])
                        .build(),
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
            self.connect_calls_appeared();
            self.connect_callslib_appeared();
            self.handle_debug_buttons();
            self.handle_builder_selection_change();
            self.handle_edit_buttons();
            self.handle_code_list_selection_change();
            self.obj().emit_by_name::<()>(
                "commands-appeared",
                &[&BoxedCommands(Rc::new(get_commands(self.get_emulator())))],
            );
            self.obj().emit_by_name::<()>(
                "calls-appeared",
                &[&BoxedCalls(Rc::new(get_calls(self.get_emulator())))],
            );
            self.obj().emit_by_name::<()>(
                "callslib-appeared",
                &[&BoxedLibCalls(Rc::new(get_libcalls(self.get_emulator())))],
            );
        }
    }

    impl GtkApplicationImpl for MtemuApplication {}
    impl AdwApplicationImpl for MtemuApplication {}
    impl MtemuApplication {
        fn push_state(&self) {
            let mut undo_stack = self.undo_stack.borrow_mut();
            if undo_stack.len() >= self.settings.uint("backtrace-steps") as usize {
                undo_stack.pop_front();
            }
            undo_stack.push_back(Arc::new(RefCell::new(Some(
                self.get_emulator().borrow().as_ref().unwrap().clone(),
            ))));
        }
        fn pop_state(&self) -> Option<EmulatorStored> {
            self.undo_stack.borrow_mut().pop_back()
        }
        pub fn set_emulator(&self, emul: emulator::OriginalImplementation) {
            self.emulator.replace(Some(emul));
        }
        pub fn get_emulator(&self) -> utils::EmulatorStored {
            return self.emulator.clone();
        }
        fn connect_repr_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let code_cmd_list = window.imp().code_view_pane.clone();
            window.imp().instr_repr_sw.connect_closure(
                "state-set",
                false,
                glib::closure_local!(move |_: gtk::Switch, state: bool| {
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
                }),
            );
        }
        fn connect_command_appeared(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let code_cmd_list = window.imp().code_view_pane.clone();
            app.connect_closure(
                "commands-appeared",
                false,
                glib::closure_local!(move |app: super::MtemuApplication, cmds: BoxedCommands| {
                    let model = cmds
                        .0
                        .iter()
                        .map(|cmd| crate::ui::code_view_pane::CommandRepr::from_command(&app, &cmd))
                        .collect::<gio::ListStore>();
                    code_cmd_list.imp().instance_model(model);
                    app.imp().handle_code_list_selection_change();
                }),
            );
        }
        fn connect_state_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let debug_pane = window.imp().debug_pane.clone();
            let code_cmd_list = window.imp().code_view_pane.clone();
            app.connect_closure(
                "state-changed",
                false,
                glib::closure_local!(move |_: super::MtemuApplication, state: BoxedState| {
                    debug_pane.renew_state(&state.0);
                    let Some(selection) = code_cmd_list.imp().code_list.model() else {
                        return;
                    };
                    selection.select_item(state.0.program_counter as u32, true);
                }),
            );
        }
        fn connect_command_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let _line_builder_pane = window.imp().line_builder_pane.clone();
            let cmd_editor = window.imp().code_view_pane.imp().instruction_editor.clone();
            let line_builder_pane = window.imp().line_builder_pane.clone();
            app.connect_closure(
                "command-changed",
                false,
                glib::closure_local!(move |_: super::MtemuApplication, cmd: BoxedCommand| {
                    line_builder_pane
                        .renew_command(&ui::line_builder_pane::CommandRepr::from_command(&cmd.0));
                    cmd_editor.renew_command(
                        &ui::code_view_pane::editor::CommandRepr::from_command(&cmd.0),
                    );
                }),
            );
        }
        fn connect_stack_changed(&self) {
            let app_clone = self.obj().clone();
            self.obj().connect_closure(
                "stack-changed",
                false,
                glib::closure_local!(move |_: super::MtemuApplication, stack: BoxedStack| {
                    let Some(ref stack_id) = *app_clone.imp().stack_window.borrow() else {
                        return;
                    };
                    let Some(window) = app_clone.window_by_id(*stack_id) else {
                        return;
                    };
                    let Ok(window) = window.downcast::<ui::stack_view::StackWindow>() else {
                        return;
                    };

                    let stack_repr = stack
                        .0
                        .iter()
                        .enumerate()
                        .map(|(ind, val)| {
                            ui::stack_view::StackValueRepr::new(ind as u32, *val as u32)
                        })
                        .collect::<gio::ListStore>();
                    window.set_stack(stack_repr);
                }),
            );
        }
        fn connect_memory_changed(&self) {
            let app_clone = self.obj().clone();
            self.obj().connect_closure(
                "memory-changed",
                false,
                glib::closure_local!(move |_: super::MtemuApplication, memory: BoxedMemory| {
                    let Some(ref memory_id) = *app_clone.imp().memory_window.borrow() else {
                        return;
                    };
                    let Some(window) = app_clone.window_by_id(*memory_id) else {
                        return;
                    };
                    let Ok(window) = window.downcast::<ui::memory_view::MemoryWindow>() else {
                        return;
                    };
                    let memory_repr = memory
                        .0
                        .iter()
                        .enumerate()
                        .map(|(ind, val)| {
                            ui::memory_view::MemoryValueRepr::new(ind as u32, *val as u32)
                        })
                        .collect::<gio::ListStore>();
                    window.set_memory(memory_repr);
                }),
            );
        }
        fn handle_code_list_selection_change(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let cmd_list = window.imp().code_view_pane.clone();
            let emul = self.get_emulator();
            let closure = glib::closure_local!(move |selection: MultiSelection, _: u32, _: u32| {
                let emul_clone = emul.clone();
                let cmd_cnt = {
                    let Some(ref emul) = *emul_clone.borrow() else {
                        return;
                    };
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
                }) else {
                    return;
                };
                let emul_clone = emul.clone();
                let cmd = BoxedCommand({
                    let Some(ref emul) = *emul_clone.borrow() else {
                        return;
                    };
                    Rc::new(emul.get_command(selected))
                });
                app.emit_by_name::<()>("command-changed", &[&cmd]);
            });
            let Some(code_list_model) = cmd_list.imp().code_list.model() else {
                return;
            };
            code_list_model.connect_closure("selection-changed", false, closure);
        }
        fn handle_builder_selection_change(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let cmd_builder = window.imp().line_builder_pane.imp();
            let cmd_builder_clone = cmd_builder.obj().clone();
            let cmd_editor = window.imp().code_view_pane.imp().instruction_editor.clone();
            let closure = move || {
                let mut new_words = cmd_builder_clone
                    .get_command()
                    .get_words()
                    .into_iter()
                    .map(|word| word as i32)
                    .collect::<Vec<i32>>();
                let cur_cmd = cmd_editor.get_codes();
                if !(new_words[6] == 0b1011 && new_words[5] == 0b1000) {
                    new_words[7] = cur_cmd[7] as i32;
                }
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
            let Some(model) = cmd_builder.jump_type.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.alu_instr_type.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.pointer_type.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.interface_type.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.pointer_size.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.op_type.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            let Some(model) = cmd_builder.load_type.model() else {
                return;
            };
            model.connect_closure("selection-changed", false, sel_closure.clone());
            cmd_builder
                .m0_select
                .connect_closure("toggled", false, toggle_closure.clone());
            cmd_builder
                .m1_select
                .connect_closure("toggled", false, toggle_closure.clone());
        }
        fn handle_debug_buttons(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let debug_view = &window.imp().debug_pane;
            let app_clone = app.clone();
            let executor = move |app: &super::MtemuApplication| -> bool {
                let emul = app.get_emulator();
                app.imp().push_state();
                let prev_cmd = {
                    let Some(ref mut emul) = *(*emul).borrow_mut() else {
                        return false;
                    };
                    let cmd_count = emul.commands_count();
                    let mut pc = emul.get_pc();
                    // just give me the goddamn exception...
                    if pc == usize::MAX {
                        pc = 0;
                    }
                    if cmd_count == 0 || pc >= cmd_count {
                        // TODO: emit rom end
                        return false;
                    }
                    let prev_cmd = emul.get_command(pc);
                    emul.exec_one();
                    prev_cmd
                };
                let Some(words) = prev_cmd.get_words() else {
                    return false;
                };
                match words[3] {
                    4..=6 | 9 | 10 => {
                        let stack = Rc::new(
                            {
                                let Some(ref emul) = *emul.borrow() else {
                                    return false;
                                };
                                emul.get_stack()
                            }
                            .into_iter()
                            .map(|val| val as u32)
                            .collect::<Vec<u32>>(),
                        );
                        app.emit_by_name::<()>("stack_changed", &[&BoxedStack(stack)]);
                    }
                    _ => {}
                }
                match words[6] {
                    12 => {
                        let memory = Rc::new({
                            let Some(ref emul) = *emul.borrow() else {
                                return false;
                            };
                            emul.get_mem()
                                .into_iter()
                                .map(|val| val as u32)
                                .collect::<Vec<u32>>()
                        });
                        app.emit_by_name::<()>("memory-changed", &[&BoxedMemory(memory)]);
                    }
                    _ => {}
                }
                let state = BoxedState({
                    let Some(ref emul) = *emul.borrow() else {
                        return false;
                    };
                    Rc::new(emul.get_state())
                });
                app.emit_by_name::<()>("state-changed", &[&state]);
                let command = BoxedCommand({
                    let Some(ref emul) = *emul.borrow() else {
                        return false;
                    };
                    if emul.commands_count() <= state.0.program_counter {
                        // TODO: emit rom end
                        return false;
                    }
                    Rc::new(emul.get_command(state.0.program_counter))
                });
                app.emit_by_name::<()>("command-changed", &[&command]);
                return true;
            };
            debug_view.connect_closure(
                "step-clicked",
                false,
                closure_local!(move |_: glib::Object, _: &gtk::Button| {
                    let _ = executor(&app_clone);
                }),
            );
            let app_clone = app.clone();
            debug_view.connect_closure(
                "reset-clicked",
                false,
                closure_local!(move |_: glib::Object, _: &gtk::Button| {
                    let emul = app_clone.get_emulator();
                    app_clone.imp().push_state();
                    {
                        let Some(ref mut emul) = *(*emul).borrow_mut() else {
                            return;
                        };
                        emul.reset();
                    }
                    let state = BoxedState({
                        let Some(ref emul) = *emul.borrow() else {
                            todo!()
                        };
                        // pc returned by engine is set to -1
                        // very hacky and breaks ui but works
                        // because it breaks ui (kinda), we need to
                        // overwrite that
                        let mut state = emul.get_state();
                        state.program_counter = 0;
                        Rc::new(state)
                    });
                    app_clone.emit_by_name::<()>("state-changed", &[&state]);

                    let stack = Rc::new(
                        {
                            let Some(ref emul) = *emul.borrow() else {
                                return;
                            };
                            emul.get_stack()
                        }
                        .into_iter()
                        .map(|val| val as u32)
                        .collect::<Vec<u32>>(),
                    );
                    app_clone.emit_by_name::<()>("stack_changed", &[&BoxedStack(stack)]);

                    let command = BoxedCommand({
                        let Some(ref emul) = *emul.borrow() else {
                            todo!()
                        };
                        if emul.commands_count() == 0 {
                            return;
                        }
                        Rc::new(emul.get_command(state.0.program_counter))
                    });
                    app_clone.emit_by_name::<()>("command-changed", &[&command]);
                }),
            );
            let app_clone = app.clone();
            debug_view.connect_closure(
                "run-toggled",
                false,
                closure_local!(move |_: glib::Object, button: &gtk::ToggleButton| {
                    let context = MainContext::default();
                    let button_clone = button.clone();
                    context.spawn_local(glib::clone!(@weak app_clone => async move {
                        while button_clone.is_active() {
                            if !executor(&app_clone) {
                                return;
                            }
                            glib::timeout_future(std::time::Duration::from_millis(20)).await;
                        }
                    }));
                }),
            );
        }
        fn handle_edit_buttons(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else {
                return;
            };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else {
                return;
            };
            let app_clone = app.clone();
            let cmd_view = window.imp().code_view_pane.clone();
            let cmd_view_clone = cmd_view.clone();
            cmd_view.imp().add_button.connect_closure(
                "clicked",
                false,
                glib::closure_local!(move |_: gtk::Button| {
                    let mut cmd_words = cmd_view_clone
                        .get_codes()
                        .into_iter()
                        .map(|word: u8| word as i32)
                        .collect::<Vec<i32>>();
                    let emul = app_clone.get_emulator();
                    let cur_cmd_cnt = {
                        let Some(ref emul) = *emul.borrow() else {
                            return;
                        };
                        emul.commands_count()
                    };
                    let selection = cmd_view_clone.imp().code_list.model();
                    let position = match selection {
                        None => cur_cmd_cnt as u32,
                        Some(selection) => {
                            let mut first_selected: Option<u32> = None;
                            for i in 0..cur_cmd_cnt {
                                if selection.is_selected(i as u32) {
                                    first_selected = Some((i + 1) as u32);
                                    break;
                                }
                            }
                            first_selected
                        }
                        .unwrap_or(0),
                    };
                    app_clone.imp().push_state();
                    {
                        let Some(ref mut emul) = *(*emul).borrow_mut() else {
                            return;
                        };
                        emul.add_command(
                            position as usize,
                            &emulator::Command::new(position as i32, &mut cmd_words),
                        );
                    }
                    app_clone.emit_by_name::<()>(
                        "commands-appeared",
                        &[&BoxedCommands(Rc::new(get_commands(
                            app_clone.get_emulator(),
                        )))],
                    );
                }),
            );
            let app_clone = app.clone();
            let cmd_view_clone = cmd_view.clone();
            cmd_view.imp().delete_button.connect_closure(
                "clicked",
                false,
                glib::closure_local!(move |_: gtk::Button| {
                    let emul = app_clone.get_emulator();
                    let selection = cmd_view_clone.imp().code_list.model();
                    let Some(position) = (match selection {
                        None => None,
                        Some(selection) => {
                            let mut selected = Vec::new();
                            for i in 0..selection.n_items() {
                                if selection.is_selected(i as u32) {
                                    selected.push(i as usize);
                                }
                            }
                            Some(selected)
                        }
                    }) else {
                        return;
                    };
                    app_clone.imp().push_state();
                    {
                        let Some(ref mut emul) = *(*emul).borrow_mut() else {
                            return;
                        };
                        for i in position.into_iter().enumerate() {
                            emul.remove_command(i.1 - i.0);
                        }
                    }
                    app_clone.emit_by_name::<()>(
                        "commands-appeared",
                        &[&BoxedCommands(Rc::new(get_commands(
                            app_clone.get_emulator(),
                        )))],
                    );
                }),
            );
            let app_clone = app.clone();
            let cmd_view_clone = cmd_view.clone();
            cmd_view.imp().update_button.connect_closure(
                "clicked",
                false,
                glib::closure_local!(move |_: gtk::Button| {
                    let mut cmd_words = cmd_view_clone
                        .get_codes()
                        .into_iter()
                        .map(|word: u8| word as i32)
                        .collect::<Vec<i32>>();
                    let emul = app_clone.get_emulator();
                    let cur_cmd_cnt = {
                        let Some(ref emul) = *emul.borrow() else {
                            return;
                        };
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
                    }) else {
                        return;
                    };
                    app_clone.imp().push_state();
                    {
                        let Some(ref mut emul) = *(*emul).borrow_mut() else {
                            return;
                        };
                        for i in position.into_iter() {
                            emul.remove_command(i);
                            emul.add_command(i, &emulator::Command::new(i as i32, &mut cmd_words))
                        }
                    }
                    app_clone.emit_by_name::<()>(
                        "commands-appeared",
                        &[&BoxedCommands(Rc::new(get_commands(
                            app_clone.get_emulator(),
                        )))],
                    );
                }),
            );
        }
        pub fn handle_command_buttons(&self, commands: ui::command_view::CommandWindow) {
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "add-clicked",
                false,
                glib::closure_local!(
                    move |_: ui::command_view::CommandWindow,
                          call: ui::command_view::CallValueRepr| {
                        app_clone.imp().push_state();
                        {
                            let emul = app_clone.get_emulator();
                            let Some(ref mut emul) = *emul.as_ref().borrow_mut() else {
                                return;
                            };
                            let emul_repr = emulator::Call {
                                code_: call.code() as i32,
                                arg0_: call.arg0() as i32,
                                arg1_: call.arg1() as i32,
                            };
                            emul.add_call(call.addr() as usize, emul_repr);
                        }
                        app_clone.emit_by_name::<()>(
                            "calls-appeared",
                            &[&BoxedCalls(Rc::new(get_calls(app_clone.get_emulator())))],
                        );
                    }
                ),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "update-clicked",
                false,
                glib::closure_local!(
                    move |_: ui::command_view::CommandWindow,
                          call: ui::command_view::CallValueRepr| {
                        app_clone.imp().push_state();
                        {
                            let emul = app_clone.get_emulator();
                            let Some(ref mut emul) = *emul.as_ref().borrow_mut() else {
                                return;
                            };
                            let emul_repr = emulator::Call {
                                code_: call.code() as i32,
                                arg0_: call.arg0() as i32,
                                arg1_: call.arg1() as i32,
                            };
                            emul.update_call(call.addr() as usize, emul_repr);
                        }
                        app_clone.emit_by_name::<()>(
                            "calls-appeared",
                            &[&BoxedCalls(Rc::new(get_calls(app_clone.get_emulator())))],
                        );
                    }
                ),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "delete-clicked",
                false,
                glib::closure_local!(move |_: ui::command_view::CommandWindow, calls: command_view::BoxedCalls| {
                    app_clone.imp().push_state();
                    {
                        let emul = app_clone.get_emulator();
                        let Some(ref mut emul) = *emul.as_ref().borrow_mut() else {
                            return;
                        };
                        let mut deleted = 0;
                        for call in calls.0.iter() {
                            emul.remove_call(call.addr() as usize - deleted);
                            deleted += 1;
                        }
                    }
                    app_clone.emit_by_name::<()>(
                        "calls-appeared",
                        &[&BoxedCalls(Rc::new(get_calls(app_clone.get_emulator())))],
                    );
                }),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "step-clicked",
                false,
                glib::closure_local!(move |win: ui::command_view::CommandWindow| {
                    app_clone.imp().push_state();
                    let (stack, memory, state) = {
                        let emul = app_clone.get_emulator();
                        let Some(ref mut emul) = *emul.as_ref().borrow_mut() else {
                            return;
                        };
                        emul.exec_one_call();
                        let index = emul.get_call_index();
                        win.set_call_index(index as u32);
                        let stack = Rc::new(
                            emul.get_stack()
                            .into_iter()
                            .map(|val| val as u32)
                            .collect::<Vec<u32>>(),
                        );
                        let memory = Rc::new({
                            emul.get_mem()
                                .into_iter()
                                .map(|val| val as u32)
                                .collect::<Vec<u32>>()
                        });
                        let state = BoxedState(Rc::new(emul.get_state()));
                        (stack, memory, state)
                    };
                    app_clone.emit_by_name::<()>("stack_changed", &[&BoxedStack(stack)]);
                    app_clone.emit_by_name::<()>("memory-changed", &[&BoxedMemory(memory)]);
                    app_clone.emit_by_name::<()>("state-changed", &[&state]);
                }),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "run-toggled",
                false,
                glib::closure_local!(
                    move |win: ui::command_view::CommandWindow, but: &gtk::ToggleButton| {
                        let context = MainContext::default();
                        let button_clone = but.clone();
                        context.spawn_local(glib::clone!(@weak app_clone => async move {
                    let emul = app_clone.get_emulator();
                    while button_clone.is_active() {
                        app_clone.imp().push_state();
                        let (stack, memory, state) = {
                            let Some(ref mut emul) = *emul.as_ref().borrow_mut() else { return };
                            emul.exec_one_call();
                            let index = emul.get_call_index();
                            eprintln!("call index: {}", index);
                            win.set_call_index(index as u32);
                            let stack = Rc::new(
                                emul.get_stack()
                                    .into_iter()
                                    .map(|val| val as u32)
                                    .collect::<Vec<u32>>(),
                            );
                            let memory = Rc::new({
                                emul.get_mem()
                                    .into_iter()
                                    .map(|val| val as u32)
                                    .collect::<Vec<u32>>()
                            });
                            let state = BoxedState(Rc::new(emul.get_state()));
                            (stack, memory, state)
                        };
                        app_clone.emit_by_name::<()>("stack_changed", &[&BoxedStack(stack)]);
                        app_clone.emit_by_name::<()>("memory-changed", &[&BoxedMemory(memory)]);
                        app_clone.emit_by_name::<()>("state-changed", &[&state]);
                        glib::timeout_future(std::time::Duration::from_millis(20)).await;
                    }
                }));
                    }
                ),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "reset-clicked",
                false,
                glib::closure_local!(move |win: ui::command_view::CommandWindow| {
                    app_clone.imp().push_state();
                    let emul = app_clone.get_emulator();
                    let Some(ref mut emul) = *emul.as_ref().borrow_mut() else {
                        return;
                    };
                    emul.reset();
                    let index = emul.get_call_index();
                    win.set_call_index(index as u32);
                }),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "lib-add-clicked",
                false,
                glib::closure_local!(move |_: ui::command_view::CommandWindow,
                                           code: i32,
                                           name: String,
                                           addr: i32| {
                    app_clone.imp().push_state();
                    let emul = app_clone.get_emulator();
                    {
                        let Some(ref mut emul) = *(*emul).borrow_mut() else {
                            return;
                        };
                        emul.add_map_call(&LibCall::new(code, name.to_owned(), addr));
                    }
                    app_clone.emit_by_name::<()>(
                        "callslib-appeared",
                        &[&BoxedLibCalls(Rc::new(get_libcalls(
                            app_clone.get_emulator(),
                        )))],
                    );
                }),
            );
            let app_clone = self.obj().clone();
            commands.connect_closure(
                "lib-delete-clicked",
                false,
                glib::closure_local!(move |_: ui::command_view::CommandWindow, code: i32| {
                    app_clone.imp().push_state();
                    let emul = app_clone.get_emulator();
                    {
                        let Some(ref mut emul) = *(*emul).borrow_mut() else {
                            return;
                        };
                        emul.remove_map_call(code);
                    }
                    app_clone.emit_by_name::<()>(
                        "callslib-appeared",
                        &[&BoxedLibCalls(Rc::new(get_libcalls(
                            app_clone.get_emulator(),
                        )))],
                    );
                    app_clone.emit_by_name::<()>(
                        "calls-appeared",
                        &[&BoxedCalls(Rc::new(get_calls(app_clone.get_emulator())))],
                    );
                }),
            );
        }
        fn connect_calls_appeared(&self) {
            let app_clone = self.obj().clone();
            self.obj().connect_closure(
                "calls-appeared",
                false,
                glib::closure_local!(move |_: super::MtemuApplication, calls: BoxedCalls| {
                    let Some(ref commands_id) = *app_clone.imp().commands_window.borrow() else {
                        return;
                    };
                    let Some(window) = app_clone.window_by_id(*commands_id) else {
                        return;
                    };
                    let Ok(window) = window.downcast::<ui::command_view::CommandWindow>() else {
                        return;
                    };
                    let calls_repr = calls
                        .0
                        .iter()
                        .enumerate()
                        .map(|(ind, val)| {
                            ui::command_view::CallValueRepr::new(
                                ind as u32,
                                val.code_ as u32,
                                val.arg0_ as u8,
                                val.arg1_ as u8,
                            )
                        })
                        .collect::<gio::ListStore>();
                    window.set_commands(calls_repr);
                }),
            );
        }
        fn connect_callslib_appeared(&self) {
            let app_clone = self.obj().clone();
            self.obj().connect_closure(
                "callslib-appeared",
                false,
                glib::closure_local!(move |_: super::MtemuApplication, calls: BoxedLibCalls| {
                    let Some(ref commands_id) = *app_clone.imp().commands_window.borrow() else {
                        return;
                    };
                    let Some(window) = app_clone.window_by_id(*commands_id) else {
                        return;
                    };
                    let Ok(window) = window.downcast::<ui::command_view::CommandWindow>() else {
                        return;
                    };
                    let calls_repr = calls
                        .0
                        .iter()
                        .map(|val| {
                            ui::command_view::LibCallValueRepr::new(
                                val.code as u32,
                                &val.name,
                                val.addr as u32,
                            )
                        })
                        .collect::<gio::ListStore>();
                    window.set_command_library(calls_repr);
                }),
            );
        }
        pub fn init_library(&self) {
            self.push_state();
            {
                let Some(ref mut emul) = *self.emulator.as_ref().borrow_mut() else {
                    return;
                };
                emul.init_library();
            }
            self.obj().emit_by_name::<()>(
                "commands-appeared",
                &[&BoxedCommands(Rc::new(get_commands(self.get_emulator())))],
            );
            self.obj().emit_by_name::<()>(
                "calls-appeared",
                &[&BoxedCalls(Rc::new(get_calls(self.get_emulator())))],
            );
            self.obj().emit_by_name::<()>(
                "callslib-appeared",
                &[&BoxedLibCalls(Rc::new(get_libcalls(self.get_emulator())))],
            );
        }

        pub fn undo(&self) {
            match self.pop_state() {
                None => {}
                Some(old_state) => {
                    let emul = self.get_emulator();
                    let Some(ref mut emul) = *(*emul).borrow_mut() else { return };
                    emul.swap(&mut old_state.take().unwrap())
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct MtemuApplication(ObjectSubclass<imp::MtemuApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MtemuApplication {
    pub fn new(
        application_id: &str,
        flags: &gio::ApplicationFlags,
        emul: emulator::OriginalImplementation,
    ) -> Self {
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
        let show_commands_action = gio::ActionEntry::builder("show-commands")
            .activate(move |app: &Self, _, _| app.toggle_commands())
            .build();
        let init_library_action = gio::ActionEntry::builder("init-library")
            .activate(move |app: &Self, _, _| app.init_library())
            .build();
        let undo_action = gio::ActionEntry::builder("undo")
            .activate(move |app: &Self, _, _| app.undo())
            .build();
        self.add_action_entries([
            quit_action,
            about_action,
            open_file_action,
            save_file_action,
            show_debug_action,
            show_builder_action,
            show_stack_action,
            show_memory_action,
            cut_commands_action,
            paste_commands_action,
            copy_commands_action,
            show_commands_action,
            init_library_action,
            undo_action,
        ]);
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

    fn undo(&self) {
        self.imp().undo();
        self.emit_by_name::<()>(
            "commands-appeared",
            &[&imp::BoxedCommands(Rc::new(get_commands(
                self.get_emulator(),
            )))],
        );
        self.emit_by_name::<()>(
            "calls-appeared",
            &[&imp::BoxedCalls(Rc::new(get_calls(self.get_emulator())))],
        );
        self.emit_by_name::<()>(
            "callslib-appeared",
            &[&imp::BoxedLibCalls(Rc::new(get_libcalls(
                self.get_emulator(),
            )))],
        );

        let emul = self.get_emulator();
        let memory = Rc::new({
            let Some(ref emul) = *emul.borrow() else {
                return;
            };
            emul.get_mem()
                .into_iter()
                .map(|val| val as u32)
                .collect::<Vec<u32>>()
        });
        self.emit_by_name::<()>("memory-changed", &[&imp::BoxedMemory(memory)]);

        let stack = Rc::new(
            {
                let Some(ref emul) = *emul.borrow() else {
                    return;
                };
                emul.get_stack()
            }
            .into_iter()
            .map(|val| val as u32)
            .collect::<Vec<u32>>(),
        );
        self.emit_by_name::<()>("stack-changed", &[&imp::BoxedStack(stack)]);

        let state = {
            let Some(ref emul) = *emul.borrow() else { return };
            Rc::new(emul.get_state())
        };
        self.emit_by_name::<()>("state-changed", &[&imp::BoxedState(state)]);
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
                let Some(ref mut emul) = *emul.borrow_mut() else {
                    return;
                };
                emul.open_raw(&bytes);
                emul.reset();
            }
            obj.emit_by_name::<()>(
                "commands-appeared",
                &[&imp::BoxedCommands(Rc::new(get_commands(
                    obj.get_emulator(),
                )))],
            );
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
            let mut path = file.path().expect("Unable to get file path");
            path.set_extension("mte");
            let file = std::fs::File::create(path).expect("Cannot create file");
            let mut writer = BufWriter::new(file);
            let bytes = {
                let Some(ref emul) = *emul.borrow() else {
                    return;
                };
                emul.export_raw()
            };
            let _ = writer.write_all(&bytes);
        });
    }
    fn toggle_debug_pane(&self) {
        let Some(window) = self.active_window().and_downcast::<MtemuWindow>() else {
            return;
        };
        let visible = window.imp().debug_pane.property::<bool>("visible");
        window.imp().debug_pane.set_property("visible", !visible);
    }
    fn toggle_builder_pane(&self) {
        let Some(window) = self.active_window().and_downcast::<MtemuWindow>() else {
            return;
        };
        let visible = window.imp().line_builder_pane.property::<bool>("visible");
        window
            .imp()
            .line_builder_pane
            .set_property("visible", !visible);
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
            let window = stack_view::StackWindow::new(self);
            self.add_window(&window);
            self.imp().stack_window.replace(Some(window.id()));
            window
        };
        stack_window.present();
        let emul = self.get_emulator();
        let stack = Rc::new(
            {
                let Some(ref emul) = *emul.borrow() else {
                    return;
                };
                emul.get_stack()
            }
            .into_iter()
            .map(|val| val as u32)
            .collect::<Vec<u32>>(),
        );
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
            let window = memory_view::MemoryWindow::new(self);
            self.add_window(&window);
            self.imp().memory_window.replace(Some(window.id()));
            window
        };
        memory_window.present();
        let emul = self.get_emulator();
        let memory = Rc::new({
            let Some(ref emul) = *emul.borrow() else {
                return;
            };
            emul.get_mem()
                .into_iter()
                .map(|val| val as u32)
                .collect::<Vec<u32>>()
        });
        self.emit_by_name::<()>("memory-changed", &[&imp::BoxedMemory(memory)]);
    }

    fn cut_commands(&self) {
        let window = self.active_window().unwrap();
        let Some(window) = window.downcast_ref::<MtemuWindow>() else {
            return;
        };
        let Some(model) = window.imp().code_view_pane.imp().code_list.model() else {
            return;
        };
        let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else {
            return;
        };
        let emul = self.get_emulator();
        let mut cut_commands = Vec::<emulator::Command>::with_capacity(model.n_items() as usize);
        for ind in 0..model.n_items() {
            if model.is_selected(ind) {
                let cmd = {
                    let Some(ref mut emul) = *emul.borrow_mut() else {
                        return;
                    };
                    let cmd = emul.get_command(ind as usize);
                    emul.remove_command(ind as usize);
                    cmd
                };
                cut_commands.push(cmd);
            }
        }
        let new_commands = imp::BoxedCommands({
            let Some(ref emul) = *emul.borrow() else {
                return;
            };
            let new_cmd_cnt = emul.commands_count();
            let mut cmds = Vec::<emulator::Command>::with_capacity(new_cmd_cnt);
            for i in 0..new_cmd_cnt {
                cmds.push(emul.get_command(i));
            }
            Rc::new(cmds)
        });
        self.emit_by_name::<()>("commands-appeared", &[&new_commands]);
        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| Some(disp.clipboard()))
        else {
            return;
        };
        clipboard.set_text(&String::from_iter(cut_commands.into_iter().map(|cmd| {
            let words = cmd.get_words().expect("Failed getting words!");
            format!(
                "{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}\n",
                words[0],
                words[1],
                words[2],
                words[3],
                words[4],
                words[5],
                words[6],
                words[7],
                words[8],
                words[9]
            )
        })));
    }

    fn copy_microcommands(&self, window: &MtemuWindow) {
        let Some(model) = window.imp().code_view_pane.imp().code_list.model() else {
            return;
        };
        let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else {
            return;
        };
        let emul = self.get_emulator();
        let mut cut_commands = Vec::<emulator::Command>::with_capacity(model.n_items() as usize);
        for ind in 0..model.n_items() {
            if model.is_selected(ind) {
                let cmd = {
                    let Some(ref mut emul) = *emul.borrow_mut() else {
                        return;
                    };
                    let cmd = emul.get_command(ind as usize);
                    cmd
                };
                cut_commands.push(cmd);
            }
        }
        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| Some(disp.clipboard()))
        else {
            return;
        };
        clipboard.set_text(&String::from_iter(cut_commands.into_iter().map(|cmd| {
            let words = cmd.get_words().expect("Failed getting words!");
            format!(
                "{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}\n",
                words[0],
                words[1],
                words[2],
                words[3],
                words[4],
                words[5],
                words[6],
                words[7],
                words[8],
                words[9]
            )
        })));
    }

    fn copy_metacommands(&self, window: &command_view::CommandWindow) {
        let Some(model) = window.imp().command_list.model() else {
            return;
        };
        let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else {
            return;
        };
        let emul = self.get_emulator();
        let mut cut_commands = Vec::<emulator::Command>::with_capacity(model.n_items() as usize);
        for ind in 0..model.n_items() {
            if model.is_selected(ind) {
                let cmd = {
                    let Some(ref mut emul) = *emul.borrow_mut() else {
                        return;
                    };
                    let cmd = emul.get_command(ind as usize);
                    cmd
                };
                cut_commands.push(cmd);
            }
        }
        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| Some(disp.clipboard()))
        else {
            return;
        };
        clipboard.set_text(&String::from_iter(cut_commands.into_iter().map(|cmd| {
            let words = cmd.get_words().expect("Failed getting words!");
            format!(
                "{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}{:0>4b}\n",
                words[0],
                words[1],
                words[2],
                words[3],
                words[4],
                words[5],
                words[6],
                words[7],
                words[8],
                words[9]
            )
        })));
    }
    
    fn copy_commands(&self) {
        let window = self.active_window().unwrap();
        if let Some(window) = window.downcast_ref::<MtemuWindow>() {
            self.copy_microcommands(window);
        };
        if let Some(window) = window.downcast_ref::<command_view::CommandWindow>() {
            self.copy_metacommands(window);
        }
    }

    fn paste_microcommands(&self, window: &MtemuWindow, data: Option<GString>) {
        let Some(model) = window.imp().code_view_pane.imp().code_list.model() else {
                return;
            };
            let Some(model) = model.downcast_ref::<gtk::MultiSelection>() else {
                return;
            };
            let selected = {
                let mut selected = None;
                for ind in 0..model.n_items() {
                    if model.is_selected(ind) {
                        selected = Some(ind);
                        break;
                    }
                }
                selected
            }
            .unwrap_or(0);
            let Some(string) = data else { return };
            let cmd_words = string
                .lines()
                .filter_map(|line| {
                    if line.len() != 40 || i32::from_str_radix(line, 2).is_err() {
                        return None;
                    }
                    let mut words = Vec::<i32>::with_capacity(10);
                    for i in (0..=line.len() - 4).step_by(4) {
                        words.push(i32::from_str_radix(&line[i..i + 4], 2).unwrap_or(0));
                    }
                    Some(words)
                })
                .collect::<Vec<Vec<i32>>>();
            let emul = self.get_emulator();
            let new_commands = imp::BoxedCommands({
                let Some(ref mut emul) = *emul.borrow_mut() else {
                    return;
                };
                cmd_words.into_iter().enumerate().for_each(|(ind, words)| {
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
            self.emit_by_name::<()>("commands-appeared", &[&new_commands]);
    }

    fn paste_metacommands(&self, window: &command_view::CommandWindow, data: Option<GString>) {

    }
    
    fn paste_commands(&self) {
        let window = self.active_window().unwrap();
        let app = self.clone();

        let Some(clipboard) = gtk::gdk::Display::default().and_then(|disp| Some(disp.clipboard()))
        else {
            return;
        };
        clipboard.read_text_async(gio::Cancellable::NONE, move |result| {
            if result.is_err() {
                return;
            }
            if let Some(window) = window.downcast_ref::<MtemuWindow>() {
                app.paste_microcommands(window, result.unwrap());
                return;
            };
            if let Some(window) = window.downcast_ref::<command_view::CommandWindow>() {
                app.paste_metacommands(window, result.unwrap());
                return;
            }
        });
    }

    fn toggle_commands(&self) {
        if let Some(commands_id) = *self.imp().commands_window.borrow() {
            if let Some(window) = self.window_by_id(commands_id) {
                self.remove_window(&window);
                window.destroy();
                return;
            }
        }
        let commands_window = {
            let window = command_view::CommandWindow::new(self);
            self.add_window(&window);
            self.imp().commands_window.replace(Some(window.id()));
            window
        };
        commands_window.present();
        let libcalls = imp::BoxedLibCalls({
            let emul = self.get_emulator();
            let Some(ref emul) = *emul.as_ref().borrow() else {
                return;
            };
            Rc::new(emul.get_map_calls())
        });
        self.emit_by_name::<()>("callslib-appeared", &[&libcalls]);
        let commands = imp::BoxedCalls({
            let emul = self.get_emulator();
            let Some(ref emul) = *emul.as_ref().borrow() else {
                return;
            };
            let call_count = emul.call_count();
            let mut calls = Vec::<emulator::Call>::with_capacity(call_count);
            for i in 0..call_count {
                calls.push(emul.get_call(i));
            }
            Rc::new(calls)
        });
        self.emit_by_name::<()>("calls-appeared", &[&commands]);
        self.imp().handle_command_buttons(commands_window);
    }

    pub fn set_emulator(&self, emul: emulator::OriginalImplementation) {
        self.imp().set_emulator(emul);
    }
    pub fn get_emulator(&self) -> utils::EmulatorStored {
        self.imp().get_emulator()
    }
    fn init_library(&self) {
        self.imp().init_library();
        let window = self.active_window().unwrap();
        window.emit_by_name::<()>("library-inited", &[]);
    }
}

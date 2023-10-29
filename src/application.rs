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
use std::io::prelude::*;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::Arc;

use gtk::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::ui::window::MtemuWindow;

mod imp {
    use std::{cell::RefCell, sync::Arc, rc::Rc};
    use gtk::glib::{once_cell::sync::Lazy, MainContext};

    use crate::emulator;

    use super::*;

    #[derive(Default)]
    pub struct MtemuApplication {
        emulator: Arc<RefCell<Option<Box<dyn crate::emulator::MT1804Emulator>>>>,
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
    #[shared_boxed_type(name = "BoxedState")]
    pub struct BoxedState(pub Rc<emulator::State>);

    impl ObjectImpl for MtemuApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            //self.connect_cmd_list();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![glib::subclass::Signal::builder("commands-appeared")
                     .param_types([BoxedCommands::static_type()])
                     .build(),
                     glib::subclass::Signal::builder("state-changed")
                     .param_types([BoxedState::static_type()])
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
            self.handle_debug_buttons();
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
        fn connect_command_appeared(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let code_cmd_list = window.imp().code_view_pane.clone();
            app.connect_closure("commands-appeared", false, glib::closure_local!(move |app: super::MtemuApplication, cmds: BoxedCommands| {
                let model = cmds.0
                    .iter()
                    .map(|cmd| { crate::ui::code_view_pane::CommandRepr::from_command(&app, cmd.clone()) })
                    .collect::<gio::ListStore>();
                code_cmd_list.imp().instance_model(model);
            }));
        }
        fn connect_state_changed(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let debug_pane = window.imp().debug_pane.clone();
            let code_cmd_list = window.imp().code_view_pane.clone();
            app.connect_closure("state-changed", false, glib::closure_local!(move |app: super::MtemuApplication, state: BoxedState| {
                debug_pane.renew_state(&state.0);
                let Some(selection) = code_cmd_list.imp().code_list.model() else { return };
                selection.unselect_all();
                selection.select_item(state.0.program_counter as u32, false);
            }));
        }
        fn handle_debug_buttons(&self) {
            let app = self.obj().clone();
            let Some(window) = app.active_window() else { return };
            let Some(window) = window.downcast_ref::<MtemuWindow>() else { return };
            let debug_pane = window.imp().debug_pane.clone();
            let button_view = window.imp().debug_pane.imp().stepping_view.imp().clone();
            let app_clone = app.clone();
            button_view.step_button.connect_clicked(move |_| {
                {
                    let emul = app_clone.get_emulator();
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    emul.exec_one();
                }
                let state = BoxedState({
                    let emul = app_clone.get_emulator();
                    let Some(ref emul) = *emul.borrow() else { todo!() };
                    Rc::new(emul.get_state())
                });
                app_clone.emit_by_name::<()>("state-changed", &[&state]);
            });
            let app_clone = app.clone();
            button_view.reset_button.connect_clicked(move |_| {
                {
                    let emul = app_clone.get_emulator();
                    let Some(ref mut emul) = *emul.borrow_mut() else { return };
                    emul.reset();
                }
                let state = BoxedState({
                    let emul = app_clone.get_emulator();
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
            });
            let app_clone = app.clone();
            button_view.run_button.connect_clicked(move |button: &gtk::Button| {
                let context = MainContext::default();
                let button_clone = button.clone();
                context.spawn_local(glib::clone!(@weak app_clone => async move {
                    button_clone.set_sensitive(false);
                    for _ in 0..200 {
                        {
                            let emul = app_clone.get_emulator();
                            let Some(ref mut emul) = *emul.borrow_mut() else { return };
                            emul.exec_one();
                        }
                        let state = BoxedState({
                            let emul = app_clone.get_emulator();
                            let Some(ref emul) = *emul.borrow() else { todo!() };
                            Rc::new(emul.get_state())
                        });
                        app_clone.emit_by_name::<()>("state-changed", &[&state]);
                        glib::timeout_future(std::time::Duration::from_millis(20)).await;
                    }
                    button_clone.set_sensitive(true);
                }));
            });
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
        self.add_action_entries([quit_action, about_action, open_file_action]);
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
    pub fn set_emulator(&self, emul: Box<dyn crate::emulator::MT1804Emulator>) {
        self.imp().set_emulator(emul);
    }
    pub fn get_emulator(&self) -> Arc<RefCell<Option<Box<dyn crate::emulator::MT1804Emulator>>>> {
        self.imp().get_emulator()
    }
}

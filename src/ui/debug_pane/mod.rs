/* debug_pane.rs
 *
 * Copyright 2023 Anton
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

mod stepping_view;
mod output_view;
mod register_view;

use adw::subclass::prelude::*;
use gtk::{gio, glib};
use stepping_view::SteppingView;
use output_view::OutputView;
use register_view::RegisterView;

use crate::emulator;

mod imp {
    use gtk::{prelude::*, glib::once_cell::sync::Lazy};
    use glib::subclass::Signal;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/debug_pane/pane.ui")]
    pub struct DebugPane {
        // Template widgets
        #[template_child]
        stepping_view: TemplateChild<SteppingView>,
        #[template_child]
        output_view: TemplateChild<OutputView>,
        #[template_child]
        register_view: TemplateChild<RegisterView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DebugPane {
        const NAME: &'static str = "DebugPane";
        type Type = super::DebugPane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DebugPane {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("reset-clicked")
                     .param_types([gtk::Button::static_type()])
                     .build(),
                     Signal::builder("step-clicked")
                     .param_types([gtk::Button::static_type()])
                     .build(),
                     Signal::builder("run-toggled")
                     .param_types([gtk::ToggleButton::static_type()])
                     .build()]
            });
            SIGNALS.as_ref()
        }
        fn constructed(&self) {
            self.parent_constructed();
            self.propagate_signals();
        }
    }
    impl WidgetImpl for DebugPane {}
    impl BoxImpl for DebugPane {}
    impl DebugPane {
        pub fn renew_state(&self, new_state: &emulator::State) {
            self.output_view.renew_state(new_state);
            self.register_view.renew_state(new_state);
        }
        fn propagate_signals(&self) {
            let pane = self.obj().clone();
            self.stepping_view.connect_closure("reset-clicked",
                                               false,
                                               glib::closure_local!(move |_: SteppingView, button: &gtk::Button| {
                                                   pane.emit_by_name::<()>("reset-clicked", &[button]);
                                               }));
            let pane = self.obj().clone();
            self.stepping_view.connect_closure("step-clicked",
                                               false,
                                               glib::closure_local!(move |_: SteppingView, button: &gtk::Button| {
                                                   pane.emit_by_name::<()>("step-clicked", &[button]);
                                               }));
            let pane = self.obj().clone();
            self.stepping_view.connect_closure("run-toggled",
                                               false,
                                               glib::closure_local!(move |_: SteppingView, button: &gtk::ToggleButton| {
                                                   pane.emit_by_name::<()>("run-toggled", &[button]);
                                               }));
        }
    }
}

glib::wrapper! {
    pub struct DebugPane(ObjectSubclass<imp::DebugPane>)
        @extends gtk::Widget,        @implements gio::ActionGroup, gio::ActionMap;
}

impl DebugPane {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        eprintln!("{:?}", application);
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn renew_state(&self, new_state: &emulator::State) {
        self.imp().renew_state(new_state);
    }
}

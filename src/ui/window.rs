/* window.rs
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

use adw::subclass::prelude::*;
use gtk::{gio, glib};

use super::code_view_pane::CodeViewPane;
use super::debug_pane::DebugPane;
use super::line_builder_pane::LineBuilderPane;

mod imp {

    use std::cell::Cell;

    use gtk::prelude::ObjectExt;

    use crate::application::MtemuApplication;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/window.ui")]
    pub struct MtemuWindow {
        #[template_child]
        pub instr_repr_sw: TemplateChild<gtk::Switch>,
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub code_view_pane: TemplateChild<CodeViewPane>,
        #[template_child]
        pub debug_pane: TemplateChild<DebugPane>,
        #[template_child]
        pub line_builder_pane: TemplateChild<LineBuilderPane>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MtemuWindow {
        const NAME: &'static str = "MtemuWindow";
        type Type = super::MtemuWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MtemuWindow {
        fn constructed(&self) {
            self.parent_constructed();

        }
    }
    impl WidgetImpl for MtemuWindow {}
    impl WindowImpl for MtemuWindow {}
    impl ApplicationWindowImpl for MtemuWindow {}
    impl AdwApplicationWindowImpl for MtemuWindow {}
}

glib::wrapper! {
    pub struct MtemuWindow(ObjectSubclass<imp::MtemuWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MtemuWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}


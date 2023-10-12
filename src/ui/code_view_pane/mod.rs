/* code_view_pane.rs
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

mod editor;

use adw::subclass::prelude::*;
use gtk::{gio, glib};
use editor::InstructionEditor;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/code_view_pane/pane.ui")]
    pub struct CodeViewPane {
        #[template_child]
        pub code_list: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub code_list_addr: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub code_list_command: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub code_list_jump: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub instruction_editor: TemplateChild<InstructionEditor>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CodeViewPane {
        const NAME: &'static str = "CodeViewPane";
        type Type = super::CodeViewPane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CodeViewPane {}
    impl WidgetImpl for CodeViewPane {}
    impl BoxImpl for CodeViewPane {}
}

glib::wrapper! {
    pub struct CodeViewPane(ObjectSubclass<imp::CodeViewPane>)
        @extends gtk::Widget,      @implements gio::ActionGroup, gio::ActionMap;
}

impl CodeViewPane {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}

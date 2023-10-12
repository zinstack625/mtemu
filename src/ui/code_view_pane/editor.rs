/* instruction_editor.rs
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

use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/code_view_pane/editor.ui")]
    pub struct InstructionEditor {}

    #[glib::object_subclass]
    impl ObjectSubclass for InstructionEditor {
        const NAME: &'static str = "InstructionEditor";
        type Type = super::InstructionEditor;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for InstructionEditor {}
    impl WidgetImpl for InstructionEditor {}
    impl BoxImpl for InstructionEditor {}
}

glib::wrapper! {
    pub struct InstructionEditor(ObjectSubclass<imp::InstructionEditor>)
        @extends gtk::Widget,      @implements gio::ActionGroup, gio::ActionMap;
}

impl InstructionEditor {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}

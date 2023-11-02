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

use crate::ui;

mod imp {
    use gtk::{prelude::{ObjectExt, EntryBufferExtManual}, traits::{EntryExt, EntryBufferExt, EditableExt}, glib::closure_local};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/code_view_pane/editor.ui")]
    pub struct InstructionEditor {
        #[template_child]
        addr: TemplateChild<gtk::Entry>,
        #[template_child]
        jump_type: TemplateChild<gtk::Entry>,
        #[template_child]
        load_type: TemplateChild<gtk::Entry>,
        #[template_child]
        op_type: TemplateChild<gtk::Entry>,
        #[template_child]
        instr_type: TemplateChild<gtk::Entry>,
        #[template_child]
        a_arg: TemplateChild<gtk::Entry>,
        #[template_child]
        b_arg: TemplateChild<gtk::Entry>,
        #[template_child]
        d_arg: TemplateChild<gtk::Entry>,
    }

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

    impl ObjectImpl for InstructionEditor {
        fn constructed(&self) {
            self.parent_constructed();
            self.limit_input_binary();
            self.addr.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>12b}", 0));
            self.jump_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
            self.load_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
            self.op_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
            self.instr_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
            self.a_arg.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
            self.b_arg.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
            self.d_arg.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", 0));
        }
    }
    impl WidgetImpl for InstructionEditor {}
    impl BoxImpl for InstructionEditor {}
    impl InstructionEditor {
        fn limit_input_binary(&self) {
            let limiter = closure_local!(move |field: gtk::Text, inserted: String, cnt: i32, pos: glib::value::Value| {
                let content = field.buffer().property::<String>("text").chars().filter(|c| { *c == '0' || *c == '1' }).collect::<String>();
                field.buffer().set_property("text", &content);
            });
            self.addr.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.jump_type.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.load_type.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.op_type.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.instr_type.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.a_arg.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.b_arg.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
            self.d_arg.delegate().unwrap().connect_closure("insert-text", true, limiter.clone());
        }
        pub fn get_codes(&self) -> [u8;10] {
            [
                u8::from_str_radix(&self.addr.buffer().property::<String>("text")[0..4], 2).unwrap_or(0),
                u8::from_str_radix(&self.addr.buffer().property::<String>("text")[4..8], 2).unwrap_or(0),
                u8::from_str_radix(&self.addr.buffer().property::<String>("text")[8..], 2).unwrap_or(0),
                u8::from_str_radix(&self.jump_type.buffer().property::<String>("text"), 2).unwrap_or(0),
                u8::from_str_radix(&self.load_type.buffer().property::<String>("text"), 2).unwrap_or(0),
                u8::from_str_radix(&self.op_type.buffer().property::<String>("text"), 2).unwrap_or(0),
                u8::from_str_radix(&self.instr_type.buffer().property::<String>("text"), 2).unwrap_or(0),
                u8::from_str_radix(&self.a_arg.buffer().property::<String>("text"), 2).unwrap_or(0),
                u8::from_str_radix(&self.b_arg.buffer().property::<String>("text"), 2).unwrap_or(0),
                u8::from_str_radix(&self.d_arg.buffer().property::<String>("text"), 2).unwrap_or(0),
            ]
        }
        pub fn renew_command(&self, cmd: &ui::line_builder_pane::CommandRepr) {
            self.addr.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>12b}", cmd.addr()));
            self.jump_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.jump()));
            self.load_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.load()));
            self.op_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.args()));
            self.instr_type.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.func()));
            self.a_arg.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.a_arg()));
            self.b_arg.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.b_arg()));
            self.d_arg.property::<gtk::EntryBuffer>("buffer").set_property("text", format!("{:0>4b}", cmd.d_arg()));
        }
    }
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
    pub fn renew_command(&self, cmd: &ui::line_builder_pane::CommandRepr) {
        self.imp().renew_command(&cmd);
    }
    pub fn get_codes(&self) -> [u8;10] {
        self.imp().get_codes()
    }
}

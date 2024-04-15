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

pub mod editor;
use adw::subclass::prelude::*;

use gtk::{gio, glib, prelude::ObjectExt};
use crate::emulator::MT1804Emulator;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{
        glib::Properties,
        prelude::{Cast, CastNone},
        traits::ListItemExt,
    };

    use crate::{application::MtemuApplication, emulator::Command};

    use super::{*, editor::InstructionEditor};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::CommandRepr)]
    pub struct CommandRepr {
        #[property(get, set)]
        addr: Cell<i32>,
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        jump: RefCell<String>,
        #[property(get, set)]
        binary: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommandRepr {
        const NAME: &'static str = "CommandRepr";
        type Type = super::CommandRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CommandRepr {}
    impl CommandRepr {
        pub fn from_command(app: &MtemuApplication, cmd: Command) -> Option<Self> {
            let emul = app.get_emulator();
            let Some(ref emul) = *emul.borrow() else { return None };
            let mut words = String::new();
            cmd.get_words().as_ref().unwrap().iter().for_each(|word| {
               words.push_str(&format!("{:0>4b} ", word & 0b1111));
            });
            Some(Self {
                addr: Cell::new(cmd.get_num() as i32),
                name: RefCell::new(emul.command_get_name(cmd.clone())),
                jump: RefCell::new(emul.command_get_jump_name(cmd.clone())),
                binary: RefCell::new(words),
            })
        }
    }

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/code_view_pane/pane.ui")]
    pub struct CodeViewPane {
        #[template_child]
        pub code_list: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub code_list_addr: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub code_list_command: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub code_list_command_binary: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub code_list_jump: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub instruction_editor: TemplateChild<InstructionEditor>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub update_button: TemplateChild<gtk::Button>,
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

    impl ObjectImpl for CodeViewPane {
        fn constructed(&self) {
            self.parent_constructed();
            self.instance_factories();
        }
    }
    impl WidgetImpl for CodeViewPane {}
    impl BoxImpl for CodeViewPane {}
    impl CodeViewPane {
        pub fn instance_model(&self, cmds: gio::ListStore) {
            self.code_list
                .set_model(Some(&gtk::MultiSelection::new(Some(
                    cmds
                ))))
        }
        pub fn get_codes(&self) -> [u8;10] {
                self.instruction_editor.get_codes()
        }
        pub fn show_binary(&self) {
            self.code_list_command_binary.set_visible(true);
        }
        pub fn hide_binary(&self) {
            self.code_list_command_binary.set_visible(false);
        }
        pub fn show_human(&self) {
            self.code_list_command.set_visible(true);
            self.code_list_jump.set_visible(true);
        }
        pub fn hide_human(&self) {
            self.code_list_command.set_visible(false);
            self.code_list_jump.set_visible(false);
        }
        fn instance_factories(&self) {
            self.code_list_addr.set_factory(Some(&{
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(move |_, obj| {
                    let Some(item) = obj.downcast_ref::<gtk::ListItem>() else {
                        return;
                    };
                    item.set_child(Some(&gtk::Label::builder().build()));
                });
                factory.connect_bind(move |_, obj| {
                    let Some(item) = obj.downcast_ref::<gtk::ListItem>() else {
                        return;
                    };
                    let Some(model) = item.item().and_downcast::<super::CommandRepr>() else {
                        return;
                    };
                    item.child()
                        .and_downcast::<gtk::Label>()
                        .unwrap()
                        .set_label(&format!("0x{:0>3X}", model.addr()));
                });
                factory
            }));
            self.code_list_command.set_factory(Some(&{
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(move |_, obj| {
                    let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    item.set_child(Some(&gtk::Label::builder().build()));
                });
                factory.connect_bind(move |_, obj| {
                    let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    let model = item
                        .item()
                        .and_downcast::<super::CommandRepr>()
                        .expect("Not a CommandRepr!");
                    item.child()
                        .and_downcast::<gtk::Label>()
                        .unwrap()
                        .set_label(&model.property::<String>("name"));
                });
                factory
            }));
            self.code_list_jump.set_factory(Some(&{
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(move |_, obj| {
                    let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    item.set_child(Some(&gtk::Label::builder().build()));
                });
                factory.connect_bind(move |_, obj| {
                    let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    let model = item
                        .item()
                        .and_downcast::<super::CommandRepr>()
                        .expect("Not a CommandRepr!");
                    item.child()
                        .and_downcast::<gtk::Label>()
                        .unwrap()
                        .set_label(&model.property::<String>("jump"));
                });
                factory
            }));
            self.code_list_command_binary.set_factory(Some(&{
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(move |_, obj| {
                    let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    item.set_child(Some(&gtk::Label::builder().build()));
                });
                factory.connect_bind(move |_, obj| {
                    let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    let model = item
                        .item()
                        .and_downcast::<super::CommandRepr>()
                        .expect("Not a CommandRepr");
                    item.child()
                        .and_downcast::<gtk::Label>()
                        .unwrap()
                        .set_label(&model.property::<String>("binary"));
                });
                factory
            }));
        }
    }
}

glib::wrapper! {
    pub struct CodeViewPane(ObjectSubclass<imp::CodeViewPane>)
        @extends gtk::Widget,      @implements gio::ActionGroup, gio::ActionMap;
}

impl CodeViewPane {
    pub fn show_binary(&self) {
        self.imp().show_binary();
    }
    pub fn hide_binary(&self) {
        self.imp().hide_binary();
    }
    pub fn show_human(&self) {
        self.imp().show_human();
    }
    pub fn hide_human(&self) {
        self.imp().hide_human();
    }
    pub fn get_codes(&self) -> [u8;10] {
        self.imp().get_codes()
    }
}

glib::wrapper! {
    pub struct CommandRepr(ObjectSubclass<imp::CommandRepr>);
}

impl CommandRepr {
    pub fn from_command(app: &crate::application::MtemuApplication, cmd: &crate::emulator::Command) -> Self {
        let emul = app.get_emulator();
        let Some(ref emul) = *emul.borrow() else { return glib::Object::builder().build() };
        let number = cmd.get_num();
        let name = emul.command_get_name(cmd.clone());
        let jump = emul.command_get_jump_name(cmd.clone());

        let mut binary = String::new();
        cmd.get_words().unwrap().iter().for_each(|word| {
            binary.push_str(&format!("{:0>4b} ", word & 0b1111));
        });
        glib::Object::builder()
            .property("addr", number as i32)
            .property("name", name)
            .property("jump", jump)
            .property("binary", binary)
            .build()
    }
}

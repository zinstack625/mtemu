/* command_view/mod.rs
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
use gtk::prelude::*;
use gtk::glib;

mod imp {
    use std::cell::Cell;
    use glib::Properties;
    use gtk::{prelude::{Cast, CastNone}, traits::ListItemExt};

    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::CommandValueRepr)]
    pub struct CommandValueRepr {
        #[property(get, set)]
        pub addr: Cell<u32>,
        #[property(get, set)]
        pub name: std::cell::RefCell<String>,
        #[property(get, set)]
        pub arg0: Cell<u8>,
        #[property(get, set)]
        pub arg1: Cell<u8>,
        #[property(get, set)]
        pub arg2: Cell<u8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommandValueRepr {
        const NAME: &'static str = "CommandValueRepr";
        type Type = super::CommandValueRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CommandValueRepr {}

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/command_view/window.ui")]
    pub struct CommandWindow {
        #[template_child]
        pub command_list: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub command_addr: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub command_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub command_arg_0: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub command_arg_1: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub command_arg_2: TemplateChild<gtk::ColumnViewColumn>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommandWindow {
        const NAME: &'static str = "CommandWindow";
        type Type = super::CommandWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CommandWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.instance_factories();
        }
    }
    impl WidgetImpl for CommandWindow {}
    impl WindowImpl for CommandWindow {}
    impl ApplicationWindowImpl for CommandWindow {}
    impl AdwApplicationWindowImpl for CommandWindow {}
    macro_rules! instance_factory {
        ($y:literal, $z:ident) => {{
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::CommandValueRepr>() else { return };
                obj.child()
                   .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!($y, item.$z()));
            });
            factory
        }}
    }
    impl CommandWindow {
        fn instance_factories(&self) {
            self.instance_addr_factory();
            self.instance_name_factory();
            self.instance_arg0_factory();
            self.instance_arg1_factory();
            self.instance_arg2_factory();
        }
        fn instance_addr_factory(&self) {
            self.command_addr.set_factory(Some(&instance_factory!("0x{:X}", addr)));
        }
        fn instance_name_factory(&self) {
            self.command_name.set_factory(Some(&instance_factory!("{}", name)));
        }
        fn instance_arg0_factory(&self) {
            self.command_arg_0.set_factory(Some(&instance_factory!("0b{:0>8b}", arg0)));
        }
        fn instance_arg1_factory(&self) {
            self.command_arg_1.set_factory(Some(&instance_factory!("0b:{:0>8b}", arg1)));
        }
        fn instance_arg2_factory(&self) {
            self.command_arg_2.set_factory(Some(&instance_factory!("0b{:0>8b}", arg2)));
        }
        pub fn set_commands(&self, commands: gtk::gio::ListStore) {
            self.command_list.set_model(Some(&gtk::SingleSelection::new(Some(commands))));
        }
        pub fn get_commands(&self) -> Vec<super::CommandValueRepr> {
            let Some(model) = self.command_list.model() else { return Vec::new() };
            model.iter().map(|elem: Result<super::CommandValueRepr, _>| {
                elem.expect("Not a CommandValueRepr")
            }).collect::<Vec<super::CommandValueRepr>>()
        }
        pub fn get_command(&self, ind: usize) -> Option<super::CommandValueRepr> {
            let Some(model) = self.command_list.model() else { return None };
            model.item(ind as u32).and_downcast::<super::CommandValueRepr>()
        }
    }
}

glib::wrapper! {
    pub struct CommandWindow(ObjectSubclass<imp::CommandWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl CommandWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn set_commands(&self, commands: gtk::gio::ListStore) {
        self.imp().set_commands(commands);
    }
    pub fn get_commands(&self) -> Vec<CommandValueRepr> {
        self.imp().get_commands()
    }
    pub fn get_command(&self, ind: usize) -> Option<CommandValueRepr> {
        self.imp().get_command(ind)
    }
}

glib::wrapper! {
    pub struct CommandValueRepr(ObjectSubclass<imp::CommandValueRepr>);
}

impl CommandValueRepr {
    pub fn new(addr: u32, name: String, arg0: u8, arg1: u8, arg2: u8) -> Self {
        glib::Object::builder()
            .property("addr", addr)
            .property("name", name)
            .property("arg0", arg0)
            .property("arg1", arg1)
            .property("arg2", arg2)
            .build()
    }
}

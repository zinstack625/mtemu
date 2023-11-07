/* memory_view/mod.rs
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
    #[properties(wrapper_type = super::MemoryValueRepr)]
    pub struct MemoryValueRepr {
        #[property(get, set)]
        pub addr: Cell<u32>,
        #[property(get, set)]
        pub value: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MemoryValueRepr {
        const NAME: &'static str = "MemoryValueRepr";
        type Type = super::MemoryValueRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MemoryValueRepr {}

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/memory_view/window.ui")]
    pub struct MemoryWindow {
        #[template_child]
        pub memory_list: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub memory_addr: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub memory_bin: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub memory_hex: TemplateChild<gtk::ColumnViewColumn>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MemoryWindow {
        const NAME: &'static str = "MemoryWindow";
        type Type = super::MemoryWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MemoryWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.instance_factories();
        }
    }
    impl WidgetImpl for MemoryWindow {}
    impl WindowImpl for MemoryWindow {}
    impl ApplicationWindowImpl for MemoryWindow {}
    impl AdwApplicationWindowImpl for MemoryWindow {}
    impl MemoryWindow {
        fn instance_factories(&self) {
            self.instance_addr_factory();
            self.instance_bin_factory();
            self.instance_hex_factory();
        }
        fn instance_addr_factory(&self) {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::MemoryValueRepr>() else { return };
                obj.child()
                    .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!("0x{:0>2X}", item.addr()));
            });
            self.memory_addr.set_factory(Some(&factory));
        }
        fn instance_bin_factory(&self) {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::MemoryValueRepr>() else { return };
                obj.child()
                    .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!("0b{:0>8b}", item.value()));
            });
            self.memory_bin.set_factory(Some(&factory));
        }
        fn instance_hex_factory(&self) {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::MemoryValueRepr>() else { return };
                obj.child()
                    .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!("0b{:0>2X}", item.value()));
            });
            self.memory_hex.set_factory(Some(&factory));
        }
        pub fn set_memory(&self, memory: gtk::gio::ListStore) {
            self.memory_list.set_model(Some(&gtk::SingleSelection::new(Some(memory))));
        }
    }
}

glib::wrapper! {
    pub struct MemoryWindow(ObjectSubclass<imp::MemoryWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl MemoryWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn set_memory(&self, memory: gtk::gio::ListStore) {
        self.imp().set_memory(memory);
    }
}

glib::wrapper! {
    pub struct MemoryValueRepr(ObjectSubclass<imp::MemoryValueRepr>);
}
impl MemoryValueRepr {
    pub fn new(addr: u32, value: u32) -> Self {
        glib::Object::builder()
            .property("addr", addr)
            .property("value", value)
            .build()
    }
}

/* stack_view/mod.rs
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
    #[properties(wrapper_type = super::StackValueRepr)]
    pub struct StackValueRepr {
        #[property(get, set)]
        pub addr: Cell<u32>,
        #[property(get, set)]
        pub pointer: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StackValueRepr {
        const NAME: &'static str = "StackValueRepr";
        type Type = super::StackValueRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for StackValueRepr {}

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/stack_view/window.ui")]
    pub struct StackWindow {
        #[template_child]
        pub stack_list: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub stack_addr: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub stack_pointer: TemplateChild<gtk::ColumnViewColumn>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StackWindow {
        const NAME: &'static str = "StackWindow";
        type Type = super::StackWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StackWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.instance_factories();
        }
    }
    impl WidgetImpl for StackWindow {}
    impl WindowImpl for StackWindow {}
    impl ApplicationWindowImpl for StackWindow {}
    impl AdwApplicationWindowImpl for StackWindow {}
    impl StackWindow {
        fn instance_factories(&self) {
            self.instance_addr_factory();
            self.instance_pointer_factory();
        }
        fn instance_addr_factory(&self) {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::StackValueRepr>() else { return };
                obj.child()
                    .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!("0x{:X}", item.addr()));
            });
            self.stack_addr.set_factory(Some(&factory));
        }
        fn instance_pointer_factory(&self) {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::StackValueRepr>() else { return };
                obj.child()
                    .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!("0b{:0>4b}", item.pointer()));
            });
            self.stack_pointer.set_factory(Some(&factory));
        }
        pub fn set_stack(&self, stack: gtk::gio::ListStore) {
            self.stack_list.set_model(Some(&gtk::SingleSelection::new(Some(stack))));
        }
    }
}

glib::wrapper! {
    pub struct StackWindow(ObjectSubclass<imp::StackWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl StackWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn set_stack(&self, stack: gtk::gio::ListStore) {
        self.imp().set_stack(stack);
    }
}

glib::wrapper! {
    pub struct StackValueRepr(ObjectSubclass<imp::StackValueRepr>);
}
impl StackValueRepr {
    pub fn new(addr: u32, pointer: u32) -> Self {
        glib::Object::builder()
            .property("addr", addr)
            .property("pointer", pointer)
            .build()
    }
}

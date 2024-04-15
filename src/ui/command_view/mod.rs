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
use gtk::glib::Properties;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod call_imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::CallValueRepr)]
    pub struct CallValueRepr {
        #[property(get, set)]
        addr: Cell<u32>,
        #[property(get, set)]
        code: Cell<u32>,
        #[property(get, set)]
        arg0: Cell<u8>,
        #[property(get, set)]
        arg1: Cell<u8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CallValueRepr {
        const NAME: &'static str = "CallValueRepr";
        type Type = super::CallValueRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CallValueRepr {}


}

mod libcall_imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::LibCallValueRepr)]
    pub struct LibCallValueRepr {
        #[property(get, set)]
        code: Cell<u32>,
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        addr: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LibCallValueRepr {
        const NAME: &'static str = "LibCallValueRepr";
        type Type = super::LibCallValueRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for LibCallValueRepr {}
}

mod imp {

    use std::rc::Rc;

    use gtk::{prelude::{Cast, CastNone}, traits::ListItemExt, glib::{once_cell::sync::Lazy, subclass::Signal}};

    use super::{*, BoxedCalls};



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
        pub library_table: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub library_ca: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub library_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub library_addr: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub update_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub step_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub run_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub reset_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub lib_add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub lib_delete_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub arg0_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub arg1_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub libname_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub libaddr_entry: TemplateChild<gtk::Entry>,
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
            self.limit_input_binary();
            let pane = self.obj().clone();
            self.add_button.connect_clicked(move |_: &gtk::Button| {
                let Some(command) = pane.imp().get_last_selected_command() else { return };
                let model = pane.imp().command_list.model();
                let cur_addr = {
                    if model.is_none() || model.as_ref().unwrap().n_items() == 0 {
                        0
                    } else {
                        command.addr() + 1
                    }
                };
                command.set_addr(cur_addr);
                pane.emit_by_name::<()>("add-clicked", &[&command]);
            });
            let pane = self.obj().clone();
            self.update_button.connect_clicked(move |_: &gtk::Button| {
                let Some(command) = pane.imp().get_last_selected_command() else { return };
                pane.emit_by_name::<()>("update-clicked", &[&command]);
            });
            let pane = self.obj().clone();
            self.delete_button.connect_clicked(move |_: &gtk::Button| {
                let commands = BoxedCalls(Rc::new(pane.imp().get_selected_commands()));
                pane.emit_by_name::<()>("delete-clicked", &[&commands]);
            });
            let pane = self.obj().clone();
            self.step_button.connect_clicked(move |_: &gtk::Button| {
                pane.emit_by_name::<()>("step-clicked", &[]);
            });
            let pane = self.obj().clone();
            self.run_button.connect_toggled(move |but: &gtk::ToggleButton| {
                pane.emit_by_name::<()>("run-toggled", &[&but]);
            });
            let pane = self.obj().clone();
            self.reset_button.connect_clicked(move |_: &gtk::Button| {
                pane.emit_by_name::<()>("reset-clicked", &[]);
            });
            let pane = self.obj().clone();
            self.lib_add_button.connect_clicked(move |_: &gtk::Button| {
                let name = pane.imp().libname_entry.buffer().text().to_string();
                let addr = i32::from_str_radix(&pane.imp().libaddr_entry.buffer().text(), 2).unwrap_or_default();
                let code = {
                    let model = pane.imp().library_table.model();
                    if model.is_none() || model.as_ref().unwrap().n_items() == 0 {
                        0
                    } else {
                        let mut next_code = 0;
                        let model = model.unwrap();
                        let selection = model.downcast_ref::<gtk::SingleSelection>().unwrap();
                        let mut taken = Vec::<i32>::with_capacity(selection.n_items() as usize);
                        let mut max = 0;
                        for i in 0..selection.n_items() {
                            let libcall = selection.item(i).and_downcast::<super::LibCallValueRepr>().unwrap();
                            taken.push(libcall.code() as i32);
                            if max < libcall.code() {
                                max = libcall.code();
                            }
                        }
                        for i in 0..=max+1 {
                            if !taken.contains(&(i as i32)) {
                                next_code = i as i32;
                            }
                        }
                        next_code
                    }
                };
                pane.emit_by_name::<()>("lib-add-clicked", &[&code, &name, &addr]);
            });
            let pane = self.obj().clone();
            self.lib_delete_button.connect_clicked(move |_: &gtk::Button| {
                let Some(model) = pane.imp().library_table.model().and_downcast::<gtk::SingleSelection>() else { return };
                let Some(item) = model.item(model.selected()).and_downcast::<super::LibCallValueRepr>() else { return };
                pane.emit_by_name::<()>("lib-delete-clicked", &[&(item.code() as i32)]);
            });
        }
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(| | {
                vec![Signal::builder("add-clicked")
                     .param_types([CallValueRepr::static_type()])
                     .build(),
                     Signal::builder("update-clicked")
                     .param_types([CallValueRepr::static_type()])
                     .build(),
                     Signal::builder("delete-clicked")
                    .param_types([BoxedCalls::static_type()])
                     .build(),
                     Signal::builder("step-clicked").build(),
                     Signal::builder("run-toggled")
                     .param_types([gtk::ToggleButton::static_type()])
                     .build(),
                     Signal::builder("reset-clicked").build(),
                     Signal::builder("lib-add-clicked")
                     .param_types([i32::static_type(), String::static_type(), i32::static_type()])
                     .build(),
                     Signal::builder("lib-delete-clicked")
                     .param_types([i32::static_type()])
                     .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for CommandWindow {}
    impl WindowImpl for CommandWindow {}
    impl ApplicationWindowImpl for CommandWindow {}
    impl AdwApplicationWindowImpl for CommandWindow {}
    macro_rules! call_instance_factory {
        ($y:literal, $z:ident) => {{
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::CallValueRepr>() else { return };
                obj.child()
                   .and_downcast_ref::<gtk::Label>()
                    .unwrap()
                    .set_label(&format!($y, item.$z()));
            });
            factory
        }}
    }
    macro_rules! libcall_instance_factory {
        ($y:literal, $z:ident) => {{
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let Some(item) = obj.item().and_downcast::<super::LibCallValueRepr>() else { return };
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
            self.instance_code_factory();
            self.instance_name_factory();
            self.instance_arg0_factory();
            self.instance_arg1_factory();
            self.instance_libca_factory();
            self.instance_libname_factory();
            self.instance_libaddr_factory();
        }
        fn instance_code_factory(&self) {
            self.command_addr.set_factory(Some(&call_instance_factory!("0x{:X}", addr)));
        }
        fn instance_name_factory(&self) {
            self.command_name.set_factory(Some(&{
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(move |_, obj| {
                    let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    obj.set_child(Some(&gtk::Label::builder().build()));
                });
                let self_clone = self.obj().clone();
                factory.connect_bind(move |_, obj| {
                    let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                    let Some(item) = obj.item().and_downcast::<super::CallValueRepr>() else { return };
                    let code = item.code();
                    let Some(model) = self_clone.imp().library_table.model() else { return };
                    let Some(call) = model.item(code).and_downcast::<super::LibCallValueRepr>() else { return };
                    obj.child()
                        .and_downcast_ref::<gtk::Label>()
                        .unwrap()
                        .set_label(&format!("{}", call.name()));
                });
                factory
            }));
        }
        fn instance_arg0_factory(&self) {
            self.command_arg_0.set_factory(Some(&call_instance_factory!("0b{:0>8b}", arg0)));
        }
        fn instance_arg1_factory(&self) {
            self.command_arg_1.set_factory(Some(&call_instance_factory!("0b{:0>8b}", arg1)));
        }
        pub fn set_commands(&self, commands: gtk::gio::ListStore) {
            self.command_list.set_model(Some(&gtk::MultiSelection::new(Some(commands))));
            let pane_clone = self.obj().clone();
            self.command_list.model().as_ref().unwrap().connect_selection_changed(move |sel: &gtk::SelectionModel, pos: u32, cnt: u32| {
                let pos = {
                    match cnt {
                        0 => return,
                        1 => pos,
                        _ => {
                            let mut selected = None;
                            for i in 0..sel.n_items() {
                                if sel.is_selected(i) {
                                    selected = Some(i);
                                    break;
                                }
                            }
                            selected.unwrap_or_default()
                        },
                    }
                };
                let Some(call) = sel.item(pos).and_downcast::<CallValueRepr>() else { return };
                pane_clone.select_lib(call.code());
                pane_clone.fill_args(call.arg0(), call.arg1());
            });
        }
        pub fn get_commands(&self) -> Vec<super::CallValueRepr> {
            let Some(model) = self.command_list.model() else { return Vec::new() };
            model.iter().map(|elem: Result<super::CallValueRepr, _>| {
                elem.expect("Not a CommandValueRepr")
            }).collect::<Vec<super::CallValueRepr>>()
        }
        pub fn get_command(&self, ind: usize) -> Option<super::CallValueRepr> {
            let Some(model) = self.command_list.model() else { return None };
            model.item(ind as u32).and_downcast::<super::CallValueRepr>()
        }
        pub fn set_command_library(&self, commands: gtk::gio::ListStore) {
            self.library_table.set_model(Some(&gtk::SingleSelection::new(Some(commands))));
        }
        fn instance_libca_factory(&self) {
            self.library_ca.set_factory(Some(&libcall_instance_factory!("0x{:0>4X}", code)));
        }
        fn instance_libname_factory(&self) {
            self.library_name.set_factory(Some(&libcall_instance_factory!("{}", name)));
        }
        fn instance_libaddr_factory(&self) {
            self.library_addr.set_factory(Some(&libcall_instance_factory!("0x{:0>4X}", addr)));
        }
        fn get_last_selected_command(&self) -> Option<super::CallValueRepr> {
            let Some(model) = self.library_table.model() else { return None };
            let selected = model.downcast_ref::<gtk::SingleSelection>().unwrap().selected();
            let Some(call) = model.item(selected).and_downcast::<super::LibCallValueRepr>() else { return None };
            let index = {
                let model = self.command_list.model();
                if model.is_none() || model.as_ref().unwrap().n_items() == 0 {
                    0
                } else {
                    let model = model.unwrap();
                    let selection = model.downcast_ref::<gtk::MultiSelection>().unwrap();
                    let mut selected = 0;
                    for i in 0..selection.n_items() {
                        if selection.is_selected(i) { selected = i; }
                    }
                    selected
                }
            };
            let arg0 = u8::from_str_radix(self.arg0_entry.buffer().text().as_str(), 2).unwrap_or(0);
            let arg1 = u8::from_str_radix(self.arg1_entry.buffer().text().as_str(), 2).unwrap_or(0);
            Some(super::CallValueRepr::new(
                index, call.code(), arg0, arg1
            ))
        }
        pub fn get_selected_commands(&self) -> Vec<super::CallValueRepr> {
            let Some(model) = self.command_list.model() else { return Vec::new() };
            let selected = model.downcast_ref::<gtk::MultiSelection>().unwrap();
            let mut result = Vec::<super::CallValueRepr>::with_capacity(selected.n_items() as usize);
            if model.n_items() == 0 {
                return result;
            }
            for i in 0..selected.n_items() {
                if selected.is_selected(i) {
                    result.push(selected.item(i).unwrap().downcast::<super::CallValueRepr>().unwrap());
                }
            }
            return result;
        }
        pub fn set_call_index(&self, ind: u32) {
            let Some(model) = self.command_list.model() else { return };
            model.select_item(ind, true);
        }
        pub fn select_lib(&self, code: u32) {
            let Some(model) = self.library_table.model() else { return };
            let mut final_item = None;
            for i in 0..model.n_items() {
                let Some(libcall) = model.item(i).and_downcast::<LibCallValueRepr>() else { continue };
                if libcall.code() == code {
                    final_item = Some(libcall);
                    model.select_item(i, true);
                    break;
                }
            }
            let Some(final_item) = final_item else { return };
            self.libname_entry.buffer().set_text(final_item.name());
            self.libaddr_entry.buffer().set_text(format!("{:0>8b}", final_item.addr()));
        }
        pub fn fill_args(&self, arg0: u8, arg1: u8) {
            self.arg0_entry.buffer().set_text(format!("{:0>8b}", arg0));
            self.arg1_entry.buffer().set_text(format!("{:0>8b}", arg1));
        }
        fn limit_input_binary(&self) {

            let limiter = move |field: &gtk::Editable, inserted: &str, _: &mut i32| {
                if inserted.chars().any(|c| { c != '0' && c != '1' }) {
                    field.stop_signal_emission_by_name("insert-text");
                }
            };
            self.arg0_entry.delegate().unwrap().connect_insert_text(limiter.clone());
            self.arg1_entry.delegate().unwrap().connect_insert_text(limiter.clone());
            self.libaddr_entry.delegate().unwrap().connect_insert_text(limiter.clone());
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
    pub fn get_commands(&self) -> Vec<CallValueRepr> {
        self.imp().get_commands()
    }
    pub fn get_command(&self, ind: usize) -> Option<CallValueRepr> {
        self.imp().get_command(ind)
    }
    pub fn set_command_library(&self, commands: gtk::gio::ListStore) {
        self.imp().set_command_library(commands);
    }
    pub fn set_call_index(&self, ind: u32) {
        self.imp().set_call_index(ind);
    }
    fn select_lib(&self, code: u32) {
        self.imp().select_lib(code);
    }
    fn fill_args(&self, arg0: u8, arg1: u8) {
        self.imp().fill_args(arg0, arg1);
    }
}

glib::wrapper! {
    pub struct CallValueRepr(ObjectSubclass<call_imp::CallValueRepr>);
}

impl CallValueRepr {
    pub fn new(addr: u32, code: u32, arg0: u8, arg1: u8) -> Self {
        glib::Object::builder()
            .property("addr", addr)
            .property("code", code)
            .property("arg0", arg0)
            .property("arg1", arg1)
            .build()
    }
}

#[derive(glib::SharedBoxed, Clone, Debug)]
#[shared_boxed_type(name = "BoxedCallsRepr")]
pub struct BoxedCalls(pub Rc<Vec<CallValueRepr>>);

glib::wrapper! {
    pub struct LibCallValueRepr(ObjectSubclass<libcall_imp::LibCallValueRepr>);
}

impl LibCallValueRepr {
    pub fn new(code: u32, name: &str, addr: u32) -> Self {
        glib::Object::builder()
            .property("code", code)
            .property("name", name.to_string())
            .property("addr", addr)
            .build()
    }
}

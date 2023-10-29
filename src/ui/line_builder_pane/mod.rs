/* line_builder_pane.rs
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

mod models;

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};
use gtk::gio::functions;
use gtk::gio::ResourceLookupFlags;
use models::*;

macro_rules! col_static_factory {
    ($x:ty, $y:ident) => {
        {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                obj.set_child(Some(&gtk::Label::builder().build()));
            });
            factory.connect_bind(move |_, obj| {
                let obj = obj.downcast_ref::<gtk::ListItem>().unwrap();
                let item = obj.item()
                              .and_downcast::<$x>()
                    .expect("Invalid type in model!");
                obj.child()
                   .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&item.$y());
            });
            factory
        }
    };
}

macro_rules! parse_res {
    ($x:ident, $r:expr, $(($y:literal, $z:ident)),*) => {
        serde_json::from_slice::<Vec<HashMap<String, String>>>(
            &functions::resources_lookup_data(
                $r,
                ResourceLookupFlags::NONE
            )
            .unwrap()
            .to_vec()
        )
        .unwrap()
        .into_iter().map(|map| {
            let entry = $x::new();
            $(
                entry.$z(map.get($y).expect("Invalid format"));
            )*
            entry
        })
        .collect::<gio::ListStore>()
    };
}

mod imp {
    use std::collections::HashMap;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/line_builder_pane/pane.ui")]
    pub struct LineBuilderPane {
        #[template_child]
        pub jump_type: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub jump_type_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub jump_type_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub alu_instr_type: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub instr_type_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub instr_type_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub instr_type_add_bit: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub pointer_type: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub pointer_type_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub pointer_type_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub interface_type: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub interface_type_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub interface_type_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub pointer_size: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub pointer_size_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub pointer_size_name: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub op_type: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub op_type_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub op_type_r: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub op_type_s: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub load_type: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub load_type_code: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub load_type_name: TemplateChild<gtk::ColumnViewColumn>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LineBuilderPane {
        const NAME: &'static str = "LineBuilderPane";
        type Type = super::LineBuilderPane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LineBuilderPane {
        fn constructed(&self) {
            self.prepare_jump_type_table("/org/bmstu/mtemu/ui/line_builder_pane/jump_table_entries.json");
            self.prepare_instr_type_table("/org/bmstu/mtemu/ui/line_builder_pane/instr_table_entries.json");
            self.prepare_pointer_type_table("/org/bmstu/mtemu/ui/line_builder_pane/pointer_type_table_entries.json");
            self.prepare_interface_type_table("/org/bmstu/mtemu/ui/line_builder_pane/interface_type_table_entries.json");
            self.prepare_pointer_size_table("/org/bmstu/mtemu/ui/line_builder_pane/pointer_size_table_entries.json");
            self.prepare_operand_type_table("/org/bmstu/mtemu/ui/line_builder_pane/operand_type_table_entries.json");
            self.prepare_load_type_table("/org/bmstu/mtemu/ui/line_builder_pane/load_type_table_entries.json");
        }
    }
    impl WidgetImpl for LineBuilderPane {}
    impl BoxImpl for LineBuilderPane {}
    impl LineBuilderPane {
        fn prepare_jump_type_table(&self, from_resource: &str) {
            self.jump_type.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(JumpEntry, from_resource,
                        ("ca", set_ca),
                        ("jump", set_jump)
                    ))
                ))
            );
            self.jump_type_code.set_factory(Some(&col_static_factory!(JumpEntry, get_ca)));
            self.jump_type_name.set_factory(Some(&col_static_factory!(JumpEntry, get_jump)));
        }
        fn prepare_instr_type_table(&self, from_resource: &str) {
            self.alu_instr_type.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(InstrEntry, from_resource,
                        ("ca", set_ca),
                        ("func", set_func),
                        ("bit", set_bit)
                    ))
                ))
            );
            self.instr_type_code.set_factory(Some(&col_static_factory!(InstrEntry, get_ca)));
            self.instr_type_name.set_factory(Some(&col_static_factory!(InstrEntry, get_func)));
            self.instr_type_add_bit.set_factory(Some(&col_static_factory!(InstrEntry, get_bit)));
        }
        fn prepare_pointer_type_table(&self, from_resource: &str) {
            self.pointer_type.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(PointerTypeEntry, from_resource,
                        ("ca", set_ca),
                        ("ptr", set_ptr)
                    ))
                ))
            );
            self.pointer_type_code.set_factory(Some(&col_static_factory!(PointerTypeEntry, get_ca)));
            self.pointer_type_name.set_factory(Some(&col_static_factory!(PointerTypeEntry, get_ptr)));
        }
        fn prepare_interface_type_table(&self, from_resource: &str) {
            self.interface_type.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(InterfaceEntry, from_resource,
                        ("ca", set_ca),
                        ("interface", set_interface)
                    ))
                ))
            );
            self.interface_type_code.set_factory(Some(&col_static_factory!(InterfaceEntry, get_ca)));
            self.interface_type_name.set_factory(Some(&col_static_factory!(InterfaceEntry, get_interface)));
        }
        fn prepare_pointer_size_table(&self, from_resource: &str) {
            self.pointer_size.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(PointerSizeEntry, from_resource,
                        ("ca", set_ca),
                        ("size", set_size)
                    ))
                ))
            );
            self.pointer_size_code.set_factory(Some(&col_static_factory!(PointerSizeEntry, get_ca)));
            self.pointer_size_name.set_factory(Some(&col_static_factory!(PointerSizeEntry, get_size)));
        }
        fn prepare_operand_type_table(&self, from_resource: &str) {
            self.op_type.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(OperandTypeEntry, from_resource,
                        ("ca", set_ca),
                        ("r", set_r),
                        ("s", set_s)
                    ))
                ))
            );
            self.op_type_code.set_factory(Some(&col_static_factory!(OperandTypeEntry, get_ca)));
            self.op_type_r.set_factory(Some(&col_static_factory!(OperandTypeEntry, get_r)));
            self.op_type_s.set_factory(Some(&col_static_factory!(OperandTypeEntry, get_s)));
        }
        fn prepare_load_type_table(&self, from_resource: &str) {
            self.load_type.set_model(
                Some(&gtk::SingleSelection::new(
                    Some(parse_res!(LoadTypeEntry, from_resource,
                        ("ca", set_ca),
                        ("load", set_load)
                    ))
                ))
            );
            self.load_type_code.set_factory(Some(&col_static_factory!(LoadTypeEntry, get_ca)));
            self.load_type_name.set_factory(Some(&col_static_factory!(LoadTypeEntry, get_load)));
        }
    }
}

glib::wrapper! {
    pub struct LineBuilderPane(ObjectSubclass<imp::LineBuilderPane>)
        @extends gtk::Widget,      @implements gio::ActionGroup, gio::ActionMap;
}

impl LineBuilderPane {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}

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

use crate::emulator;

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
    use std::{collections::HashMap, cell::Cell};

    use gtk::{
        glib::Properties,
        prelude::*,
    };

    use super::*;

    // enum JumpType {
    //     JNZ,
    //     JMP,
    //     JNXT,
    //     END,
    //     CLNZ,
    //     CALL,
    //     RET,
    //     JSP,
    //     JSNZ,
    //     PUSH,
    //     POP,
    //     JSNC4,
    //     JZ,
    //     JF3,
    //     JOVR,
    //     JC4,
    //     Unknown = 255,
    // }
    // enum FlagType {
    //     M0,
    //     M1,
    //     C0,
    //     Unknown = 255,
    // }
    // enum FuncType {
    //     RPlusS = 0,
    //     SMinusRMinus1 = 1,
    //     RMinusSMinus1 = 2,
    //     ROrS = 3,
    //     RAndS = 4,
    //     NoRAndS = 5,
    //     RXorS = 6,
    //     REqS = 7,
    //     RPlusSPlus1 = 8,
    //     SMinusR = 9,
    //     RMinusS = 10,
    //     SetPointer = 11,
    //     StoreMemory = 12,
    //     LoadMemory = 13,
    //     StoreDevice = 14,
    //     LoadDevice = 15,
    //     Unknown = 255,
    // }
    // enum FuncArgs {
    //     AAndPQ,
    //     AAndB,
    //     ZeroAndQ,
    //     ZeroAndB,
    //     ZeroAndA,
    //     DAndA,
    //     DAndQ,
    //     DAndZero,
    //     Unknown = 255,
    // }
    // enum FuncLoad {
    //     FInQ,
    //     NoLoad,
    //     FInBAndAInY,
    //     FInB,
    //     SRFInBAndSRQInQ,
    //     SRFInB,
    //     SLFInBAndSLQInQ,
    //     SLFInB,
    //     Unknown = 255,
    // }

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::CommandRepr)]
    pub struct CommandRepr {
        #[property(get, set)]
        pub addr: Cell<u32>,
        #[property(get, set)]
        pub jump: Cell<u8>,
        #[property(get, set)]
        pub flag: Cell<u8>,
        #[property(get, set)]
        pub func: Cell<u8>,
        #[property(get, set)]
        pub args: Cell<u8>,
        #[property(get, set)]
        pub load: Cell<u8>,
        #[property(get, set)]
        pub pointer: Cell<u8>,
        #[property(get, set)]
        pub pointer_size: Cell<u8>,
        #[property(get, set)]
        pub a_arg: Cell<u8>,
        #[property(get, set)]
        pub b_arg: Cell<u8>,
        #[property(get, set)]
        pub d_arg: Cell<u8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommandRepr {
        const NAME: &'static str = "EditCommandRepr";
        type Type = super::CommandRepr;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CommandRepr {}

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
        pub fn get_command(&self) -> super::CommandRepr {
            let cmd = super::CommandRepr::new();
            let Some(selection) = self.jump_type.model() else { return cmd; };
            cmd.set_jump(selection.downcast_ref::<gtk::SingleSelection>().unwrap().selected() as u8);
            let Some(selection) = self.alu_instr_type.model() else { return cmd; };
            cmd.set_func(selection.downcast_ref::<gtk::SingleSelection>().unwrap().selected() as u8);
            let Some(selection) = self.pointer_type.model() else { return cmd; };
            cmd.set_pointer(selection.downcast_ref::<gtk::SingleSelection>().unwrap().selected() as u8);
            let Some(selection) = self.op_type.model() else { return cmd; };
            cmd.set_args(selection.downcast_ref::<gtk::SingleSelection>().unwrap().selected() as u8);
            let Some(selection) = self.load_type.model() else { return cmd; };
            cmd.set_load(selection.downcast_ref::<gtk::SingleSelection>().unwrap().selected() as u8);
            let Some(selection) = self.pointer_size.model() else { return cmd; };
            cmd.set_pointer_size(selection.downcast_ref::<gtk::SingleSelection>().unwrap().selected() as u8);
            cmd
        }
        pub fn renew_command(&self, new_command: &CommandRepr) {
            let Some(selection) = self.jump_type.model() else { return };
            selection.select_item(new_command.jump.get() as u32, true);
            let Some(selection) = self.alu_instr_type.model() else { return };
            selection.select_item(new_command.func.get() as u32, true);
            // let Some(selection) = self.pointer_type.model() else { return };
            // selection.select_item(, unselect_rest)
            let Some(selection) = self.op_type.model() else { return };
            selection.select_item(new_command.args.get() as u32, true);
            let Some(selection) = self.load_type.model() else { return };
            selection.select_item(new_command.load.get() as u32, true);
            let Some(selection) = self.pointer_type.model() else { return };
            selection.select_item(new_command.pointer.get() as u32, true);
            let Some(selection) = self.pointer_size.model() else { return };
            selection.select_item(new_command.pointer_size.get() as u32, true);
        }
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

glib::wrapper! {
    pub struct CommandRepr(ObjectSubclass<imp::CommandRepr>);
}

impl CommandRepr {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
    pub fn from_command(cmd: emulator::Command) -> Self {
        let words = cmd.get_words().unwrap();
        glib::Object::builder()
            .property("addr", (words[2] & 0b1111) | ((words[1] & 0b1111) << 4) | ((words[0] & 0b1111) << 8))
            .property("jump", words[3] as u8)
            .property("flag", words[4] as u8)
            .property("func", words[5] as u8)
            .property("args", words[6] as u8)
            .property("load", words[4] as u8)
            .property("pointer", {
                let mut res = words[5];
                if words[5] == 8 { res = 3; }
                res as u8
            }) // 8 => 3 && <3 || -1
            .property("pointer-size", {
                words[5] as u8
            }) // <3 || -1
            .property("a-arg", words[7] as u8)
            .property("b-arg", words[8] as u8)
            .property("d-arg", words[9] as u8)
            .build()
    }
    pub fn get_words(&self) -> [i32;12]  {
        [
            ((self.addr() >> 8) & 0b1111) as i32,
            ((self.addr() >> 4) & 0b1111) as i32,
            (self.addr() & 0b1111) as i32,
            self.jump() as i32,
            self.flag() as i32,
            self.func() as i32,
            self.args() as i32,
            self.load() as i32,
            self.pointer_size() as i32,
            self.a_arg() as i32,
            self.b_arg() as i32,
            self.d_arg() as i32,
        ]
    }
}

impl LineBuilderPane {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn renew_command(&self, new_command: &CommandRepr) {
        self.imp().renew_command(new_command.imp());
    }
    pub fn get_words(&self) -> [i32;12] {
        self.imp().get_command().get_words()
    }
    pub fn get_command(&self) -> CommandRepr {
        self.imp().get_command()
    }
}

/* register_view.rs
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

use crate::emulator;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/debug_pane/register_view.ui")]
    pub struct RegisterView {
        #[template_child]
        pq_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r0_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r1_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r2_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r3_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r4_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r5_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r6_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r7_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r8_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r9_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r10_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r11_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r12_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r13_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r14_reg: TemplateChild<gtk::Label>,
        #[template_child]
        r15_reg: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RegisterView {
        const NAME: &'static str = "RegisterView";
        type Type = super::RegisterView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RegisterView {}
    impl WidgetImpl for RegisterView {}
    impl BoxImpl for RegisterView {}
    impl RegisterView {
        pub fn renew_state(&self, new_state: &emulator::State) {
            self.r0_reg.set_label(&format!("{:0>4b}", new_state.registers[0]));
            self.r1_reg.set_label(&format!("{:0>4b}", new_state.registers[1]));
            self.r2_reg.set_label(&format!("{:0>4b}", new_state.registers[2]));
            self.r3_reg.set_label(&format!("{:0>4b}", new_state.registers[3]));
            self.r4_reg.set_label(&format!("{:0>4b}", new_state.registers[4]));
            self.r5_reg.set_label(&format!("{:0>4b}", new_state.registers[5]));
            self.r6_reg.set_label(&format!("{:0>4b}", new_state.registers[6]));
            self.r7_reg.set_label(&format!("{:0>4b}", new_state.registers[7]));
            self.r8_reg.set_label(&format!("{:0>4b}", new_state.registers[8]));
            self.r9_reg.set_label(&format!("{:0>4b}", new_state.registers[9]));
            self.r10_reg.set_label(&format!("{:0>4b}", new_state.registers[10]));
            self.r11_reg.set_label(&format!("{:0>4b}", new_state.registers[11]));
            self.r12_reg.set_label(&format!("{:0>4b}", new_state.registers[12]));
            self.r13_reg.set_label(&format!("{:0>4b}", new_state.registers[13]));
            self.r14_reg.set_label(&format!("{:0>4b}", new_state.registers[14]));
            self.r15_reg.set_label(&format!("{:0>4b}", new_state.registers[15]));
            self.pq_reg.set_label(&format!("{:0>4b}", new_state.registers[16]));
        }
    }
}

glib::wrapper! {
    pub struct RegisterView(ObjectSubclass<imp::RegisterView>)
        @extends gtk::Widget,        @implements gio::ActionGroup, gio::ActionMap;
}

impl RegisterView {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn renew_state(&self, new_state: &emulator::State) {
        self.imp().renew_state(new_state);
    }
}

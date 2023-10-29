/* output_view.rs
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
    #[template(resource = "/org/bmstu/mtemu/ui/debug_pane/output_view.ui")]
    pub struct OutputView {
        #[template_child]
        ovr_flag: TemplateChild<gtk::Label>,
        #[template_child]
        c4_flag: TemplateChild<gtk::Label>,
        #[template_child]
        f3_flag: TemplateChild<gtk::Label>,
        #[template_child]
        z_flag: TemplateChild<gtk::Label>,
        #[template_child]
        g_flag: TemplateChild<gtk::Label>,
        #[template_child]
        p_flag: TemplateChild<gtk::Label>,
        #[template_child]
        f_out: TemplateChild<gtk::Label>,
        #[template_child]
        y_out: TemplateChild<gtk::Label>,
        #[template_child]
        pc_out: TemplateChild<gtk::Label>,
        #[template_child]
        sp_out: TemplateChild<gtk::Label>,
        #[template_child]
        mp_out: TemplateChild<gtk::Label>,
        #[template_child]
        port_out: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OutputView {
        const NAME: &'static str = "OutputView";
        type Type = super::OutputView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OutputView {}
    impl WidgetImpl for OutputView {}
    impl BoxImpl for OutputView {}
    impl OutputView {
        pub fn renew_state(&self, new_state: &emulator::State) {
            self.set_flags(new_state);
            self.set_outputs(new_state);
        }
        fn set_flags(&self, new_state: &emulator::State) {
            self.ovr_flag.set_label(&format!("OVR={:b}", &new_state.flags[0]));
            self.c4_flag.set_label(&format!("C4={:b}", &new_state.flags[1]));
            self.f3_flag.set_label(&format!("F3={:b}", &new_state.flags[2]));
            self.z_flag.set_label(&format!("Z={:b}", &new_state.flags[3]));
            self.g_flag.set_label(&format!("/G={:b}", &new_state.flags[4]));
            self.p_flag.set_label(&format!("/P={:b}", &new_state.flags[5]));
        }
        fn set_outputs(&self, new_state: &emulator::State) {
            self.f_out.set_label(&format!("{:0>4b}", &new_state.func_output));
            self.y_out.set_label(&format!("{:0>4b}", &new_state.func_value));
            self.pc_out.set_label(&format!("{:0>4b}", &new_state.program_counter));
            self.sp_out.set_label(&format!("{:0>4b}", &new_state.stack_pointer));
            self.mp_out.set_label(&format!("{:0>4b}", &new_state.multiplexor_value));
            // self.port_out.set_label(&format!("{:0>4b}", &new_state.port_value));
        }
    }
}

glib::wrapper! {
    pub struct OutputView(ObjectSubclass<imp::OutputView>)
        @extends gtk::Widget,        @implements gio::ActionGroup, gio::ActionMap;
}

impl OutputView {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn renew_state(&self, new_state: &emulator::State) {
        self.imp().renew_state(new_state);
    }
}

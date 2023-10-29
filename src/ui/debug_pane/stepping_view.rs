/* stepping_view.rs
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

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bmstu/mtemu/ui/debug_pane/stepping_view.ui")]
    pub struct SteppingView {
        #[template_child]
        pub reset_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub step_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub run_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SteppingView {
        const NAME: &'static str = "SteppingView";
        type Type = super::SteppingView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SteppingView {}
    impl WidgetImpl for SteppingView {}
    impl BoxImpl for SteppingView {}
}

glib::wrapper! {
    pub struct SteppingView(ObjectSubclass<imp::SteppingView>)
        @extends gtk::Widget,        @implements gio::ActionGroup, gio::ActionMap;
}

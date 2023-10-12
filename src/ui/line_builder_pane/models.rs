/* line_builder_models.rs
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
use gtk::glib;
use paste::paste;

macro_rules! wrap_obj_subclass {
    ($x:ty, $y:ty, $z:literal ) => {
        #[glib::object_subclass]
        impl ObjectSubclass for $x {
            const NAME: &'static str = $z;
            type Type = $y;
            type ParentType = glib::Object;
        }
        impl ObjectImpl for $x {}
    }
}

mod imp {
    use std::cell::Cell;

    use super::*;

    #[derive(Default)]
    pub struct JumpEntry {
        pub ca: Cell<String>,
        pub jump: Cell<String>,
    }

    #[derive(Default)]
    pub struct InstrEntry {
        pub ca: Cell<String>,
        pub func: Cell<String>,
        pub bit: Cell<String>,
    }

    #[derive(Default)]
    pub struct PointerTypeEntry {
        pub ca: Cell<String>,
        pub ptr: Cell<String>,
    }

    #[derive(Default)]
    pub struct InterfaceEntry {
        pub ca: Cell<String>,
        pub interface: Cell<String>,
    }

    #[derive(Default)]
    pub struct PointerSizeEntry {
        pub ca: Cell<String>,
        pub size: Cell<String>,
    }

    #[derive(Default)]
    pub struct OperandTypeEntry {
        pub ca: Cell<String>,
        pub r: Cell<String>,
        pub s: Cell<String>,
    }

    #[derive(Default)]
    pub struct LoadTypeEntry {
        pub ca: Cell<String>,
        pub load: Cell<String>,
    }

    wrap_obj_subclass!(JumpEntry, super::JumpEntry, "JumpEntry");
    wrap_obj_subclass!(InstrEntry, super::InstrEntry, "InstrEntry");
    wrap_obj_subclass!(PointerTypeEntry, super::PointerTypeEntry, "PointerTypeEntry");
    wrap_obj_subclass!(InterfaceEntry, super::InterfaceEntry, "InterfaceEntry");
    wrap_obj_subclass!(PointerSizeEntry, super::PointerSizeEntry, "PointerSizeEntry");
    wrap_obj_subclass!(OperandTypeEntry, super::OperandTypeEntry, "OperandTypeEntry");
    wrap_obj_subclass!(LoadTypeEntry, super::LoadTypeEntry, "LoadTypeEntry");
}

glib::wrapper! { pub struct JumpEntry(ObjectSubclass<imp::JumpEntry>); }
glib::wrapper! { pub struct InstrEntry(ObjectSubclass<imp::InstrEntry>); }
glib::wrapper! { pub struct PointerTypeEntry(ObjectSubclass<imp::PointerTypeEntry>); }
glib::wrapper! { pub struct InterfaceEntry(ObjectSubclass<imp::InterfaceEntry>); }
glib::wrapper! { pub struct PointerSizeEntry(ObjectSubclass<imp::PointerSizeEntry>); }
glib::wrapper! { pub struct OperandTypeEntry(ObjectSubclass<imp::OperandTypeEntry>); }
glib::wrapper! { pub struct LoadTypeEntry(ObjectSubclass<imp::LoadTypeEntry>); }

macro_rules! new_obj {
    () => {
        pub fn new() -> Self {
            glib::Object::builder().build()
        }
    };
}

macro_rules! get_set_str {
    ($x:ident, $y:ty ) => {
        paste! {
            pub fn [<set_ $x>](&self, data: &str) {
                <$y>::from_obj(self).$x.set(data.to_string());
            }
        }
        paste! {
            pub fn [<get_ $x>](&self) -> String {
                let entry = <$y>::from_obj(self).$x.take();
                self.[<set_ $x>](&entry);
                entry
            }
        }
    };
}

impl JumpEntry {
    new_obj!();
    get_set_str!(ca, imp::JumpEntry);
    get_set_str!(jump, imp::JumpEntry);
}

impl InstrEntry {
    new_obj!();
    get_set_str!(ca, imp::InstrEntry);
    get_set_str!(func, imp::InstrEntry);
    get_set_str!(bit, imp::InstrEntry);
}

impl PointerTypeEntry {
    new_obj!();
    get_set_str!(ca, imp::PointerTypeEntry);
    get_set_str!(ptr, imp::PointerTypeEntry);
}

impl InterfaceEntry {
    new_obj!();
    get_set_str!(ca, imp::InterfaceEntry);
    get_set_str!(interface, imp::InterfaceEntry);
}

impl PointerSizeEntry {
    new_obj!();
    get_set_str!(ca, imp::PointerSizeEntry);
    get_set_str!(size, imp::PointerSizeEntry);
}

impl OperandTypeEntry {
    new_obj!();
    get_set_str!(ca, imp::OperandTypeEntry);
    get_set_str!(r, imp::OperandTypeEntry);
    get_set_str!(s, imp::OperandTypeEntry);
}

impl LoadTypeEntry {
    new_obj!();
    get_set_str!(ca, imp::LoadTypeEntry);
    get_set_str!(load, imp::LoadTypeEntry);
}

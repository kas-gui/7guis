// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! CRUD

use kas::event::VoidResponse;
use kas::prelude::*;
use kas::widget::{
    Column, EditBox, EditGuard, Frame, Label, MenuEntry, ScrollRegion, TextButton, Window,
};
use std::collections::HashMap;

#[derive(Clone, Debug, VoidMsg)]
enum Control {
    Create,
    Update,
    Delete,
    Select(u64),
}

#[derive(Clone, Debug)]
struct NameGuard;
impl EditGuard for NameGuard {
    type Msg = VoidMsg;
    fn edit(edit: &mut EditBox<Self>) -> Option<VoidMsg> {
        edit.set_error_state(edit.get_str().len() == 0);
        None
    }
}

// Data entries, each with a unique identifier
type Item = (String, String);
type Data = HashMap<u64, Item>;

trait Editor {
    fn make_item(&self) -> Option<Item>;
}

pub fn window() -> Box<dyn kas::Window> {
    // TODO: add a real listings box
    let entries: Vec<MenuEntry<u64>> = vec![];
    let list = Frame::new(ScrollRegion::new(Column::new(entries)));

    let filter_list = make_widget! {
        #[layout(column)]
        #[handler(msg = Control)]
        struct {
            #[widget] _ = Label::new("WARNING: UNFINISHED!"),
            // TODO: filter should optionally hide list entries (but without removing widgets?)
            #[widget] _ = make_widget! {
                #[layout(row)]
                #[handler(msg = VoidMsg)]
                struct {
                    #[widget] _ = Label::new("Filter prefix:"),
                    #[widget] _ = EditBox::new(""),
                }
            },
            #[widget(handler = select)] _ = list,
        }
        impl {
            fn select(&mut self, _: &mut Manager, key: u64) -> Response<Control> {
                Control::Select(key).into()
            }
        }
    };

    let mut edit = EditBox::new("").with_guard(NameGuard);
    edit.set_error_state(true);

    let editor = make_widget! {
        #[layout(grid)]
        #[handler(msg = VoidMsg)]
        struct {
            #[widget(row = 0, col = 0)] _ = Label::new("First name:"),
            #[widget(row = 0, col = 1)] firstname: EditBox<NameGuard> = edit.clone(),
            #[widget(row = 1, col = 0)] _ = Label::new("Surname:"),
            #[widget(row = 1, col = 1)] surname: EditBox<NameGuard> = edit,
        }
        impl Editor {
            fn make_item(&self) -> Option<Item> {
                if self.surname.len() == 0 {
                    return None;
                }
                Some((self.firstname.get_string(), self.surname.get_string()))
            }
        }
    };

    let controls = make_widget! {
        #[layout(row)]
        #[handler(msg = Control)]
        struct {
            #[widget] _ = TextButton::new("Create", Control::Create),
            #[widget] _ = TextButton::new("Update", Control::Update),
            #[widget] _ = TextButton::new("Delete", Control::Delete),
        }
    };

    Box::new(Window::new(
        "Create, Read, Update, Delete",
        make_widget! {
            #[layout(grid)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(row = 0, col = 0, handler = controls)] _ = filter_list,
                #[widget(row = 0, col = 1)] editor: impl Editor = editor,
                #[widget(row = 1, cspan = 2, handler = controls)] _ = controls,
                data: Data = Data::default(),
                next_id: u64 = 0,
            }
            impl {
                fn update_list(&mut self, mgr: &mut Manager) {
                    // TODO: for each item in the DB matching the filter,
                    // generate a new entry
                }
                fn controls(&mut self, mgr: &mut Manager, control: Control) -> VoidResponse {
                    match control {
                        Control::Create => {
                            if let Some(item) = self.editor.make_item() {
                                let id = self.next_id;
                                self.next_id += 1;
                                self.data.insert(id, item);
                                self.update_list(mgr);
                            }
                        }
                        Control::Update => {
                            // TODO: if filter_list has a selected entry,
                            // TODO: construct new entry from editor and update DB
                        }
                        Control::Delete => {
                            // TODO: if filter_list has a selected entry, delete
                        }
                        Control::Select(key) => {
                            // TODO: update editor with selected item
                        }
                    }
                    Response::None
                }
            }
        },
    ))
}

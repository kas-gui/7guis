// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::dir::Down;
use kas::event::VoidResponse;
use kas::prelude::*;
use kas::widget::view::ListView;
use kas::widget::view::{Accessor, FilterAccessor, SimpleCaseInsensitiveFilter};
use kas::widget::{EditBox, EditGuard, Filler, Label, ScrollBars, TextButton, Window};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Entry {
    first: String,
    last: String,
}
impl Entry {
    pub fn new<S: ToString, T: ToString>(last: T, first: S) -> Self {
        Entry {
            first: first.to_string(),
            last: last.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Entries(Vec<Entry>);
impl Entries {
    pub fn push(&mut self, entry: Entry) {
        self.0.push(entry)
    }
}

pub type SharedData = Rc<RefCell<FilterAccessor<usize, Entries, SimpleCaseInsensitiveFilter>>>;

impl Accessor<usize> for Entries {
    type Item = String;
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> Self::Item {
        let entry = &self.0[index];
        format!("{}, {}", entry.last, entry.first)
    }
}

pub fn make_data() -> SharedData {
    let entries = vec![
        Entry::new("Emil", "Hans"),
        Entry::new("Mustermann", "Max"),
        Entry::new("Tisch", "Roman"),
    ];
    let filter = SimpleCaseInsensitiveFilter::new("");
    Rc::new(RefCell::new(FilterAccessor::new(Entries(entries), filter)))
}

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
    fn edit(edit: &mut EditBox<Self>, _: &mut Manager) -> Option<VoidMsg> {
        edit.set_error_state(edit.get_str().len() == 0);
        None
    }
}

trait Editor {
    fn make_item(&self) -> Option<Entry>;
}

pub fn window() -> Box<dyn kas::Window> {
    let data = make_data();
    let data2 = data.clone();
    let data3 = data.clone();

    let filter_list = make_widget! {
        #[layout(column)]
        #[handler(msg = Control)]
        struct {
            #[widget] filter = EditBox::new("").on_edit(move |text, mgr| {
                let filter = SimpleCaseInsensitiveFilter::new(text);
                let update = data2.borrow_mut().set_filter(filter);
                mgr.trigger_update(update, 0);
                Option::<VoidMsg>::None
            }),
            #[widget] list =
                ScrollBars::new(ListView::<Down, SharedData>::new(data)),
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
            fn make_item(&self) -> Option<Entry> {
                let last = self.surname.get_string();
                if last.len() == 0 {
                    return None;
                }
                Some(Entry::new(last, self.firstname.get_string()))
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
            #[widget] _ = Filler::maximize(),
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
                data: SharedData = data3,
                next_id: u64 = 0,
            }
            impl {
                fn controls(&mut self, mgr: &mut Manager, control: Control) -> VoidResponse {
                    match control {
                        Control::Create => {
                            if let Some(item) = self.editor.make_item() {
                                let mut data = self.data.borrow_mut();
                                data.data.push(item);
                                let update = data.refresh();
                                mgr.trigger_update(update, 0);
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

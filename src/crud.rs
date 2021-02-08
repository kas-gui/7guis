// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! CRUD

use kas::dir::Down;
use kas::event::VoidResponse;
use kas::prelude::*;
use kas::widget::view::ListView;
use kas::widget::{EditBox, EditGuard, Filler, Label, ScrollBars, TextButton, Window};
use std::collections::HashMap;

mod data {
    use kas::widget::view::{Accessor, FilterAccessor};
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

    pub type Shared = Rc<RefCell<FilterAccessor<usize, Entries>>>;

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

    pub fn get() -> Shared {
        let entries = vec![
            Entry::new("Emil", "Hans"),
            Entry::new("Mustermann", "Max"),
            Entry::new("Tisch", "Roman"),
        ];
        Rc::new(RefCell::new(FilterAccessor::new_visible(Entries(entries))))
    }
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

// Data entries, each with a unique identifier
type Item = (String, String);
type Data = HashMap<u64, Item>;

trait Editor {
    fn make_item(&self) -> Option<Item>;
}

pub fn window() -> Box<dyn kas::Window> {
    let data = data::get();
    let data2 = data.clone();

    let filter_list = make_widget! {
        #[layout(column)]
        #[handler(msg = Control)]
        struct {
            #[widget] filter = EditBox::new("").on_edit(move |text, mgr| {
                // Note: this method of caseless matching is not unicode compliant!
                // https://stackoverflow.com/questions/47298336/case-insensitive-string-matching-in-rust
                let text = text.to_uppercase();
                let update = data2
                    .borrow_mut()
                    .update_filter(|s| s.to_uppercase().contains(&text));
                mgr.trigger_update(update, 0);
                Option::<VoidMsg>::None
            }),
            #[widget] list =
                ScrollBars::new(ListView::<Down, data::Shared>::new(data)),
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

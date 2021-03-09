// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::dir::Down;
use kas::event::{UpdateHandle, VoidResponse};
use kas::prelude::*;
use kas::widget::view::{FilteredList, SimpleCaseInsensitiveFilter};
use kas::widget::view::{ListData, ListMsg, ListView, SelectionMode};
use kas::widget::{EditBox, EditField, EditGuard, Filler, Label, ScrollBars, TextButton, Window};
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
    pub fn format(&self) -> String {
        format!("{}, {}", self.last, self.first)
    }
}

#[derive(Debug)]
pub struct Entries {
    v: RefCell<Vec<Entry>>,
    // TODO: we now have two update handles
    u: UpdateHandle,
}

// Implement a simple (lazy) CRUD interface
impl Entries {
    pub fn new(v: Vec<Entry>) -> Self {
        Entries {
            v: RefCell::new(v),
            u: UpdateHandle::new(),
        }
    }
    pub fn create(&self, entry: Entry) -> UpdateHandle {
        self.v.borrow_mut().push(entry);
        self.u
    }
    pub fn read(&self, index: usize) -> Entry {
        self.v.borrow()[index].clone()
    }
    pub fn update(&self, index: usize, entry: Entry) -> UpdateHandle {
        self.v.borrow_mut()[index] = entry;
        self.u
    }
    pub fn delete(&self, index: usize) -> UpdateHandle {
        self.v.borrow_mut().remove(index);
        self.u
    }
}

pub type SharedData = Rc<FilteredList<Entries, SimpleCaseInsensitiveFilter>>;

impl ListData for Entries {
    type Key = usize;
    type Item = String;

    fn len(&self) -> usize {
        self.v.borrow().len()
    }

    fn get_cloned(&self, key: &Self::Key) -> Option<Self::Item> {
        self.v.borrow().get(*key).map(|e| e.format())
    }

    fn update(&self, _: &Self::Key, _: Self::Item) -> Option<UpdateHandle> {
        None // we could implement updates but don't need to
    }

    fn iter_vec_from(&self, start: usize, limit: usize) -> Vec<(Self::Key, Self::Item)> {
        let v = self.v.borrow();
        v.iter()
            .map(|e| e.format())
            .enumerate()
            .skip(start)
            .take(limit)
            .collect()
    }
}

pub fn make_data() -> SharedData {
    let entries = vec![
        Entry::new("Emil", "Hans"),
        Entry::new("Mustermann", "Max"),
        Entry::new("Tisch", "Roman"),
    ];
    let filter = SimpleCaseInsensitiveFilter::new("");
    Rc::new(FilteredList::new(Entries::new(entries), filter))
}

#[derive(Clone, Debug, VoidMsg)]
enum Control {
    Create,
    Update,
    Delete,
    Select(usize),
    Filter,
}

#[derive(Clone, Debug)]
struct NameGuard;
impl EditGuard for NameGuard {
    type Msg = VoidMsg;
    fn update(edit: &mut EditField<Self>) {
        edit.set_error_state(edit.get_str().len() == 0);
    }
}

trait Editor {
    fn make_item(&self) -> Option<Entry>;
    fn set_item(&mut self, item: Entry) -> TkAction;
    fn clear(&mut self) -> TkAction;
}

trait Selected {
    fn selected(&self) -> Option<usize>;
    fn clear_selected(&mut self);
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
                let update = data2.set_filter(filter);
                mgr.trigger_update(update, 0);
                Some(Control::Filter)
            }),
            #[widget(handler=select)] list: ScrollBars<ListView<Down, SharedData>> =
                ScrollBars::new(ListView::new(data).with_selection_mode(SelectionMode::Single)),
        }
        impl {
            fn select(&mut self, _: &mut Manager, msg: ListMsg<usize, VoidMsg>) -> Response<Control> {
                match msg {
                    ListMsg::Select(key) => Control::Select(key).into(),
                    _ => Response::None
                }
            }
        }
        impl Selected {
            fn selected(&self) -> Option<usize> {
                self.list.selected_iter().next().cloned()
            }
            fn clear_selected(&mut self) {
                self.list.clear_selected();
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
            fn make_item(&self) -> Option<Entry> {
                let last = self.surname.get_string();
                if last.len() == 0 {
                    return None;
                }
                Some(Entry::new(last, self.firstname.get_string()))
            }
            fn set_item(&mut self, item: Entry) -> TkAction {
                self.firstname.set_string(item.first) | self.surname.set_string(item.last)
            }
            fn clear(&mut self) -> TkAction {
                self.firstname.set_string("".into()) | self.surname.set_string("".into())
            }
        }
    };

    let controls = make_widget! {
        #[layout(row)]
        #[handler(msg = Control)]
        struct {
            #[widget] _ = TextButton::new_msg("Create", Control::Create),
            #[widget] _ = TextButton::new_msg("Update", Control::Update),
            #[widget] _ = TextButton::new_msg("Delete", Control::Delete),
            #[widget] _ = Filler::maximize(),
        }
    };

    Box::new(Window::new(
        "Create, Read, Update, Delete",
        make_widget! {
            #[layout(grid)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(row = 0, col = 0, handler = controls)] filter_list: impl Selected =
                    filter_list,
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
                                let update = self.data.data.create(item);
                                mgr.trigger_update(update, 0);
                            }
                        }
                        Control::Update => {
                            if let Some(index) = self.filter_list.selected() {
                                if let Some(item) = self.editor.make_item() {
                                    let update = self.data.data.update(index, item);
                                    mgr.trigger_update(update, 0);
                                }
                            }
                        }
                        Control::Delete => {
                            if let Some(index) = self.filter_list.selected() {
                                let update = self.data.data.delete(index);
                                mgr.trigger_update(update, 0);
                            }
                        }
                        Control::Select(key) => {
                            let item = self.data.data.read(key);
                            *mgr |= self.editor.set_item(item);
                        }
                        Control::Filter => {
                            if let Some(index) = self.filter_list.selected() {
                                if self.data.get_cloned(&index).is_none() {
                                    self.filter_list.clear_selected();
                                    *mgr |= self.editor.clear();
                                }
                            }
                        }
                    }
                    Response::None
                }
            }
        },
    ))
}

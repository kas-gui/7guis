// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::data::{ListData, SharedData, SharedDataRec};
use kas::dir::Down;
use kas::event::{UpdateHandle, VoidResponse};
use kas::prelude::*;
use kas::widget::view::{FilteredList, SimpleCaseInsensitiveFilter};
use kas::widget::view::{ListMsg, ListView, SelectionMode};
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
    pub fn create(&self, entry: Entry) -> (usize, UpdateHandle) {
        let mut v = self.v.borrow_mut();
        let i = v.len();
        v.push(entry);
        (i, self.u)
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

pub type Data = Rc<FilteredList<Entries, SimpleCaseInsensitiveFilter>>;

impl SharedData for Entries {
    fn update_handle(&self) -> Option<UpdateHandle> {
        Some(self.u)
    }
}
impl SharedDataRec for Entries {}
impl ListData for Entries {
    type Key = usize;
    type Item = String;

    fn len(&self) -> usize {
        self.v.borrow().len()
    }

    fn contains_key(&self, key: &Self::Key) -> bool {
        *key < self.len()
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

pub fn make_data() -> Data {
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
    fn select(&mut self, index: usize) -> bool;
}

trait Disable {
    fn disable_update_delete(&mut self, disable: bool) -> TkAction;
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
            #[widget(handler=select)] list: ScrollBars<ListView<Down, Data>> =
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
            fn select(&mut self, index: usize) -> bool {
                self.list.select(index).is_ok()
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
            #[widget] update = TextButton::new_msg("Update", Control::Update)
                .with_disabled(true),
            #[widget] delete = TextButton::new_msg("Delete", Control::Delete)
                .with_disabled(true),
            #[widget] _ = Filler::maximize(),
        }
        impl Disable {
            fn disable_update_delete(&mut self, disable: bool) -> TkAction {
                self.update.set_disabled(disable) | self.delete.set_disabled(disable)
            }
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
                #[widget(row = 1, cspan = 2, handler = controls)] controls: impl Disable = controls,
                data: Data = data3,
            }
            impl {
                fn controls(&mut self, mgr: &mut Manager, control: Control) -> VoidResponse {
                    match control {
                        Control::Create => {
                            if let Some(item) = self.editor.make_item() {
                                let (index, update) = self.data.data.create(item);
                                mgr.trigger_update(update, 0);
                                self.filter_list.select(index);
                                *mgr |= self.controls.disable_update_delete(false);
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
                                let any_selected = self.filter_list.select(index);
                                if any_selected {
                                    let item = self.data.data.read(index);
                                    *mgr |= self.editor.set_item(item);
                                }
                                *mgr |= self.controls.disable_update_delete(!any_selected);
                            }
                        }
                        Control::Select(key) => {
                            let item = self.data.data.read(key);
                            *mgr |= self.editor.set_item(item);
                            *mgr |= self.controls.disable_update_delete(false);
                        }
                        Control::Filter => {
                            if let Some(index) = self.filter_list.selected() {
                                if !self.data.contains_key(&index) {
                                    self.filter_list.clear_selected();
                                    *mgr |= self.editor.clear()
                                        | self.controls.disable_update_delete(true);
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

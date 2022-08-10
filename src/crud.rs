// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::dir::Down;
use kas::model::filter::{ContainsCaseInsensitive, FilteredList};
use kas::model::{ListData, SharedData};
use kas::prelude::*;
use kas::view::{driver, ListView, SelectionMode, SelectionMsg};
use kas::widgets::edit::{EditBox, EditField, EditGuard};
use kas::widgets::{Frame, ScrollBars, TextButton};
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
struct EntriesInner {
    ver: u64,
    vec: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entries {
    inner: RefCell<EntriesInner>,
    u: UpdateId,
}

// Implement a simple (lazy) CRUD interface
impl Entries {
    pub fn new(vec: Vec<Entry>) -> Self {
        let ver = 0;
        let inner = RefCell::new(EntriesInner { ver, vec });
        let u = UpdateId::new();
        Entries { inner, u }
    }
    pub fn create(&self, entry: Entry) -> (usize, UpdateId) {
        let mut inner = self.inner.borrow_mut();
        let index = inner.vec.len();
        inner.ver += 1;
        inner.vec.push(entry);
        (index, self.u)
    }
    pub fn read(&self, index: usize) -> Entry {
        self.inner.borrow().vec[index].clone()
    }
    pub fn update_entry(&self, index: usize, entry: Entry) -> UpdateId {
        let mut inner = self.inner.borrow_mut();
        inner.ver += 1;
        inner.vec[index] = entry;
        self.u
    }
    pub fn delete(&self, index: usize) -> UpdateId {
        let mut inner = self.inner.borrow_mut();
        inner.ver += 1;
        inner.vec.remove(index);
        self.u
    }
}

pub type Data = Rc<Entries>;

impl SharedData for Entries {
    type Key = usize;
    type Item = String;

    fn version(&self) -> u64 {
        self.inner.borrow().ver
    }

    fn contains_key(&self, key: &Self::Key) -> bool {
        *key < self.len()
    }

    fn get_cloned(&self, key: &Self::Key) -> Option<Self::Item> {
        self.inner.borrow().vec.get(*key).map(|e| e.format())
    }

    fn update(&self, _: &mut EventMgr, _: &Self::Key, _: Self::Item) {}
}
impl ListData for Entries {
    fn len(&self) -> usize {
        self.inner.borrow().vec.len()
    }

    fn make_id(&self, parent: &WidgetId, key: &Self::Key) -> WidgetId {
        parent.make_child(*key)
    }
    fn reconstruct_key(&self, parent: &WidgetId, child: &WidgetId) -> Option<Self::Key> {
        child.next_key_after(parent)
    }

    fn iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::Key> {
        (0..self.inner.borrow().vec.len())
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
    Rc::new(Entries::new(entries))
}

#[derive(Clone, Debug)]
enum Control {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug)]
struct NameGuard;
impl EditGuard for NameGuard {
    fn update(edit: &mut EditField<Self>) {
        edit.set_error_state(edit.get_str().len() == 0);
    }
}

impl_scope! {
    #[derive(Debug)]
    #[impl_default]
    #[widget {
        layout = grid: {
            0, 0: "First name:";
            1, 0: self.firstname;
            0, 1: "Surname:";
            1, 1: self.surname;
        };
    }]
    struct Editor {
        core: widget_core!(),
        #[widget] firstname: EditBox<NameGuard> = EditBox::new("".to_string()).with_guard(NameGuard),
        #[widget] surname: EditBox<NameGuard> = EditBox::new("".to_string()).with_guard(NameGuard),
    }
    impl Self {
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
    }
}

impl_scope! {
    #[derive(Debug)]
    #[impl_default]
    #[widget {
        layout = row: [
            TextButton::new_msg("Create", Control::Create),
            self.update,
            self.delete,
        ];
    }]
    struct Controls {
        core: widget_core!(),
        #[widget] update: TextButton = TextButton::new_msg("Update", Control::Update),
        #[widget] delete: TextButton = TextButton::new_msg("Delete", Control::Delete),
    }
    impl Self {
        fn disable_update_delete(&mut self, mgr: &mut EventMgr, disable: bool) {
            mgr.set_disabled(self.update.id(), disable);
            mgr.set_disabled(self.delete.id(), disable);
        }
    }
    impl Widget for Self {
        fn configure(&mut self, mgr: &mut ConfigMgr) {
            mgr.set_disabled(self.update.id(), true);
            mgr.set_disabled(self.delete.id(), true);
        }
    }
}

pub fn window() -> Box<dyn Window> {
    let data = make_data();
    let filter = ContainsCaseInsensitive::new("");

    type MyFilteredList = FilteredList<Data, ContainsCaseInsensitive>;
    type FilterList = ListView<Down, MyFilteredList, driver::NavView>;
    let list_view = FilterList::new(MyFilteredList::new(data.clone(), filter.clone()))
        .with_selection_mode(SelectionMode::Single);

    Box::new(impl_singleton! {
        #[derive(Debug)]
        #[widget {
            layout = grid: {
                0, 0: "Filter:";
                1, 0: self.filter;
                0..2, 1..3: self.list;
                3, 1: self.editor;
                0..4, 3: self.controls;
            };
        }]
        struct {
            core: widget_core!(),
            #[widget] filter = EditBox::new("")
                .on_edit(move |mgr, s| filter.update(mgr, &(), s.to_string())),
            #[widget] list: Frame<ScrollBars<FilterList>> =
                Frame::new(ScrollBars::new(list_view)),
            #[widget] editor: Editor = Editor::default(),
            #[widget] controls: Controls = Controls::default(),
            data: Data = data,
        }
        impl Self {
            fn selected(&self) -> Option<usize> {
                self.list.selected_iter().next().cloned()
            }
        }
        impl Widget for Self {
            fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
                if let Some(msg) = mgr.try_pop_msg() {
                    match msg {
                        SelectionMsg::Select(key) => {
                            let item = self.data.read(key);
                            *mgr |= self.editor.set_item(item);
                            self.controls.disable_update_delete(mgr, false);
                        }
                        _ => (),
                    }
                } else if let Some(control) = mgr.try_pop_msg() {
                    match control {
                        Control::Create => {
                            if let Some(item) = self.editor.make_item() {
                                let (index, update) = self.data.create(item);
                                mgr.update_all(update, 0);
                                let _ = self.list.select(index);
                                self.controls.disable_update_delete(mgr, false);
                            }
                        }
                        Control::Update => {
                            if let Some(index) = self.selected() {
                                if let Some(item) = self.editor.make_item() {
                                    let update = self.data.update_entry(index, item);
                                    mgr.update_all(update, 0);
                                }
                            }
                        }
                        Control::Delete => {
                            if let Some(index) = self.selected() {
                                let update = self.data.delete(index);
                                mgr.update_all(update, 0);
                                let any_selected = self.list.select(index).is_ok();
                                if any_selected {
                                    let item = self.data.read(index);
                                    *mgr |= self.editor.set_item(item);
                                }
                                self.controls.disable_update_delete(mgr, !any_selected);
                            }
                        }
                    }
                }
            }
        }
        impl Window for Self {
            fn title(&self) -> &str {
                "Create, Read, Update, Delete"
            }
        }
    })
}

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::prelude::*;
use kas::view::filter::{
    ContainsCaseInsensitive, Filter, FilterList, KeystrokeGuard, SetFilter, UnsafeFilteredList,
};
use kas::view::{Driver, ListView, SelectionMode, SelectionMsg};
use kas::widgets::edit::{EditBox, EditField, EditGuard};
use kas::widgets::{AccessLabel, Button, Frame, NavFrame, ScrollBars, Text};

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
    pub fn format(_: &ConfigCx, entry: &Entry) -> String {
        format!("{}, {}", entry.last, entry.first)
    }
}
impl Filter<Entry> for ContainsCaseInsensitive {
    fn matches(&self, item: &Entry) -> bool {
        Filter::<&str>::matches(self, &item.first.as_str())
            || Filter::<&str>::matches(self, &item.last.as_str())
    }
}

#[derive(Clone, Debug)]
enum Control {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug)]
struct NameGuard {
    is_last: bool,
}
impl EditGuard for NameGuard {
    type Data = Option<Entry>;

    fn update(edit: &mut EditField<Self>, cx: &mut ConfigCx, data: &Self::Data) {
        let mut act = Action::empty();
        if let Some(entry) = data.as_ref() {
            let name = match edit.guard.is_last {
                false => &entry.first,
                true => &entry.last,
            };
            act = edit.set_str(name);
        }
        act |= edit.set_error_state(edit.get_str().is_empty());
        cx.action(edit, act);
    }
}

impl_scope! {
    #[impl_default]
    #[widget {
        Data = Option<Entry>;
        layout = grid! {
            (0, 0) => "First name:",
            (1, 0) => self.firstname,
            (0, 1) => "Surname:",
            (1, 1) => self.surname,
        };
    }]
    struct Editor {
        core: widget_core!(),
        #[widget] firstname: EditBox<NameGuard> = EditBox::new(NameGuard { is_last: false }),
        #[widget] surname: EditBox<NameGuard> = EditBox::new(NameGuard { is_last: true }),
    }
    impl Self {
        fn make_item(&self) -> Option<Entry> {
            let last = self.surname.get_string();
            if last.is_empty() {
                return None;
            }
            Some(Entry::new(last, self.firstname.get_string()))
        }
    }
}

impl_scope! {
    #[impl_default]
    #[widget {
        layout = row! [
            Button::label_msg("Create", Control::Create).map_any(),
            self.update,
            self.delete,
        ];
    }]
    struct Controls {
        core: widget_core!(),
        #[widget(&())] update: Button<AccessLabel> = Button::label_msg("Update", Control::Update),
        #[widget(&())] delete: Button<AccessLabel> = Button::label_msg("Delete", Control::Delete),
    }
    impl Events for Self {
        type Data = bool;

        fn update(&mut self, cx: &mut ConfigCx, any_selected: &bool) {
            if self.update.id_ref().is_valid() {
                let disable = !any_selected;
                cx.set_disabled(self.update.id(), disable);
                cx.set_disabled(self.delete.id(), disable);
            }
        }
    }
}

pub fn window() -> Window<()> {
    struct ListGuard;
    type FilteredList = UnsafeFilteredList<Vec<Entry>>;
    impl Driver<Entry, FilteredList> for ListGuard {
        type Widget = NavFrame<Text<Entry, String>>;
        fn make(&mut self, _: &usize) -> Self::Widget {
            NavFrame::new(Text::new(Entry::format))
        }
    }
    let filter = ContainsCaseInsensitive::new();
    let guard = KeystrokeGuard;
    type MyListView = ListView<UnsafeFilteredList<Vec<Entry>>, ListGuard, kas::dir::Down>;
    type MyFilterList = FilterList<Vec<Entry>, ContainsCaseInsensitive, MyListView>;
    let list_view = MyListView::new(ListGuard).with_selection_mode(SelectionMode::Single);

    let ui = impl_anon! {
        #[widget {
            layout = grid! {
                (0, 0) => "Filter:",
                (1, 0) => self.filter,
                (0..2, 1..3) => self.list,
                (3, 1) => self.editor,
                (0..4, 3) => self.controls,
            };
        }]
        struct {
            core: widget_core!(),
            #[widget(&())] filter: EditBox<KeystrokeGuard> = EditBox::new(guard),
            #[widget(&self.entries)] list: Frame<ScrollBars<MyFilterList>> =
                Frame::new(ScrollBars::new(FilterList::new(list_view, filter))),
            #[widget(&self.selected)] editor: Editor = Editor::default(),
            #[widget(&self.selected.is_some())] controls: Controls = Controls::default(),
            entries: Vec<Entry> = vec![
                Entry::new("Emil", "Hans"),
                Entry::new("Mustermann", "Max"),
                Entry::new("Tisch", "Roman"),
            ],
            selected: Option<Entry>,
        }
        impl Self {
            fn selected(&self) -> Option<usize> {
                self.list.selected_iter().next().cloned()
            }
        }
        impl Events for Self {
            type Data = ();

            fn handle_messages(&mut self, cx: &mut EventCx, _: &()) {
                if let Some(SetFilter(value)) = cx.try_pop() {
                    self.list.set_filter(&mut cx.config_cx(), &self.entries, value);
                } else if let Some(SelectionMsg::Select(key)) = cx.try_pop() {
                    self.selected = self.entries.get::<usize>(key).cloned();
                    cx.update(self.as_node(&()));
                } else if let Some(control) = cx.try_pop() {
                    match control {
                        Control::Create => {
                            if let Some(item) = self.editor.make_item() {
                                let index = self.entries.len();
                                self.entries.push(item);
                                let action = self.list.select(index);
                                cx.action(&self, action);
                                self.selected = self.entries.get(index).cloned();
                                cx.update(self.as_node(&()));
                            }
                        }
                        Control::Update => {
                            if let Some(index) = self.selected() {
                                if let Some(item) = self.editor.make_item() {
                                    self.entries[index] = item;
                                    cx.update(self.as_node(&()));
                                }
                            }
                        }
                        Control::Delete => {
                            if let Some(index) = self.selected() {
                                self.entries.remove(index);
                                let action = self.list.select(index);
                                cx.action(&self, action);
                                self.selected = self.entries.get(index).cloned();
                                cx.update(self.as_node(&()));
                            }
                        }
                    }
                }
            }
        }
    };

    Window::new(ui, "Create, Read, Update, Delete")
}

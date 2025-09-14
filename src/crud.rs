// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use std::ops::Range;

use kas::dir::Down;
use kas::view::filter::{ContainsCaseInsensitive, Filter, FilterValue, KeystrokeGuard, SetFilter};
use kas::view::{DataChanges, DataClerk, DataLen, Driver, ListView, SelectionMsg, TokenChanges};
use kas::widgets::edit::{EditBox, EditField, EditGuard};
use kas::widgets::{AccessLabel, Button, ScrollBars, Text};
use kas::{prelude::*, TextOrSource};

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
        Filter::<str>::matches(self, item.first.as_str())
            || Filter::<str>::matches(self, item.last.as_str())
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
        if let Some(entry) = data.as_ref() {
            let name = match edit.guard.is_last {
                false => &entry.first,
                true => &entry.last,
            };
            edit.set_str(cx, name);
        }
        edit.set_error_state(cx, edit.as_str().is_empty());
    }
}

impl_scope! {
    #[impl_default]
    #[widget]
    #[layout(grid! {
        (0, 0) => "First name:",
        (1, 0) => self.firstname,
        (0, 1) => "Surname:",
        (1, 1) => self.surname,
    })]
    struct Editor {
        core: widget_core!(),
        #[widget] firstname: EditBox<NameGuard> = EditBox::new(NameGuard { is_last: false }),
        #[widget] surname: EditBox<NameGuard> = EditBox::new(NameGuard { is_last: true }),
    }
    impl Self {
        fn make_item(&self) -> Option<Entry> {
            let last = self.surname.clone_string();
            if last.is_empty() {
                return None;
            }
            Some(Entry::new(last, self.firstname.clone_string()))
        }
    }
    impl Events for Self {
        type Data = Option<Entry>;
    }
}

impl_scope! {
    #[impl_default]
    #[widget]
    #[layout(row! [
        Button::label_msg("Create", Control::Create).map_any(),
        self.update,
        self.delete,
    ])]
    struct Controls {
        core: widget_core!(),
        #[widget(&())] update: Button<AccessLabel> = Button::label_msg("Update", Control::Update),
        #[widget(&())] delete: Button<AccessLabel> = Button::label_msg("Delete", Control::Delete),
    }
    impl Events for Self {
        type Data = Option<Entry>;

        fn update(&mut self, cx: &mut ConfigCx, selected: &Self::Data) {
            if self.update.id_ref().is_valid() {
                let disable = selected.is_none();
                cx.set_disabled(self.update.id(), disable);
                cx.set_disabled(self.delete.id(), disable);
            }
        }
    }
}

struct EntriesClerk {
    // Note: deleted entries are replaced with None instead of being removed.
    // This is an easy way of ensuring that Key-Entry mappings do not change.
    entries: Vec<Option<Entry>>,
    filtered_entries: Vec<usize>,
}

impl DataClerk<usize> for EntriesClerk {
    type Data = ContainsCaseInsensitive;
    type Key = usize;
    type Item = Entry;
    type Token = usize;

    fn update(
        &mut self,
        _: &mut ConfigCx,
        _: Id,
        _: Range<usize>,
        filter: &Self::Data,
    ) -> DataChanges<usize> {
        // TODO(opt) determine when updates are a no-op and return DataChanges::None

        self.filtered_entries = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, opt)| {
                opt.as_ref()
                    .map(|entry| filter.matches(entry))
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();

        DataChanges::Any
    }

    fn len(&self, _: &Self::Data, _: usize) -> DataLen<usize> {
        DataLen::Known(self.filtered_entries.len())
    }

    fn update_token(
        &self,
        _: &Self::Data,
        index: usize,
        _: bool,
        token: &mut Option<usize>,
    ) -> TokenChanges {
        let key = self.filtered_entries.get(index).cloned();
        if *token == key {
            TokenChanges::None
        } else {
            *token = key;
            TokenChanges::Any
        }
    }

    fn item(&self, _: &Self::Data, key: &usize) -> &Entry {
        self.entries
            .get(*key)
            .map(|inner| inner.as_ref())
            .flatten()
            .unwrap()
    }
}

pub fn window() -> Window<()> {
    struct EntriesDriver;
    impl Driver<usize, Entry> for EntriesDriver {
        type Widget = Text<Entry, String>;

        fn make(&mut self, _: &usize) -> Self::Widget {
            Text::new(Entry::format)
        }

        fn navigable(_: &Self::Widget) -> bool {
            true
        }

        fn label(widget: &Self::Widget) -> Option<TextOrSource<'_>> {
            Some(widget.as_str().into())
        }
    }

    type EntriesView = ListView<EntriesClerk, EntriesDriver, Down>;
    let clerk = EntriesClerk {
        entries: vec![
            Some(Entry::new("Emil", "Hans")),
            Some(Entry::new("Mustermann", "Max")),
            Some(Entry::new("Tisch", "Roman")),
        ],
        filtered_entries: vec![],
    };

    let ui = impl_anon! {
        #[widget]
        #[layout(grid! {
            (0, 0) => "Filter:",
            (1, 0) => self.filter_field,
            (0..2, 1..3) => frame!(self.list),
            (3, 1) => self.editor,
            (0..4, 3) => self.controls,
        })]
        struct {
            core: widget_core!(),
            #[widget(&())] filter_field: EditBox<KeystrokeGuard> = EditBox::new(KeystrokeGuard),
            #[widget(&self.filter)] list: ScrollBars<EntriesView> =
                ScrollBars::new(EntriesView::new(clerk, EntriesDriver).with_selection_mode(kas::view::SelectionMode::Single)),
            #[widget(&self.selected)] editor: Editor = Editor::default(),
            #[widget(&self.selected)] controls: Controls = Controls::default(),
            filter: ContainsCaseInsensitive,
            selected: Option<Entry>,
        }
        impl Self {
            fn selected(&self) -> Option<usize> {
                self.list.inner().selected_iter().next().cloned()
            }
        }
        impl Events for Self {
            type Data = ();

            fn handle_messages(&mut self, cx: &mut EventCx, _: &()) {
                if let Some(SetFilter(value)) = cx.try_pop() {
                    self.filter.set_filter(value);
                    cx.update(self.list.as_node(&self.filter));
                } else if let Some(SelectionMsg::Select(key)) = cx.try_pop() {
                    self.selected = self.list.inner().clerk().entries.get::<usize>(key).cloned().flatten();
                    cx.update(self.as_node(&()));
                } else if let Some(control) = cx.try_pop() {
                    match control {
                        Control::Create => {
                            if let Some(item) = self.editor.make_item() {
                                let index = self.list.inner().clerk().entries.len();
                                self.list.inner_mut().clerk_mut().entries.push(Some(item));
                                cx.update(self.list.as_node(&self.filter));
                                self.list.inner_mut().select(cx, index);
                                self.selected = self.list.inner().clerk().entries.get(index).cloned().flatten();
                                cx.update(self.as_node(&()));
                            }
                        }
                        Control::Update => {
                            if let Some(index) = self.selected() {
                                if let Some(item) = self.editor.make_item() {
                                    self.list.inner_mut().clerk_mut().entries[index] = Some(item);
                                    cx.update(self.list.as_node(&self.filter));
                                    cx.update(self.as_node(&()));
                                }
                            }
                        }
                        Control::Delete => {
                            if let Some(index) = self.selected() {
                                self.list.inner_mut().clerk_mut().entries[index] = None;
                                cx.update(self.list.as_node(&self.filter));
                                self.list.inner_mut().select(cx, index);
                                self.selected = self.list.inner().clerk().entries.get(index).cloned().flatten();
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

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::prelude::*;
use kas::updatable::{RecursivelyUpdatable, Updatable, UpdatableHandler};
use kas::widget::view::{Driver, MatrixData, MatrixDataMut, MatrixView};
use kas::widget::{EditField, EditGuard, Window};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
struct ColKey(u8);
impl ColKey {
    const LEN: u8 = 26;
    fn try_from_u8(n: u8) -> Option<Self> {
        if n >= b'A' && n <= b'Z' {
            Some(ColKey(n))
        } else {
            None
        }
    }
    fn iter_keys() -> impl Iterator<Item = Self> {
        (b'A'..=b'Z').map(|n| ColKey::try_from_u8(n).unwrap())
    }
}

#[derive(Debug, PartialEq)]
pub enum Formula {
    Val(f64),
    // Ref(String),
    Add(Box<Formula>, Box<Formula>),
    Sub(Box<Formula>, Box<Formula>),
    Mul(Box<Formula>, Box<Formula>),
    Div(Box<Formula>, Box<Formula>),
}

impl Formula {
    fn eval(&self) -> f64 {
        use Formula::*;
        match self {
            Val(x) => *x,
            Add(f, g) => f.eval() + g.eval(),
            Sub(f, g) => f.eval() - g.eval(),
            Mul(f, g) => f.eval() * g.eval(),
            Div(f, g) => f.eval() / g.eval(),
        }
    }
}

#[derive(Debug, Default)]
struct Cell {
    input: String,
    formula: Option<Formula>,
    display: String,
}

impl Cell {
    // TODO: construct formula from input string instead
    fn new(input: String, formula: Option<Formula>) -> Self {
        Cell {
            input,
            formula,
            display: String::new(),
        }
    }
    fn eval(&mut self) {
        self.display = if let Some(ref f) = self.formula {
            f.eval().to_string()
        } else {
            self.input.clone()
        };
    }
}

type Key = (ColKey, u8);

#[derive(Debug)]
struct CellData {
    cells: RefCell<HashMap<Key, Cell>>,
    update: UpdateHandle,
}

impl CellData {
    fn new() -> Self {
        CellData {
            cells: RefCell::new(HashMap::new()),
            update: UpdateHandle::new(),
        }
    }
    fn eval_all(&mut self) {
        for cell in self.cells.get_mut().values_mut() {
            cell.eval();
        }
    }
}

impl Updatable for CellData {
    fn update_handle(&self) -> Option<UpdateHandle> {
        Some(self.update)
    }
}
impl RecursivelyUpdatable for CellData {}

impl MatrixData for CellData {
    type ColKey = ColKey;
    type RowKey = u8;
    type Key = (ColKey, u8);
    type Item = String;

    fn col_len(&self) -> usize {
        ColKey::LEN.cast()
    }

    fn row_len(&self) -> usize {
        100
    }

    fn contains(&self, _: &Self::Key) -> bool {
        // we know both keys are valid and length is fixed
        true
    }

    fn get_cloned(&self, key: &Self::Key) -> Option<Self::Item> {
        Some(
            self.cells
                .borrow()
                .get(key)
                .map(|cell| cell.display.clone())
                .unwrap_or("".to_string()),
        )
    }

    fn update(&self, _: &Self::Key, _: Self::Item) -> Option<UpdateHandle> {
        None
    }

    fn col_iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::ColKey> {
        ColKey::iter_keys().skip(start).take(limit).collect()
    }

    fn row_iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::RowKey> {
        (0..=99).skip(start).take(limit).collect()
    }

    fn make_key(col: &Self::ColKey, row: &Self::RowKey) -> Self::Key {
        (*col, *row)
    }
}

impl MatrixDataMut for CellData {
    fn set(&mut self, key: &Self::Key, item: Self::Item) {
        let cell = Cell::new(item, None);
        self.cells.get_mut().insert(*key, cell);
    }
}

impl UpdatableHandler<(ColKey, u8), String> for CellData {
    fn handle(&self, key: &(ColKey, u8), msg: &String) -> Option<UpdateHandle> {
        let cell = Cell::new(msg.clone(), None); // TODO: formula
        self.cells.borrow_mut().insert(key.clone(), cell);
        // TODO: update cells where needed
        Some(self.update)
    }
}

#[derive(Clone, Debug)]
struct CellGuard;
impl EditGuard for CellGuard {
    type Msg = String;

    fn activate(edit: &mut EditField<Self>, mgr: &mut Manager) -> Option<Self::Msg> {
        Self::focus_lost(edit, mgr)
    }

    fn focus_lost(edit: &mut EditField<Self>, _: &mut Manager) -> Option<Self::Msg> {
        Some(edit.get_string())
    }
}

#[derive(Debug)]
struct CellDriver;

impl Driver<String> for CellDriver {
    type Msg = String;
    type Widget = EditField<CellGuard>;

    fn new(&self) -> Self::Widget {
        EditField::new("".to_string()).with_guard(CellGuard)
    }

    fn set(&self, widget: &mut Self::Widget, data: String) -> TkAction {
        widget.set_string(data)
    }
}

pub fn window() -> Box<dyn kas::Window> {
    let mut data = CellData::new();
    data.cells
        .get_mut()
        .insert((ColKey(b'B'), 0), Cell::new("Example".to_string(), None));
    data.cells.get_mut().insert(
        (ColKey(b'B'), 1),
        Cell::new(
            "= 5 / 2".to_string(),
            Some(Formula::Div(
                Box::new(Formula::Val(5.0)),
                Box::new(Formula::Val(2.0)),
            )),
        ),
    );
    data.eval_all();

    let view = CellDriver;
    let cells = MatrixView::new_with_view(view, data).map_msg_discard::<VoidMsg>();

    Box::new(Window::new("Cells", cells))
}

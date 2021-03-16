// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::adapter::ReserveP;
use kas::data::{MatrixData, MatrixDataMut, SharedData, SharedDataRec};
use kas::prelude::*;
use kas::widget::view::{Driver, MatrixView};
use kas::widget::{Label, Window};
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
    cells: HashMap<Key, Cell>,
}

impl CellData {
    fn new() -> Self {
        CellData {
            cells: HashMap::new(),
        }
    }
    fn eval_all(&mut self) {
        for cell in self.cells.values_mut() {
            cell.eval();
        }
    }
}

impl SharedData for CellData {
    fn update_handle(&self) -> Option<UpdateHandle> {
        None
    }
}
impl SharedDataRec for CellData {}

impl MatrixData for CellData {
    type ColKey = ColKey;
    type RowKey = u8;
    type Item = String;

    fn col_len(&self) -> usize {
        ColKey::LEN.cast()
    }

    fn row_len(&self) -> usize {
        100
    }

    fn contains(&self, _: &Self::ColKey, _: &Self::RowKey) -> bool {
        // we know both keys are valid and length is fixed
        true
    }

    fn get_cloned(&self, col: &Self::ColKey, row: &Self::RowKey) -> Option<Self::Item> {
        let key = (*col, *row);
        Some(
            self.cells
                .get(&key)
                .map(|cell| cell.display.clone())
                .unwrap_or("".to_string()),
        )
    }

    fn update(&self, _: &Self::ColKey, _: &Self::RowKey, _: Self::Item) -> Option<UpdateHandle> {
        None
    }

    fn col_iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::ColKey> {
        ColKey::iter_keys().skip(start).take(limit).collect()
    }

    fn row_iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::RowKey> {
        (0..=99).skip(start).take(limit).collect()
    }
}

impl MatrixDataMut for CellData {
    fn set(&mut self, col: &Self::ColKey, row: &Self::RowKey, item: Self::Item) {
        let key = (*col, *row);
        let cell = Cell::new(item, None);
        self.cells.insert(key, cell);
    }
}

fn reserve(size_handle: &mut dyn SizeHandle, _axis: AxisInfo) -> SizeRules {
    SizeRules::fixed(size_handle.pixels_from_em(5.0).cast_nearest(), (0, 0))
}

#[derive(Debug)]
struct CellDriver;

impl Driver<Key, String> for CellDriver {
    type Widget = ReserveP<Label<String>>;

    fn default(&self) -> Self::Widget {
        Label::new("".to_string()).reserve(reserve)
    }
    fn new(&self, _: Key, data: String) -> Self::Widget {
        Label::new(data).reserve(reserve)
    }

    fn set(&self, widget: &mut Self::Widget, _: Key, data: String) -> TkAction {
        widget.set_string(data)
    }

    fn get(&self, _: &Self::Widget, _: &Key) -> Option<String> {
        None
    }
}

pub fn window() -> Box<dyn kas::Window> {
    let mut data = CellData::new();
    data.cells
        .insert((ColKey(b'B'), 0), Cell::new("Example".to_string(), None));
    data.cells.insert(
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

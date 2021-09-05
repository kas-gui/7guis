// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Cells: a mini spreadsheet

use kas::prelude::*;
use kas::updatable::{MatrixData, RecursivelyUpdatable, Updatable, UpdatableHandler};
use kas::widgets::view::{Driver, MatrixView};
use kas::widgets::{EditBox, EditField, EditGuard, Window};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ColKey(u8);
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

impl fmt::Display for ColKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = [self.0];
        write!(f, "{}", std::str::from_utf8(&b).unwrap())
    }
}

pub type Key = (ColKey, u8);

fn make_key(k: &str) -> Key {
    let col = ColKey::try_from_u8(k.as_bytes()[0]).unwrap();
    let row: u8 = k[1..].parse().unwrap();
    (col, row)
}

#[derive(Debug, PartialEq, Eq)]
enum EvalError {
    /// Value we depend on is missing
    Dependancy,
}

#[derive(Debug, PartialEq)]
pub enum Formula {
    Value(f64),
    Reference(Key),
    /// List of values to add/subtract; if bool is true then subtract
    Summation(Vec<(Formula, bool)>),
    /// List of values to multiply/divide; if bool is true then divide
    Product(Vec<(Formula, bool)>),
}

impl Formula {
    fn eval(&self, values: &HashMap<Key, f64>) -> Result<f64, EvalError> {
        use Formula::*;
        Ok(match self {
            Value(x) => *x,
            Reference(key) => return values.get(key).cloned().ok_or(EvalError::Dependancy),
            Summation(v) => {
                let mut sum = 0.0;
                for (f, neg) in v {
                    let x = f.eval(values)?;
                    if *neg {
                        sum -= x;
                    } else {
                        sum += x;
                    }
                }
                sum
            }
            Product(v) => {
                let mut prod = 1.0;
                for (f, div) in v {
                    let x = f.eval(values)?;
                    if *div {
                        prod /= x;
                    } else {
                        prod *= x;
                    }
                }
                prod
            }
        })
    }
}

mod parser {
    use super::{ColKey, Formula};
    use pest::iterators::Pairs;
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "cells.pest"]
    pub struct FormulaParser;

    fn parse_value<'a>(mut pairs: Pairs<'a, Rule>) -> Formula {
        let pair = pairs.next().unwrap();
        assert!(pairs.next().is_none());
        match pair.as_rule() {
            Rule::number => Formula::Value(pair.as_span().as_str().parse().unwrap()),
            Rule::reference => {
                let s = pair.as_span().as_str();
                assert!(s.len() >= 2);
                let mut col = s.as_bytes()[0];
                if col > b'Z' {
                    col -= b'a' - b'A';
                }
                let col = ColKey::try_from_u8(col).unwrap();
                let row = s[1..].parse().unwrap();
                let key = (col, row);
                Formula::Reference(key)
            }
            Rule::expression => parse_expression(pair.into_inner()),
            _ => unreachable!(),
        }
    }

    fn parse_product<'a>(mut pairs: Pairs<'a, Rule>) -> Formula {
        let mut product = vec![];
        let mut div = false;
        while let Some(pair) = pairs.next() {
            match pair.as_rule() {
                Rule::product_op => {
                    if pair.as_span().as_str() == "/" {
                        div = true;
                    }
                }
                Rule::value => {
                    let formula = parse_value(pair.into_inner());
                    product.push((formula, div));
                    div = false;
                }
                _ => unreachable!(),
            }
        }
        debug_assert!(div == false);
        if product.len() == 1 {
            debug_assert!(product[0].1 == false);
            product.pop().unwrap().0
        } else {
            debug_assert!(product.len() > 1);
            Formula::Product(product)
        }
    }

    fn parse_summation<'a>(mut pairs: Pairs<'a, Rule>) -> Formula {
        let mut summation = vec![];
        let mut sub = false;
        while let Some(pair) = pairs.next() {
            match pair.as_rule() {
                Rule::sum_op => {
                    if pair.as_span().as_str() == "-" {
                        sub = true;
                    }
                }
                Rule::product => {
                    let formula = parse_product(pair.into_inner());
                    summation.push((formula, sub));
                    sub = false;
                }
                _ => unreachable!(),
            }
        }
        debug_assert!(sub == false);
        if summation.len() == 1 && summation[0].1 == false {
            summation.pop().unwrap().0
        } else {
            debug_assert!(summation.len() > 1);
            Formula::Summation(summation)
        }
    }

    fn parse_expression<'a>(mut pairs: Pairs<'a, Rule>) -> Formula {
        let pair = pairs.next().unwrap();
        assert!(pairs.next().is_none());
        assert_eq!(pair.as_rule(), Rule::expression);
        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        assert!(pairs.next().is_none());
        assert_eq!(pair.as_rule(), Rule::summation);
        parse_summation(pair.into_inner())
    }

    pub fn parse(source: &str) -> Result<Option<Formula>, ()> {
        match FormulaParser::parse(Rule::cell, source) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap();
                Ok(match pair.as_rule() {
                    Rule::formula => Some(parse_expression(pair.into_inner())),
                    Rule::text => None,
                    _ => unreachable!(),
                })
            }
            Err(error) => {
                println!("Error: {}", error);
                Err(())
            }
        }
    }
}

#[derive(Debug, Default)]
struct Cell {
    input: String,
    formula: Option<Formula>,
    parse_error: bool,
    display: String,
}

impl Cell {
    fn new<T: ToString>(input: T) -> Self {
        let input = input.to_string();
        let result = parser::parse(&input);
        let parse_error = result.is_err();
        Cell {
            input,
            formula: result.ok().flatten(),
            parse_error,
            display: String::new(),
        }
    }

    /// Get display string
    fn display(&self) -> String {
        if self.display.len() > 0 {
            self.display.clone()
        } else {
            self.input.clone()
        }
    }

    fn try_eval(&mut self, values: &HashMap<Key, f64>) -> Result<Option<f64>, EvalError> {
        if let Some(ref f) = self.formula {
            let value = f.eval(values)?;
            self.display = value.to_string();
            Ok(Some(value))
        } else {
            Ok(self.input.parse().ok())
        }
    }
}

#[derive(Debug)]
struct CellDataInner {
    cells: HashMap<Key, Cell>,
    values: HashMap<Key, f64>,
}

impl CellDataInner {
    fn new() -> Self {
        CellDataInner {
            cells: HashMap::new(),
            values: HashMap::new(),
        }
    }
    fn update_values(&mut self) {
        // NOTE: this is a fairly naive algorithm, but correct!
        self.values.clear();

        let mut waiting = vec![];
        for (key, cell) in self.cells.iter_mut() {
            match cell.try_eval(&self.values) {
                Ok(Some(value)) => {
                    self.values.insert(*key, value);
                }
                Ok(None) => (),
                Err(EvalError::Dependancy) => waiting.push(*key),
            }
        }

        let mut remaining = waiting.len();
        let mut queue = vec![];

        while remaining > 0 {
            std::mem::swap(&mut waiting, &mut queue);
            for key in queue.drain(..) {
                let cell = self.cells.get_mut(&key).unwrap();
                match cell.try_eval(&self.values) {
                    Ok(Some(value)) => {
                        self.values.insert(key, value);
                    }
                    Ok(None) => (),
                    Err(EvalError::Dependancy) => waiting.push(key),
                }
            }

            if waiting.len() >= remaining {
                for key in waiting.drain(..) {
                    let cell = self.cells.get_mut(&key).unwrap();
                    cell.display = "Ref error".to_string();
                }
                return;
            } else {
                remaining = waiting.len();
            }
        }
    }
}

#[derive(Debug)]
struct CellData {
    inner: RefCell<CellDataInner>,
    update: UpdateHandle,
}

impl CellData {
    fn new() -> Self {
        CellData {
            inner: RefCell::new(CellDataInner::new()),
            update: UpdateHandle::new(),
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
    /// Item is (input_string, display_string, error_state)
    type Item = (String, String, bool);

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
        let inner = self.inner.borrow();
        let cell = inner.cells.get(key);
        Some(
            cell.map(|cell| (cell.input.clone(), cell.display().clone(), cell.parse_error))
                .unwrap_or(("".to_string(), "".to_string(), false)),
        )
    }

    fn update(&self, _: &Self::Key, _: Self::Item) -> Option<UpdateHandle> {
        None
    }

    fn col_iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::ColKey> {
        ColKey::iter_keys().skip(start).take(limit).collect()
    }

    fn row_iter_vec_from(&self, start: usize, limit: usize) -> Vec<Self::RowKey> {
        // NOTE: for strict compliance with the 7GUIs challenge the rows should
        // start from 0, but any other spreadsheet I've seen starts from 1!
        (1..=99).skip(start).take(limit).collect()
    }

    fn make_key(col: &Self::ColKey, row: &Self::RowKey) -> Self::Key {
        (*col, *row)
    }
}

impl UpdatableHandler<(ColKey, u8), String> for CellData {
    fn handle(&self, key: &(ColKey, u8), msg: &String) -> Option<UpdateHandle> {
        let cell = Cell::new(msg);
        let mut inner = self.inner.borrow_mut();
        inner.cells.insert(key.clone(), cell);
        // TODO: we should not recompute everything here!
        inner.update_values();
        Some(self.update)
    }
}

#[derive(Clone, Default, Debug)]
struct CellGuard {
    input: String,
}
impl EditGuard for CellGuard {
    type Msg = String;

    fn activate(edit: &mut EditField<Self>, mgr: &mut Manager) -> Option<Self::Msg> {
        Self::focus_lost(edit, mgr)
    }

    fn focus_gained(edit: &mut EditField<Self>, mgr: &mut Manager) {
        let mut s = String::default();
        std::mem::swap(&mut edit.guard.input, &mut s);
        *mgr |= edit.set_string(s);
    }

    fn focus_lost(edit: &mut EditField<Self>, _: &mut Manager) -> Option<Self::Msg> {
        Some(edit.get_string())
    }
}

#[derive(Debug)]
struct CellDriver;

impl Driver<(String, String, bool)> for CellDriver {
    type Msg = String;
    // TODO: we should use EditField instead of EditBox but:
    // (a) there is currently no code to draw separators between cells
    // (b) EditField relies on a parent (EditBox) to draw background highlight on error state
    type Widget = EditBox<CellGuard>;

    fn new(&self) -> Self::Widget {
        EditBox::new("".to_string()).with_guard(CellGuard::default())
    }

    fn set(&self, edit: &mut Self::Widget, data: (String, String, bool)) -> TkAction {
        edit.guard.input = data.0;
        edit.set_error_state(data.2);
        if edit.has_key_focus() {
            // assume that the contents of the EditBox are the latest
            TkAction::empty()
        } else {
            edit.set_string(data.1)
        }
    }
}

pub fn window() -> Box<dyn kas::Window> {
    let mut data = CellData::new();
    let inner = data.inner.get_mut();
    inner.cells.insert(make_key("A1"), Cell::new("Some values"));
    inner.cells.insert(make_key("A2"), Cell::new("3"));
    inner.cells.insert(make_key("A3"), Cell::new("4"));
    inner.cells.insert(make_key("A4"), Cell::new("5"));
    inner.cells.insert(make_key("B1"), Cell::new("Sum"));
    inner
        .cells
        .insert(make_key("B2"), Cell::new("= A2 + A3 + A4"));
    inner.cells.insert(make_key("C1"), Cell::new("Prod"));
    inner
        .cells
        .insert(make_key("C2"), Cell::new("= A2 * A3 * A4"));
    inner.update_values();

    let view = CellDriver;
    let cells = MatrixView::new_with_driver(view, data)
        .with_num_visible(5, 20)
        .map_msg_discard::<VoidMsg>();

    Box::new(Window::new("Cells", cells))
}

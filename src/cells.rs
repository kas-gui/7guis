// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Create Read Update Delete

use kas::prelude::*;
use kas::updatable::{RecursivelyUpdatable, Updatable, UpdatableHandler};
use kas::widget::view::{Driver, MatrixData, MatrixView};
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

type Key = (ColKey, u8);

#[derive(Debug, PartialEq)]
pub enum Formula {
    Value(f64),
    // Ref(String),
    /// List of values to add/subtract; if bool is true then subtract
    Summation(Vec<(Formula, bool)>),
    /// List of values to multiply/divide; if bool is true then divide
    Product(Vec<(Formula, bool)>),
}

impl Formula {
    fn eval(&self) -> f64 {
        use Formula::*;
        match self {
            Value(x) => *x,
            Summation(v) => v.iter().fold(0.0, |sum, (f, neg)| {
                let x = f.eval();
                if *neg {
                    sum - x
                } else {
                    sum + x
                }
            }),
            Product(v) => v.iter().fold(1.0, |prod, (f, div)| {
                let x = f.eval();
                if *div {
                    prod / x
                } else {
                    prod * x
                }
            }),
        }
    }
}

mod parser {
    use super::Formula;
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
    fn new(input: String) -> Self {
        let result = parser::parse(&input);
        let parse_error = result.is_err();
        Cell {
            input,
            formula: result.ok().flatten(),
            parse_error,
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
        Some(
            self.cells
                .borrow()
                .get(key)
                .map(|cell| (cell.input.clone(), cell.display.clone(), cell.parse_error))
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
        (0..=99).skip(start).take(limit).collect()
    }

    fn make_key(col: &Self::ColKey, row: &Self::RowKey) -> Self::Key {
        (*col, *row)
    }
}

impl UpdatableHandler<(ColKey, u8), String> for CellData {
    fn handle(&self, key: &(ColKey, u8), msg: &String) -> Option<UpdateHandle> {
        let mut cell = Cell::new(msg.clone());
        cell.eval();
        self.cells.borrow_mut().insert(key.clone(), cell);
        // TODO: update cells where needed
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

    fn focus_gained(edit: &mut EditField<Self>, mgr: &mut Manager) -> Option<Self::Msg> {
        let mut s = String::default();
        std::mem::swap(&mut edit.guard.input, &mut s);
        *mgr |= edit.set_string(s);
        None
    }

    fn focus_lost(edit: &mut EditField<Self>, _: &mut Manager) -> Option<Self::Msg> {
        Some(edit.get_string())
    }
}

#[derive(Debug)]
struct CellDriver;

impl Driver<(String, String, bool)> for CellDriver {
    type Msg = String;
    type Widget = EditField<CellGuard>;

    fn new(&self) -> Self::Widget {
        EditField::new("".to_string()).with_guard(CellGuard::default())
    }

    fn set(&self, edit: &mut Self::Widget, data: (String, String, bool)) -> TkAction {
        edit.guard.input = data.0;
        edit.set_error_state(data.2);
        if edit.has_key_focus() {
            // assume that the contents of the EditField are the latest
            TkAction::empty()
        } else {
            edit.set_string(data.1)
        }
    }
}

pub fn window() -> Box<dyn kas::Window> {
    let mut data = CellData::new();
    data.cells
        .get_mut()
        .insert((ColKey(b'B'), 0), Cell::new("Example".to_string()));
    data.cells
        .get_mut()
        .insert((ColKey(b'B'), 1), Cell::new("= 5 / 2".to_string()));
    data.eval_all();

    let view = CellDriver;
    let cells = MatrixView::new_with_driver(view, data)
        .with_num_visible(5, 20)
        .map_msg_discard::<VoidMsg>();

    Box::new(Window::new("Cells", cells))
}

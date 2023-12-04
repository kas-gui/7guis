// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Cells: a mini spreadsheet

use kas::event::{Command, FocusSource};
use kas::prelude::*;
use kas::view::{DataKey, Driver, MatrixData, MatrixView, SharedData};
use kas::widgets::{EditBox, EditField, EditGuard, ScrollBars};
use std::collections::HashMap;
use std::{fmt, iter, ops};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct ColKey(u8);
type ColKeyIter = iter::Map<ops::RangeInclusive<u8>, fn(u8) -> ColKey>;
impl ColKey {
    const LEN: u8 = 26;
    fn try_from_u8(n: u8) -> Option<Self> {
        if (b'A'..=b'Z').contains(&n) {
            Some(ColKey(n))
        } else {
            None
        }
    }
    fn from_u8(n: u8) -> Self {
        Self::try_from_u8(n).expect("bad column key")
    }
    fn iter_keys() -> ColKeyIter {
        (b'A'..=b'Z').map(ColKey::from_u8)
    }
}

impl fmt::Display for ColKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = [self.0];
        write!(f, "{}", std::str::from_utf8(&b).unwrap())
    }
}

const MAX_ROW: u8 = 99;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Key(ColKey, u8);
impl DataKey for Key {
    fn make_id(&self, parent: &Id) -> Id {
        assert_eq!(std::mem::size_of::<ColKey>(), 1);
        let key = (((self.0).0 as usize) << 8) | (self.1 as usize);
        parent.make_child(key)
    }

    fn reconstruct_key(parent: &Id, child: &Id) -> Option<Self> {
        child.next_key_after(parent).map(|key| {
            let col = ColKey((key >> 8) as u8);
            let row = key as u8;
            Key(col, row)
        })
    }
}

fn make_key(k: &str) -> Key {
    let col = ColKey::from_u8(k.as_bytes()[0]);
    let row: u8 = k[1..].parse().unwrap();
    Key(col, row)
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
    use super::{ColKey, Formula, Key};
    use pest::error::Error;
    use pest::iterators::Pairs;
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "cells.pest"]
    pub struct FormulaParser;

    fn parse_value(mut pairs: Pairs<'_, Rule>) -> Formula {
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
                let col = ColKey::from_u8(col);
                let row = s[1..].parse().unwrap();
                let key = Key(col, row);
                Formula::Reference(key)
            }
            Rule::expression => parse_expression(pair.into_inner()),
            _ => unreachable!(),
        }
    }

    fn parse_product(pairs: Pairs<'_, Rule>) -> Formula {
        let mut product = vec![];
        let mut div = false;
        for pair in pairs {
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
        debug_assert!(!div);
        if product.len() == 1 {
            debug_assert!(!product[0].1);
            product.pop().unwrap().0
        } else {
            debug_assert!(product.len() > 1);
            Formula::Product(product)
        }
    }

    fn parse_summation(pairs: Pairs<'_, Rule>) -> Formula {
        let mut summation = vec![];
        let mut sub = false;
        for pair in pairs {
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
        debug_assert!(!sub);
        if summation.len() == 1 && !summation[0].1 {
            summation.pop().unwrap().0
        } else {
            debug_assert!(summation.len() > 1);
            Formula::Summation(summation)
        }
    }

    fn parse_expression(mut pairs: Pairs<'_, Rule>) -> Formula {
        let pair = pairs.next().unwrap();
        assert!(pairs.next().is_none());
        assert_eq!(pair.as_rule(), Rule::expression);
        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        assert!(pairs.next().is_none());
        assert_eq!(pair.as_rule(), Rule::summation);
        parse_summation(pair.into_inner())
    }

    pub fn parse(source: &str) -> Result<Option<Formula>, Error<Rule>> {
        FormulaParser::parse(Rule::cell, source).map(|mut pairs| {
            let pair = pairs.next().unwrap();
            match pair.as_rule() {
                Rule::formula => Some(parse_expression(pair.into_inner())),
                Rule::text => None,
                _ => unreachable!(),
            }
        })
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
        let mut cell = Cell::default();
        cell.update(input.to_string());
        cell
    }

    fn update(&mut self, input: String) {
        match parser::parse(&input) {
            Ok(opt_formula) => {
                self.formula = opt_formula;
                self.parse_error = false;
            }
            Err(error) => {
                println!("Parse error: {error}");
                self.display = "BAD FORMULA".to_string();
                self.parse_error = true;
            }
        }
        self.input = input;
    }

    /// Get display string
    fn display(&self) -> String {
        if !self.display.is_empty() {
            self.display.clone()
        } else {
            self.input.clone()
        }
    }

    fn try_eval(&mut self, values: &HashMap<Key, f64>) -> Result<Option<f64>, EvalError> {
        if self.parse_error {
            // Display the error locally; propegate NaN
            Ok(Some(f64::NAN))
        } else if let Some(ref f) = self.formula {
            let value = f.eval(values)?;
            self.display = value.to_string();
            Ok(Some(value))
        } else {
            Ok(self.input.parse().ok())
        }
    }
}

#[derive(Debug)]
struct CellData {
    cells: HashMap<Key, Cell>,
    values: HashMap<Key, f64>,
}

impl CellData {
    fn new() -> Self {
        CellData {
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

#[derive(Clone, Debug, Default)]
struct Item {
    input: String,
    display: String,
    error: bool,
}

impl SharedData for CellData {
    type Key = Key;
    type Item = Item;
    type ItemRef<'b> = Self::Item;

    fn contains_key(&self, _: &Self::Key) -> bool {
        // we know both sub-keys are valid and that the length is fixed
        true
    }

    fn borrow(&self, key: &Self::Key) -> Option<Self::Item> {
        self.cells
            .get(key)
            .map(|cell| Item {
                input: cell.input.clone(),
                display: cell.display(),
                error: cell.parse_error,
            })
            .or_else(|| Some(Item::default()))
    }
}

impl MatrixData for CellData {
    type ColKey = ColKey;
    type RowKey = u8;
    type ColKeyIter<'b> = iter::Take<iter::Skip<ColKeyIter>>;
    type RowKeyIter<'b> = iter::Take<iter::Skip<ops::RangeInclusive<u8>>>;

    fn is_empty(&self) -> bool {
        false
    }
    fn len(&self) -> (usize, usize) {
        (ColKey::LEN.cast(), 99)
    }

    fn col_iter_from(&self, start: usize, limit: usize) -> Self::ColKeyIter<'_> {
        ColKey::iter_keys().skip(start).take(limit)
    }

    fn row_iter_from(&self, start: usize, limit: usize) -> Self::RowKeyIter<'_> {
        // NOTE: for strict compliance with the 7GUIs challenge the rows should
        // start from 0, but any other spreadsheet I've seen starts from 1!
        (1..=MAX_ROW).skip(start).take(limit)
    }

    fn make_key(&self, col: &Self::ColKey, row: &Self::RowKey) -> Self::Key {
        Key(*col, *row)
    }
}

#[derive(Debug)]
struct UpdateInput(Key, String);

#[derive(Clone, Default, Debug)]
struct CellGuard {
    key: Key,
    is_input: bool,
}
impl EditGuard for CellGuard {
    type Data = Item;

    fn update(edit: &mut EditField<Self>, cx: &mut ConfigCx, item: &Item) {
        let mut action = edit.set_error_state(item.error);
        if !edit.has_edit_focus() {
            action |= edit.set_str(&item.display);
            edit.guard.is_input = false;
        }
        cx.action(edit, action);
    }

    fn activate(edit: &mut EditField<Self>, cx: &mut EventCx, item: &Item) -> IsUsed {
        Self::focus_lost(edit, cx, item);
        IsUsed::Used
    }

    fn focus_gained(edit: &mut EditField<Self>, cx: &mut EventCx, item: &Item) {
        cx.action(edit.id(), edit.set_str(&item.input));
        edit.guard.is_input = true;
    }

    fn focus_lost(edit: &mut EditField<Self>, cx: &mut EventCx, item: &Item) {
        let s = edit.get_string();
        if edit.guard.is_input && s != item.input {
            cx.push(UpdateInput(edit.guard.key, s));
        }
    }
}

#[derive(Debug)]
struct CellDriver;

impl Driver<Item, CellData> for CellDriver {
    // TODO: we should use EditField instead of EditBox but:
    // (a) there is currently no code to draw separators between cells
    // (b) EditField relies on a parent (EditBox) to draw background highlight on error state
    type Widget = EditBox<CellGuard>;

    fn make(&mut self, key: &Key) -> Self::Widget {
        EditBox::new(CellGuard {
            key: *key,
            is_input: false,
        })
    }
}

pub fn window() -> Window<()> {
    let mut data = CellData::new();
    let cells = &mut data.cells;
    cells.insert(make_key("A1"), Cell::new("Some values"));
    cells.insert(make_key("A2"), Cell::new("3"));
    cells.insert(make_key("A3"), Cell::new("4"));
    cells.insert(make_key("A4"), Cell::new("5"));
    cells.insert(make_key("B1"), Cell::new("Sum"));
    cells.insert(make_key("B2"), Cell::new("= A2 + A3 + A4"));
    cells.insert(make_key("C1"), Cell::new("Prod"));
    cells.insert(make_key("C2"), Cell::new("= A2 * A3 * A4"));
    data.update_values();

    let cells = MatrixView::new(CellDriver).with_num_visible(5, 20);

    let ui = impl_anon! {
        #[widget {
            layout = self.cells;
        }]
        struct {
            core: widget_core!(),
            data: CellData = data,
            #[widget(&self.data)] cells: ScrollBars<MatrixView<CellData, CellDriver>> =
                ScrollBars::new(cells),
        }
        impl Events for Self {
            type Data = ();

            fn steal_event(&mut self, cx: &mut EventCx, _: &(), _: &Id, event: &Event) -> IsUsed {
                match event {
                    Event::Command(Command::Enter, _) => {
                        if let Some(Key(col, row)) = cx.nav_focus().and_then(|id| {
                            Key::reconstruct_key(self.cells.inner().id_ref(), id)
                        })
                        {
                            let row = if cx.modifiers().shift_key() {
                                (row - 1).max(1)
                            } else {
                                (row + 1).min(MAX_ROW)
                            };
                            let id = Key(col, row).make_id(self.cells.inner().id_ref());
                            cx.next_nav_focus(Some(id), false, FocusSource::Synthetic);
                        }
                        IsUsed::Used
                    },
                    _ => IsUsed::Unused
                }
            }

            fn handle_messages(&mut self, cx: &mut EventCx, _: &()) {
                if let Some(UpdateInput(key, input)) = cx.try_pop() {
                    self.data.cells.entry(key).or_default().update(input);
                    self.data.update_values();
                }
            }
        }
    };
    Window::new(ui, "Cells")
}

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Cells: a mini spreadsheet

use kas::view::{
    DataChanges, DataClerk, DataKey, DataLen, Driver, GridIndex, GridView, TokenChanges,
};
use kas::widgets::{EditBox, EditField, EditGuard, ScrollBars};
use kas::{prelude::*, TextOrSource};
use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct ColKey(u8);
impl ColKey {
    const LEN: u8 = 26;
    fn try_from_u8(n: u8) -> Option<Self> {
        if (b'A'..=b'Z').contains(&n) {
            Some(ColKey(n - b'A'))
        } else {
            None
        }
    }
    fn from_u8(n: u8) -> Self {
        Self::try_from_u8(n).expect("bad column key")
    }
}

impl fmt::Display for ColKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = [self.0];
        write!(f, "{}", std::str::from_utf8(&b).unwrap())
    }
}

const ROW_LEN: u32 = 100;

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
                    div = match pair.as_span().as_str() {
                        "*" => false,
                        "/" => true,
                        other => panic!("expected `*` or `/`, found `{other}`"),
                    };
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
                    sub = match pair.as_span().as_str() {
                        "+" => false,
                        "-" => true,
                        other => panic!("expected `+` or `-`, found `{other}`"),
                    };
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
        if let Some(pair) = pairs.next() {
            if pair.as_rule() != Rule::EOI {
                panic!("unexpected next pair: {pair:?}");
            }
        }
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

struct Clerk {
    empty_cell: Cell,
}

impl DataClerk<GridIndex> for Clerk {
    type Data = CellData;
    type Key = Key;
    type Item = Cell;
    type Token = Key;

    fn update(&mut self, _: &mut ConfigCx<'_>, _: Id, _: &Self::Data) -> DataChanges {
        DataChanges::Any
    }

    fn len(&self, _: &CellData, _: GridIndex) -> DataLen<GridIndex> {
        DataLen::Known(GridIndex {
            col: ColKey::LEN.cast(),
            row: ROW_LEN,
        })
    }

    fn update_token(
        &self,
        _: &CellData,
        index: GridIndex,
        _: bool,
        token: &mut Option<Key>,
    ) -> TokenChanges {
        if index.col >= ColKey::LEN as u32 || index.row >= ROW_LEN {
            *token = None;
            return TokenChanges::Any;
        }

        let key = Key(ColKey(index.col as u8), index.row as u8);
        if *token == Some(key) {
            TokenChanges::None
        } else {
            *token = Some(key);
            TokenChanges::Any
        }
    }

    fn item<'r>(&'r self, data: &'r CellData, key: &'r Key) -> &'r Cell {
        data.cells.get(key).unwrap_or(&self.empty_cell)
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
    type Data = Cell;

    fn update(edit: &mut EditField<Self>, cx: &mut ConfigCx, item: &Cell) {
        edit.set_error_state(cx, item.parse_error);
        if !edit.has_edit_focus() {
            let text = if !item.display.is_empty() {
                &item.display
            } else {
                &item.input
            };
            edit.set_str(cx, text);
            edit.guard.is_input = false;
        }
    }

    fn activate(edit: &mut EditField<Self>, cx: &mut EventCx, item: &Cell) -> IsUsed {
        Self::focus_lost(edit, cx, item);
        IsUsed::Used
    }

    fn focus_gained(edit: &mut EditField<Self>, cx: &mut EventCx, item: &Cell) {
        edit.set_str(cx, &item.input);
        edit.guard.is_input = true;
    }

    fn focus_lost(edit: &mut EditField<Self>, cx: &mut EventCx, item: &Cell) {
        if edit.guard.is_input && edit.as_str() != item.input {
            cx.push(UpdateInput(edit.guard.key, edit.clone_string()));
        }
    }
}

#[derive(Debug)]
struct CellDriver;

impl Driver<Key, Cell> for CellDriver {
    // TODO: we should use EditField instead of EditBox but:
    // (a) there is currently no code to draw separators between cells
    // (b) EditField relies on a parent (EditBox) to draw background highlight on error state
    type Widget = EditBox<CellGuard>;

    fn make(&mut self, key: &Key) -> Self::Widget {
        EditBox::new(CellGuard {
            key: *key,
            is_input: false,
        })
        .with_width_em(6.0, 6.0)
    }

    fn navigable(_: &Self::Widget) -> bool {
        false
    }

    fn label(widget: &Self::Widget) -> Option<TextOrSource<'_>> {
        Some(widget.as_str().into())
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

    let clerk = Clerk {
        empty_cell: Cell::default(),
    };

    let cells = GridView::new(clerk, CellDriver).with_num_visible(5, 20);

    let ui = impl_anon! {
        #[widget]
        #[layout(self.cells)]
        struct {
            core: widget_core!(),
            data: CellData = data,
            #[widget(&self.data)] cells: ScrollBars<GridView<Clerk, CellDriver>> =
                ScrollBars::new(cells),
        }
        impl Events for Self {
            type Data = ();

            fn handle_messages(&mut self, cx: &mut EventCx, _: &()) {
                if let Some(UpdateInput(key, input)) = cx.try_pop() {
                    self.data.cells.entry(key).or_default().update(input);
                    self.data.update_values();
                    cx.update(self.cells.as_node(&self.data));
                }
            }
        }
    };
    Window::new(ui, "Cells")
}

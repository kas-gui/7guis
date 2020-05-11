// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Flight booker
#![feature(proc_macro_hygiene)]

use chrono::{Duration, Local, NaiveDate};
use kas::class::HasText;
use kas::event::VoidResponse;
use kas::prelude::*;
use kas::widget::{ComboBox, EditBox, EditGuard, MessageBox, TextButton, Window};
use kas_wgpu::{kas, theme};

#[derive(Clone, Debug, PartialEq, Eq, VoidMsg)]
enum Flight {
    OneWay,
    Return,
}

#[derive(Clone, Debug)]
struct Guard {
    date: Option<NaiveDate>,
}
impl Guard {
    fn new() -> Self {
        Guard { date: None }
    }
}
impl EditGuard for Guard {
    type Msg = ();
    fn edit(edit: &mut EditBox<Self>) -> Option<()> {
        let date = NaiveDate::parse_from_str(&edit.get_text(), "%Y-%m-%d");
        edit.guard.date = match date {
            Ok(date) => Some(date),
            Err(e) => {
                // TODO: display error in GUI
                println!("Error parsing date: {}", e);
                None
            }
        };
        edit.set_error_state(edit.guard.date.is_none());

        // On any change, we notify the parent that it should call date():
        Some(())
    }
}

fn main() -> Result<(), kas_wgpu::Error> {
    // Default dates:
    let out = Local::today();
    let back = out + Duration::days(7);

    let mut d1 = EditBox::new(out.format("%Y-%m-%d").to_string()).with_guard(Guard::new());
    let mut d2 = EditBox::new(back.format("%Y-%m-%d").to_string()).with_guard(Guard::new());

    // Run edit guards once to set date value
    // TODO: a content-centric widget type (with a trait to parse and format
    // values) would make this a little easier
    Guard::edit(&mut d1);
    Guard::edit(&mut d2);
    let _ = d2.set_disabled(true);

    let window = Window::new(
        "Flight Booker",
        make_widget! {
            #[widget]
            #[layout(column)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(handler = combo)] _: ComboBox<Flight> = [
                    ("One-way flight", Flight::OneWay),
                    ("Return flight", Flight::Return)
                ].iter().cloned().collect(),
                #[widget(handler = date)] d1: EditBox<Guard> = d1,
                #[widget(handler = date)] d2: EditBox<Guard> = d2,
                #[widget(handler = book)] book = TextButton::new("Book", ()),
            }
            impl {
                fn combo(&mut self, mgr: &mut Manager, msg: Flight) -> VoidResponse {
                    *mgr += self.d2.set_disabled(msg == Flight::OneWay);
                    self.date(mgr, ())
                }
                fn date(&mut self, mgr: &mut Manager, _: ()) -> VoidResponse {
                    let is_ready = match self.d1.guard.date {
                        None => false,
                        Some(_) if self.d2.is_disabled() => true,
                        Some(d1) => {
                            match self.d2.guard.date {
                                None => false,
                                Some(d2) => {
                                    // TODO: display error in GUI
                                    if !(d1 < d2) {
                                        println!("Out-bound flight must be before return flight!");
                                    }
                                    d1 < d2
                                }
                            }
                        }
                    };
                    *mgr += self.book.set_disabled(!is_ready);
                    VoidResponse::None
                }
                fn book(&mut self, mgr: &mut Manager, _: ()) -> VoidResponse {
                    let d1 = self.d1.guard.date.unwrap();
                    let msg = if self.d2.is_disabled() {
                        format!("You have booked a one-way flight on {}", d1.format("%Y-%m-%d"))
                    } else {
                        let d2 = self.d2.guard.date.unwrap();
                        format!(
                            "You have booked an out-bound flight on {} and a return flight on {}",
                            d1.format("%Y-%m-%d"),
                            d2.format("%Y-%m-%d"),
                        )
                    };
                    mgr.add_window(Box::new(MessageBox::new("Booker result", msg)));
                    VoidResponse::None
                }
            }
        },
    );

    let theme = theme::ShadedTheme::new();
    let mut toolkit = kas_wgpu::Toolkit::new(theme)?;
    toolkit.add(window)?;
    toolkit.run()
}

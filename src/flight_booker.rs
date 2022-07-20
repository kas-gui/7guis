// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Flight booker

use chrono::{Duration, Local, NaiveDate};
use kas::prelude::*;
use kas::widgets::dialog::MessageBox;
use kas::widgets::menu::MenuEntry;
use kas::widgets::{ComboBox, EditBox, EditField, EditGuard, TextButton};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flight {
    OneWay,
    Return,
}
#[derive(Clone, Debug)]
struct ActionDates;
#[derive(Clone, Debug)]
struct ActionBook;

// TODO: consider adding a view-and-edit widget (like SingleView but supporting
// text editing) so that string representation is just a view of date repr.
#[derive(Clone, Debug)]
struct Guard {
    date: Option<NaiveDate>,
}
impl Guard {
    fn new(date: NaiveDate) -> Self {
        Guard { date: Some(date) }
    }
}
impl EditGuard for Guard {
    fn edit(edit: &mut EditField<Self>, mgr: &mut EventMgr) {
        let date = NaiveDate::parse_from_str(edit.get_str().trim(), "%Y-%m-%d");
        edit.guard.date = match date {
            Ok(date) => Some(date),
            Err(e) => {
                // TODO: display error in GUI
                println!("Error parsing date: {}", e);
                None
            }
        };
        edit.set_error_state(edit.guard.date.is_none());

        // On any change, we notify the parent that it should update the book button:
        mgr.push_msg(ActionDates);
    }
}

pub fn window() -> Box<dyn Window> {
    // Default dates:
    let out = Local::today().naive_local();
    let back = out + Duration::days(7);

    let d1 = EditBox::new(out.format("%Y-%m-%d").to_string()).with_guard(Guard::new(out));
    let d2 = EditBox::new(back.format("%Y-%m-%d").to_string()).with_guard(Guard::new(back));

    Box::new(impl_singleton! {
        #[derive(Debug)]
        #[widget {
            layout = column: [
                self.combo,
                self.d1,
                self.d2,
                self.book,
            ];
        }]
        struct {
            core: widget_core!(),
            #[widget] combo: ComboBox<Flight> = ComboBox::new_vec(vec![
                MenuEntry::new("One-way flight", Flight::OneWay),
                MenuEntry::new("Return flight", Flight::Return),
            ]),
            #[widget] d1: EditBox<Guard> = d1,
            #[widget] d2: EditBox<Guard> = d2,
            #[widget] book = TextButton::new_msg("Book", ActionBook),
        }
        impl Self {
            fn update_dates(&mut self, mgr: &mut EventMgr) {
                let is_ready = match self.d1.guard.date.as_ref() {
                    None => false,
                    Some(_) if mgr.is_disabled(self.d2.id_ref()) => true,
                    Some(d1) => {
                        match self.d2.guard.date.as_ref() {
                            None => false,
                            Some(d2) if d1 < d2 => true,
                            _ => {
                                // TODO: display error in GUI
                                println!("Out-bound flight must be before return flight!");
                                false
                            }
                        }
                    }
                };
                mgr.set_disabled(self.book.id(), !is_ready);
            }
        }
        impl Widget for Self {
            fn configure(&mut self, mgr: &mut ConfigMgr) {
                mgr.set_disabled(self.d2.id(), true);
            }
            fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
                if let Some(flight) = mgr.try_pop_msg::<Flight>() {
                    mgr.set_disabled(self.d2.id(), flight == Flight::OneWay);
                    self.update_dates(mgr);
                } else if let Some(ActionDates) = mgr.try_pop_msg() {
                    self.update_dates(mgr);
                } else if let Some(ActionBook) = mgr.try_pop_msg() {
                    let d1 = self.d1.guard.date.unwrap();
                    let msg = if mgr.is_disabled(self.d2.id_ref()) {
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
                }
            }
        }
        impl Window for Self {
            fn title(&self) -> &str {
                "Flight Booker"
            }
        }
    })
}

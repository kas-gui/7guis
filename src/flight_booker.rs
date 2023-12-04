// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Flight booker

use chrono::{Duration, Local, NaiveDate, ParseError};
use kas::prelude::*;
use kas::widgets::dialog::MessageBox;
use kas::widgets::{label_any, Adapt, Button, ComboBox, EditBox, EditField, EditGuard, Text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Flight {
    #[default]
    OneWay,
    Return,
}

#[derive(Debug)]
enum Error {
    None,
    OutParse(ParseError),
    RetParse(ParseError),
    OutBeforeToday,
    ReturnTooSoon,
}
impl Error {
    fn is_none(&self) -> bool {
        matches!(self, Error::None)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::None => Ok(()),
            Error::OutParse(err) => f.write_fmt(format_args!("Error: outbound date: {err}")),
            Error::RetParse(err) => f.write_fmt(format_args!("Error: return date: {err}")),
            Error::OutBeforeToday => f.write_str("Error: outbound date is before today!"),
            Error::ReturnTooSoon => f.write_str("Error: return date must be after outbound date!"),
        }
    }
}

#[derive(Debug)]
struct Data {
    out: Result<NaiveDate, ParseError>,
    ret: Result<NaiveDate, ParseError>,
    flight: Flight,
    error: Error,
}
impl Data {
    fn update_error(&mut self) {
        self.error = match self.out {
            Ok(out_date) => {
                if out_date < Local::now().naive_local().date() {
                    Error::OutBeforeToday
                } else {
                    match (self.flight, self.ret) {
                        (Flight::OneWay, _) => Error::None,
                        (Flight::Return, Ok(ret_date)) => {
                            if ret_date < out_date {
                                Error::ReturnTooSoon
                            } else {
                                Error::None
                            }
                        }
                        (Flight::Return, Err(err)) => Error::RetParse(err),
                    }
                }
            }
            Err(err) => Error::OutParse(err),
        };
    }
}

#[derive(Clone, Debug)]
struct ActionDate {
    result: Result<NaiveDate, ParseError>,
    is_return_field: bool,
}

#[derive(Clone, Debug)]
struct ActionBook;

#[derive(Clone, Debug)]
struct Guard {
    is_return_field: bool,
}
impl Guard {
    fn new(is_return_field: bool) -> Self {
        Guard { is_return_field }
    }
}
impl EditGuard for Guard {
    type Data = Data;

    fn edit(edit: &mut EditField<Self>, cx: &mut EventCx, _: &Self::Data) {
        let result = NaiveDate::parse_from_str(edit.get_str().trim(), "%Y-%m-%d");
        let act = edit.set_error_state(result.is_err());
        cx.action(edit.id(), act);

        cx.push(ActionDate {
            result,
            is_return_field: edit.guard.is_return_field,
        });
    }

    fn update(edit: &mut EditField<Self>, cx: &mut ConfigCx, data: &Self::Data) {
        if !edit.has_edit_focus() && edit.get_str().is_empty() {
            if let Ok(date) = match edit.guard.is_return_field {
                false => data.out,
                true => data.ret,
            } {
                let act = edit.set_string(date.format("%Y-%m-%d").to_string());
                cx.action(edit.id(), act);
            }
        }
        if edit.guard.is_return_field {
            cx.set_disabled(edit.id(), data.flight == Flight::OneWay);
        }
    }
}

pub fn window() -> Window<()> {
    let out_date = Local::now().naive_local().date();
    let data = Data {
        out: Ok(out_date),
        ret: Ok(out_date + Duration::days(7)),
        flight: Flight::OneWay,
        error: Error::None,
    };

    let ui = kas::column![
        ComboBox::new(
            [
                ("One-way flight", Flight::OneWay),
                ("Return flight", Flight::Return)
            ],
            |_, data: &Data| data.flight
        ),
        EditBox::new(Guard::new(false)),
        EditBox::new(Guard::new(true)),
        Text::new(|_, data: &Data| format!("{}", data.error)),
        Button::new_msg(label_any("Book"), ActionBook).on_update(
            |cx, button, data: &Data| cx.set_disabled(button.id(), !data.error.is_none())
        ),
    ];

    let ui = Adapt::new(ui, data)
        .on_message(|_, data, flight| {
            data.flight = flight;
            data.update_error();
        })
        .on_message(|_, data, parse: ActionDate| {
            if parse.is_return_field {
                data.ret = parse.result;
            } else {
                data.out = parse.result;
            }

            data.update_error();
        })
        .on_messages(|cx, _, data| {
            if cx.try_pop::<ActionBook>().is_some() {
                let msg = if !data.error.is_none() {
                    // should be impossible since the button is disabled
                    format!("{}", data.error)
                } else {
                    match data.flight {
                        Flight::OneWay => format!(
                            "You have booked a one-way flight on {}",
                            data.out.unwrap().format("%Y-%m-%d")
                        ),
                        Flight::Return => format!(
                            "You have booked an out-bound flight on {} and a return flight on {}",
                            data.out.unwrap().format("%Y-%m-%d"),
                            data.ret.unwrap().format("%Y-%m-%d"),
                        ),
                    }
                };
                cx.add_window::<()>(MessageBox::new(msg).into_window("Booker result"));
            }
            false
        });

    Window::new(ui, "Flight Booker")
}

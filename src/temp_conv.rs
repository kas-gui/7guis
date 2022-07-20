// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Temperature converter
//!
//! TODO: force single-line labels

use kas::event::EventMgr;
use kas::prelude::*;
use kas::widgets::EditBox;

#[derive(Clone, Debug)]
enum Message {
    FromCelsius(f64),
    FromFahrenheit(f64),
}

pub fn window() -> Box<dyn Window> {
    Box::new(impl_singleton! {
        #[derive(Debug)]
        #[widget {
            layout = row: [
                self.celsius,
                "Celsius =",
                self.fahrenheit,
                "Fahrenheit",
            ];
        }]
        struct {
            core: widget_core!(),
            #[widget] celsius: impl HasString = EditBox::new("0").on_edit(|text, mgr| {
                if let Ok(c) = text.parse::<f64>() {
                    mgr.push_msg(Message::FromCelsius(c));
                }
            }),
            #[widget] fahrenheit: impl HasString = EditBox::new("32").on_edit(|text, mgr| {
                if let Ok(f) = text.parse::<f64>() {
                    mgr.push_msg(Message::FromFahrenheit(f));
                }
            }),
        }
        impl Widget for Self {
            fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
                if let Some(msg) = mgr.try_pop_msg() {
                    match msg {
                        Message::FromCelsius(c) => {
                            let f = c * (9.0/5.0) + 32.0;
                            *mgr |= self.fahrenheit.set_string(f.to_string());
                        }
                        Message::FromFahrenheit(f) => {
                            let c = (f - 32.0) * (5.0 / 9.0);
                            *mgr |= self.celsius.set_string(c.to_string());
                        }
                    }
                }
            }
        }
        impl Window for Self {
            fn title(&self) -> &str {
                "Temperature Converter"
            }
        }
    })
}

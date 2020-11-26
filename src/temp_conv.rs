// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Temperature converter
//!
//! TODO: force single-line labels

use kas::event::{Manager, VoidMsg, VoidResponse};
use kas::macros::{make_widget, VoidMsg};
use kas::prelude::*;
use kas::widget::{EditBox, Label, Window};

#[derive(Clone, Debug, VoidMsg)]
enum Message {
    FromCelsius(f64),
    FromFahrenheit(f64),
}

pub fn window() -> Box<dyn kas::Window> {
    Box::new(Window::new(
        "Temperature Converter",
        make_widget! {
            #[widget]
            #[layout(row)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(handler=convert)] celsius: impl HasString = EditBox::new("0")
                    .on_edit(|text| text.parse::<f64>().ok().map(|c| Message::FromCelsius(c))),
                #[widget] _ = Label::new("Celsius ="),
                #[widget(handler=convert)] fahrenheit: impl HasString = EditBox::new("32")
                    .on_edit(|text| text.parse::<f64>().ok().map(|c| Message::FromFahrenheit(c))),
                #[widget] _ = Label::new("Fahrenheit"),
            }
            impl {
                fn convert(&mut self, mgr: &mut Manager, msg: Message) -> VoidResponse {
                    match msg {
                        Message::FromCelsius(c) => {
                            let f = c * (9.0/5.0) + 32.0;
                            *mgr += self.fahrenheit.set_string(f.to_string());
                        }
                        Message::FromFahrenheit(f) => {
                            let c = (f - 32.0) * (5.0 / 9.0);
                            *mgr += self.celsius.set_string(c.to_string());
                        }
                    }
                    VoidResponse::None
                }
            }
        },
    ))
}

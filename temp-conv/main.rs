// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Temperature converter
//!
//! TODO: restrict initial size, without making it fixed or too small.
//!
//! TODO: update on char entry
#![feature(proc_macro_hygiene)]

use kas::class::HasText;
use kas::event::{Manager, VoidMsg, VoidResponse};
use kas::macros::{make_widget, VoidMsg};
use kas::widget::{EditBox, Label, Window};
use kas_wgpu::{kas, theme};

#[derive(Clone, Debug, VoidMsg)]
enum Message {
    Invalid,
    FromCelsius(f64),
    FromFahrenheit(f64),
}

fn main() -> Result<(), kas_wgpu::Error> {
    let window = Window::new(
        "temp-conv",
        make_widget! {
            #[widget]
            #[layout(horizontal)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(handler=convert)] celsius: impl HasText = EditBox::new("0")
                    .on_activate(|entry| entry.parse::<f64>()
                        .map(|c| Message::FromCelsius(c))
                        .unwrap_or(Message::Invalid)),
                #[widget] _ = Label::from("Celsius ="),
                #[widget(handler=convert)] fahrenheit: impl HasText = EditBox::new("32")
                    .on_activate(|entry| entry.parse::<f64>()
                        .map(|f| Message::FromFahrenheit(f))
                        .unwrap_or(Message::Invalid)),
                #[widget] _ = Label::from("Fahrenheit"),
            }
            impl {
                fn convert(&mut self, mgr: &mut Manager, msg: Message) -> VoidResponse {
                    match msg {
                        Message::Invalid => (),
                        Message::FromCelsius(c) => {
                            let f = c * (9.0/5.0) + 32.0;
                            self.fahrenheit.set_text(mgr, f.to_string());
                        }
                        Message::FromFahrenheit(f) => {
                            let c = (f - 32.0) * (5.0 / 9.0);
                            self.celsius.set_text(mgr, c.to_string());
                        }
                    }
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

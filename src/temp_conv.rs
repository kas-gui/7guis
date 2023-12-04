// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Temperature converter

use kas::prelude::*;
use kas::widgets::{Adapt, EditBox};

#[derive(Clone, Debug)]
enum Message {
    FromCelsius(f64),
    FromFahrenheit(f64),
}

impl_scope! {
    #[impl_default]
    #[derive(Debug)]
    struct Temperature {
        celsius: f64 = 0.0,
        fahrenheit: f64 = 32.0,
    }

    impl Self {
        fn handle(&mut self, msg: Message) {
            match msg {
                Message::FromCelsius(c) => {
                    self.celsius = c;
                    self.fahrenheit = c * (9.0/5.0) + 32.0;
                }
                Message::FromFahrenheit(f) => {
                    self.celsius = (f - 32.0) * (5.0 / 9.0);
                    self.fahrenheit = f;
                }
            }
        }
    }
}

pub fn window() -> Window<()> {
    let ui = kas::row![
        EditBox::parser(|temp: &Temperature| temp.celsius, Message::FromCelsius),
        "Celsius =",
        EditBox::parser(
            |temp: &Temperature| temp.fahrenheit,
            Message::FromFahrenheit
        ),
        "Fahrenheit",
    ];
    let ui = Adapt::new(ui, Temperature::default()).on_message(|_, temp, msg| temp.handle(msg));
    Window::new(ui, "Temperature Converter")
}

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! 7-GUIs launcher
#![feature(proc_macro_hygiene)]

mod counter;
mod temp_conv;
mod flight_booker;

use kas::prelude::*;
use kas::widget::{MessageBox, TextButton, Window};

#[derive(Clone, Debug, VoidMsg)]
enum X {
    Counter,
    Temp,
    Flight,
    Timer,
    Crud,
    Circle,
    Cells,
}

fn main() -> Result<(), kas_wgpu::Error> {
    let window = Window::new(
        "7GUIs Launcher",
        make_widget! {
            #[widget]
            #[layout(column)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(handler = launch)] _ = TextButton::new("Counter", X::Counter),
                #[widget(handler = launch)] _ = TextButton::new("Temperature Converter", X::Temp),
                #[widget(handler = launch)] _ = TextButton::new("Flight Booker", X::Flight),
                #[widget(handler = launch)] _ = TextButton::new("Timer", X::Timer),
                #[widget(handler = launch)] _ = TextButton::new("CRUD (Create, Read, Update and Delete)", X::Crud),
                #[widget(handler = launch)] _ = TextButton::new("Circle Drawer", X::Circle),
                #[widget(handler = launch)] _ = TextButton::new("Cells", X::Cells),
            }
            impl {
                fn launch(&mut self, mgr: &mut Manager, x: X) -> Response<VoidMsg> {
                    mgr.add_window(match x {
                        X::Counter => counter::window(),
                        X::Temp => temp_conv::window(),
                        X::Flight => flight_booker::window(),
                        _ => Box::new(MessageBox::new("TODO", "Not implemented yet!")),
                    });
                    Response::None
                }
            }
        },
    );

    let theme = kas_wgpu::theme::ShadedTheme::new();
    let mut toolkit = kas_wgpu::Toolkit::new(theme)?;
    toolkit.add(window)?;
    toolkit.run()
}

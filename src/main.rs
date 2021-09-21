// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! 7-GUIs launcher

mod cells;
mod counter;
mod crud;
mod flight_booker;
mod temp_conv;
mod timer;

use kas::prelude::*;
use kas::widgets::{MessageBox, TextButton, Window};

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

fn main() -> Result<(), kas::shell::Error> {
    env_logger::init();

    let window = Window::new(
        "7GUIs Launcher",
        make_widget! {
            #[widget]
            #[layout(column)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(use_msg = launch)] _ = TextButton::new_msg("&Counter", X::Counter),
                #[widget(use_msg = launch)] _ = TextButton::new_msg("Tem&perature Converter", X::Temp),
                #[widget(use_msg = launch)] _ = TextButton::new_msg("&Flight &Booker", X::Flight),
                #[widget(use_msg = launch)] _ = TextButton::new_msg("&Timer", X::Timer),
                #[widget(use_msg = launch)] _ = TextButton::new_msg("CRUD (Create, Read, &Update and &Delete)", X::Crud),
                #[widget(use_msg = launch)] _ = TextButton::new_msg("Ci&rcle Drawer", X::Circle),
                #[widget(use_msg = launch)] _ = TextButton::new_msg("Ce&lls", X::Cells),
            }
            impl {
                fn launch(&mut self, mgr: &mut Manager, x: X) {
                    mgr.add_window(match x {
                        X::Counter => counter::window(),
                        X::Temp => temp_conv::window(),
                        X::Flight => flight_booker::window(),
                        X::Timer => timer::window(),
                        X::Crud => crud::window(),
                        X::Cells => cells::window(),
                        _ => Box::new(MessageBox::new("TODO", "Not implemented yet!")),
                    });
                }
            }
        },
    );

    let theme = kas::theme::ShadedTheme::new();
    let mut toolkit = kas::shell::Toolkit::new(theme)?;
    toolkit.add(window)?;
    toolkit.run()
}

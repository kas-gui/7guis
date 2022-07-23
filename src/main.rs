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
use kas::widgets::dialog::MessageBox;
use kas::widgets::TextButton;

#[derive(Clone, Debug)]
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

    let window = impl_singleton! {
        #[derive(Debug)]
        #[widget {
            layout = column: [
                TextButton::new_msg("&Counter", X::Counter),
                TextButton::new_msg("Tem&perature Converter", X::Temp),
                TextButton::new_msg("&Flight &Booker", X::Flight),
                TextButton::new_msg("&Timer", X::Timer),
                TextButton::new_msg("CRUD (Create, Read, &Update and &Delete)", X::Crud),
                TextButton::new_msg("Ci&rcle Drawer", X::Circle),
                TextButton::new_msg("Ce&lls", X::Cells),
            ];
        }]
        struct {
            core: widget_core!(),
        }
        impl Widget for Self {
            fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
                if let Some(x) = mgr.try_pop_msg() {
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
        }
        impl Window for Self {
            fn title(&self) -> &str {
                "7GUIs Launcher"
            }
        }
    };

    let theme = kas::theme::ShadedTheme::new();
    let mut toolkit = kas::shell::Toolkit::new(theme)?;
    toolkit.add(window)?;
    toolkit.run()
}

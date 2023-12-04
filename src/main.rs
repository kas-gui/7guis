// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! 7-GUIs launcher

// mod cells;
mod counter;
mod crud;
mod flight_booker;
mod temp_conv;
mod timer;

use kas::prelude::*;
use kas::widgets::dialog::MessageBox;
use kas::widgets::Button;

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

    let ui = impl_anon! {
        #[widget {
            layout = column! [
                Button::label_msg("&Counter", X::Counter),
                Button::label_msg("Tem&perature Converter", X::Temp),
                Button::label_msg("&Flight &Booker", X::Flight),
                Button::label_msg("&Timer", X::Timer),
                Button::label_msg("CRUD (Create, Read, &Update and &Delete)", X::Crud),
                Button::label_msg("Ci&rcle Drawer", X::Circle),
                Button::label_msg("Ce&lls", X::Cells),
            ];
        }]
        struct {
            core: widget_core!(),
        }
        impl Events for Self {
            type Data = ();

            fn handle_messages(&mut self, cx: &mut EventCx, _: &Self::Data) {
                if let Some(x) = cx.try_pop() {
                    cx.add_window(match x {
                        X::Counter => counter::window(),
                        X::Temp => temp_conv::window(),
                        X::Flight => flight_booker::window(),
                        X::Timer => timer::window(),
                        X::Crud => crud::window(),
                        // X::Cells => cells::window(),
                        _ => MessageBox::new("Not implemented yet!").into_window("TODO"),
                    });
                }
            }
        }
    };
    let window = Window::new(ui, "7GUIs Launcher");

    let theme = kas::theme::FlatTheme::new();
    let mut shell = kas::shell::Default::with_theme(theme).build(())?;
    shell.add(window);
    shell.run()
}

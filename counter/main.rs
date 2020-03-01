// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Counter
//!
//! TODO: restrict initial size, without making it fixed or too small.
#![feature(proc_macro_hygiene)]

use kas_wgpu::{kas, theme};
use kas::class::HasText;
use kas::event::{Manager, VoidMsg, VoidResponse};
use kas::macros::make_widget;
use kas::widget::{Label, TextButton, Window};

fn main() -> Result<(), kas_wgpu::Error> {
    let window = Window::new(
        "Counter",
        make_widget! {
            #[widget]
            #[layout(horizontal)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(halign = centre)] display: Label = Label::from("0"),
                #[widget(handler = count)] _ = TextButton::new("Count", ()),
                counter: usize = 0,
            }
            impl {
                fn count(&mut self, mgr: &mut Manager, _msg: ()) -> VoidResponse {
                    self.counter = self.counter.saturating_add(1);
                    self.display.set_text(mgr, self.counter.to_string());
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

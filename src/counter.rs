// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Counter

use kas::event::{Manager, VoidMsg, VoidResponse};
use kas::macros::make_widget;
use kas::prelude::*;
use kas::widget::{Label, TextButton, Window};

pub fn window() -> Box<dyn kas::Window> {
    Box::new(Window::new(
        "Counter",
        make_widget! {
            #[widget]
            #[layout(row)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(halign = centre)] display: Label<String> =
                    Label::new("0".to_string()).with_reserve("0000".to_string()),
                #[widget(handler = count)] _ = TextButton::new("Count", ()),
                counter: usize = 0,
            }
            impl {
                fn count(&mut self, mgr: &mut Manager, _msg: ()) -> VoidResponse {
                    self.counter = self.counter.saturating_add(1);
                    *mgr += self.display.set_string(self.counter.to_string());
                    VoidResponse::None
                }
            }
        },
    ))
}

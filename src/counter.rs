// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Counter

use kas::event::EventMgr;
use kas::macros::impl_singleton;
use kas::prelude::*;
use kas::widgets::{EditBox, TextButton};

pub fn window() -> Box<dyn Window> {
    Box::new(impl_singleton! {
        #[derive(Debug)]
        #[widget {
            layout = row: [
                align(right): self.display,
                TextButton::new_msg("Count", ()),
            ];
        }]
        struct {
            core: widget_core!(),
            #[widget] display: impl HasString = EditBox::new("0".to_string())
                .with_width_em(3.0, 3.0)
                .with_editable(false),
            counter: usize = 0,
        }
        impl Widget for Self {
            fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
                if let Some(()) = mgr.try_pop_msg() {
                    self.counter = self.counter.saturating_add(1);
                    *mgr |= self.display.set_string(self.counter.to_string());
                }
            }
        }
        impl Window for Self {
            fn title(&self) -> &str {
                "Counter"
            }
        }
    })
}

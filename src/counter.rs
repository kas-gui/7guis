// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Counter

use kas::prelude::*;
use kas::widgets::{row, Button, EditBox};

#[derive(Clone, Debug)]
struct Incr;

pub fn window() -> Window<()> {
    let ui = row![
        EditBox::string(|count| format!("{count}"))
            .with_width_em(3.0, 3.0)
            .align(AlignHints::RIGHT),
        Button::label_msg("&Count", Incr).map_any(),
    ];
    let ui = ui.with_state(0).on_message(|_, count, Incr| *count += 1);
    Window::new(ui, "Counter")
}

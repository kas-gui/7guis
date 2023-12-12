// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Timer

use kas::prelude::*;
use kas::widgets::{label_any, Adapt, Button, ProgressBar, Slider, Text};
use std::time::{Duration, Instant};

const DUR_MIN: Duration = Duration::from_secs(0);
const DUR_MAX: Duration = Duration::from_secs(30);
const DUR_STEP: Duration = Duration::from_millis(100);
const TIMER_ID: u64 = 0;
const TIMER_SLEEP: Duration = DUR_STEP;

#[derive(Clone, Debug)]
struct ActionReset;

pub fn window() -> Window<()> {
    #[derive(Debug)]
    struct Data {
        duration: Duration,
        elapsed: Duration,
        start: Option<Instant>,
    }

    let ui = kas::grid! {
        (0, 0) => "Elapsed time:",
        (1, 0) => ProgressBar::right(|_, data: &Data| data.elapsed.as_secs_f32() / data.duration.as_secs_f32()),
        (1, 1) => Text::new(|_, data: &Data| {
            format!("{}.{}s", data.elapsed.as_secs(), data.elapsed.subsec_millis() / 100)
        }),
        (0, 2) => "Duration:",
        (1, 2) => Slider::right(DUR_MIN..=DUR_MAX, |_, data: &Data| data.duration)
                    .with_step(DUR_STEP)
                    .with_msg(|value| value),
        (0..2, 3) => Button::new_msg(label_any("Reset"), ActionReset),
    };

    let data = Data {
        duration: Duration::from_secs(10),
        elapsed: Duration::default(),
        start: None,
    };

    let ui = Adapt::new(ui, data)
        .on_configure(|cx, data| {
            data.start = Some(Instant::now());
            cx.request_timer(TIMER_ID, TIMER_SLEEP);
        })
        .on_timer(TIMER_ID, |cx, data, _| {
            if let Some(start) = data.start {
                data.elapsed = data.duration.min(Instant::now() - start);
                if data.elapsed < data.duration {
                    cx.request_timer(TIMER_ID, TIMER_SLEEP);
                } else {
                    data.start = None;
                }
                true
            } else {
                false
            }
        })
        .on_message(|cx, data, dur| {
            data.duration = dur;
            if let Some(start) = data.start {
                data.elapsed = data.duration.min(Instant::now() - start);
                if data.elapsed >= data.duration {
                    data.start = None;
                }
            } else if data.elapsed < data.duration {
                data.start = Some(Instant::now() - data.elapsed);
                cx.request_timer(TIMER_ID, Duration::ZERO);
            }
        })
        .on_message(|cx, data, ActionReset| {
            data.start = Some(Instant::now());
            cx.request_timer(TIMER_ID, TIMER_SLEEP);
        });

    Window::new(ui, "Timer")
}

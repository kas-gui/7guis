// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Timer

use kas::event::TimerHandle;
use kas::prelude::*;
use kas::widgets::{grid, Adapt, Button, ProgressBar, Slider, Text};
use std::time::{Duration, Instant};

const DUR_MIN: u64 = 0;
const DUR_MAX: u64 = 30_000;
const DUR_STEP: u64 = 100;
const TIMER_ID: TimerHandle = TimerHandle::new(0, true);
const TIMER_SLEEP: Duration = Duration::from_millis(DUR_STEP);

#[derive(Clone, Debug)]
struct ActionReset;

pub fn window() -> Window<()> {
    #[derive(Debug)]
    struct Data {
        millis: u64,
        elapsed: Duration,
        start: Option<Instant>,
    }

    let ui = grid! {
        (0, 0) => "Elapsed time:",
        (1, 0) => ProgressBar::right(|_, data: &Data| data.elapsed.as_secs_f32() * 1000.0 / f32::conv(data.millis)),
        (1, 1) => Text::new_gen(|_, data: &Data| {
            format!("{}.{}s", data.elapsed.as_secs(), data.elapsed.subsec_millis() / 100)
        }),
        (0, 2) => "Duration:",
        (1, 2) => Slider::right(DUR_MIN..=DUR_MAX, |_, data: &Data| data.millis)
                    .with_step(DUR_STEP)
                    .with_msg(|value| value),
        (0..=1, 3) => Button::label_msg("&Reset", ActionReset).map_any(),
    };

    let data = Data {
        millis: 10_000,
        elapsed: Duration::default(),
        start: None,
    };

    let ui = Adapt::new(ui, data)
        .on_configure(|cx, _, data| {
            data.start = Some(Instant::now());
            cx.request_timer(TIMER_ID, TIMER_SLEEP);
        })
        .on_timer(TIMER_ID, |cx, _, data, _| {
            if let Some(start) = data.start {
                let duration = Duration::from_millis(data.millis);
                data.elapsed = duration.min(Instant::now() - start);
                if data.elapsed < duration {
                    cx.request_timer(TIMER_ID, TIMER_SLEEP);
                } else {
                    data.start = None;
                }
            }
        })
        .on_message(|cx, data, millis| {
            data.millis = millis;
            let duration = Duration::from_millis(data.millis);
            if let Some(start) = data.start {
                data.elapsed = duration.min(Instant::now() - start);
                if data.elapsed >= duration {
                    data.start = None;
                }
            } else if data.elapsed < duration {
                data.start = Some(Instant::now() - data.elapsed);
                cx.request_timer(TIMER_ID, Duration::ZERO);
            }
        })
        .on_message(|cx, data, ActionReset| {
            data.start = Some(Instant::now());
            cx.request_timer(TIMER_ID, TIMER_SLEEP);
        });

    Window::new(ui, "Timer").escapable()
}

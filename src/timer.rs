// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Timer

use kas::dir::Right;
use kas::prelude::*;
use kas::widgets::{Label, ProgressBar, Slider, TextButton};
use std::time::{Duration, Instant};

const DUR_MIN: Duration = Duration::from_secs(0);
const DUR_MAX: Duration = Duration::from_secs(30);
const DUR_STEP: Duration = Duration::from_millis(100);
const TIMER_ID: u64 = 0;

#[derive(Clone, Debug)]
struct ActionReset;

pub fn window() -> Box<dyn Window> {
    Box::new(singleton! {
        #[derive(Debug)]
        #[widget {
            layout = grid: {
                0, 0: "Elapsed time:";
                1, 0: self.progress;
                1, 1: self.elapsed;
                0, 2: "Duration:";
                1, 2: self.slider;
                0..2, 3: TextButton::new_msg("Reset", ActionReset);
            };
        }]
        struct {
            core: widget_core!(),
            #[widget] progress: ProgressBar<Right> = ProgressBar::new(),
            #[widget] elapsed: Label<String> = Label::new("0.0s".to_string()),
            #[widget] slider =
                Slider::new_with_direction(DUR_MIN..=DUR_MAX, DUR_STEP, Right)
                    .with_value(Duration::from_secs(10))
                    .on_move(|mgr, value| mgr.push_msg(value)),
            dur: Duration = Duration::from_secs(10),
            saved: Duration = Duration::default(),
            start: Option<Instant> = None,
        }
        impl Self {
            fn update(&mut self, mgr: &mut EventMgr, elapsed: Duration) {
                let frac = elapsed.as_secs_f32() / self.dur.as_secs_f32();
                *mgr |= self.progress.set_value(frac);
                *mgr |= self.elapsed.set_string(format!(
                    "{}.{}s",
                    elapsed.as_secs(),
                    elapsed.subsec_millis() / 100
                ));
            }
        }
        impl Widget for Self {
            fn configure(&mut self, mgr: &mut ConfigMgr) {
                self.start = Some(Instant::now());
                mgr.request_update(self.id(), TIMER_ID, DUR_STEP, true);
            }
            fn handle_event(&mut self, mgr: &mut EventMgr, event: Event) -> Response {
                match event {
                    Event::TimerUpdate(TIMER_ID) => {
                        if let Some(start) = self.start {
                            let mut elapsed = self.saved + (Instant::now() - start);
                            if elapsed < self.dur {
                                mgr.request_update(self.id(), TIMER_ID, DUR_STEP, true);
                            } else {
                                elapsed = self.dur;
                                self.saved = elapsed;
                                self.start = None;
                            }
                            self.update(mgr, elapsed);
                        }
                        Response::Used
                    }
                    _ => Response::Unused,
                }
            }
            fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
                if let Some(dur) = mgr.try_pop_msg() {
                    self.dur = dur;
                    let mut elapsed = self.saved;
                    if let Some(start) = self.start {
                        elapsed += Instant::now() - start;
                        if elapsed >= self.dur {
                            self.saved = elapsed;
                            self.start = None;
                        }
                    } else if self.saved < self.dur {
                        self.start = Some(Instant::now());
                        mgr.request_update(self.id(), TIMER_ID, Duration::ZERO, true);
                    }
                    self.update(mgr, elapsed);
                } else if let Some(ActionReset) = mgr.try_pop_msg() {
                    self.saved = Duration::default();
                    self.start = Some(Instant::now());
                    mgr.request_update(self.id(), TIMER_ID, DUR_STEP, true);
                    *mgr |= self.progress.set_value(0.0);
                    *mgr |= self.elapsed.set_string("0.0s".to_string());
                }
            }
        }
        impl Window for Self {
            fn title(&self) -> &str {
                "Timer"
            }
        }
    })
}

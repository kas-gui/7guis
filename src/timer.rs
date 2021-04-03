// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Timer

use kas::dir::Right;
use kas::event::VoidResponse;
use kas::prelude::*;
use kas::widget::{Label, ProgressBar, Slider, TextButton, Window};
use std::time::{Duration, Instant};

const DUR_MIN: Duration = Duration::from_secs(0);
const DUR_MAX: Duration = Duration::from_secs(30);
const DUR_STEP: Duration = Duration::from_millis(100);
const TIMER_ID: u64 = 0;

pub fn window() -> Box<dyn kas::Window> {
    Box::new(Window::new(
        "Timer",
        make_widget! {
            #[widget(config = noauto)]
            #[layout(column)]
            #[handler(handle = noauto)]
            struct {
                #[widget] progress: ProgressBar<Right> = ProgressBar::new(),
                #[widget] elapsed: Label<String> = Label::new("0.0s".to_string()),
                #[widget(handler=slider)] _ =
                    Slider::new_with_direction(DUR_MIN, DUR_MAX, DUR_STEP, Right)
                        .with_value(Duration::from_secs(10))
                        .with_reserve(|sh, axis| {
                            if axis.is_horizontal() {
                                SizeRules::fixed(sh.pixels_from_em(6.0).cast_nearest(), (0, 0))
                            } else {
                                SizeRules::EMPTY
                            }
                        })
                        .with_label(kas::dir::Left, "Duration:"),
                #[widget(handler=reset)] _ = TextButton::new_msg("Reset", ()),
                dur: Duration = Duration::from_secs(10),
                saved: Duration = Duration::default(),
                start: Option<Instant> = None,
            }
            impl WidgetConfig {
                fn configure(&mut self, mgr: &mut Manager) {
                    self.start = Some(Instant::now());
                    mgr.update_on_timer(DUR_STEP, self.id(), TIMER_ID);
                }
            }
            impl Handler {
                type Msg = VoidMsg;

                fn handle(&mut self, mgr: &mut Manager, event: Event) -> VoidResponse {
                    match event {
                        Event::TimerUpdate(TIMER_ID) => {
                            if let Some(start) = self.start {
                                let mut dur = self.saved + (Instant::now() - start);
                                if dur < self.dur {
                                    mgr.update_on_timer(DUR_STEP, self.id(), TIMER_ID);
                                } else {
                                    dur = self.dur;
                                    self.saved = dur;
                                    self.start = None;
                                }
                                let frac = dur.as_secs_f32() / self.dur.as_secs_f32();
                                *mgr |= self.progress.set_value(frac);
                                *mgr |= self.elapsed.set_string(format!(
                                    "{}.{}s",
                                    dur.as_secs(),
                                    dur.subsec_millis() / 100
                                ));
                            }
                            Response::None
                        }
                        _ => Response::Unhandled,
                    }
                }
            }
            impl {
                fn slider(&mut self, mgr: &mut Manager, dur: Duration) -> VoidResponse {
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
                        mgr.update_on_timer(DUR_STEP, self.id(), TIMER_ID);
                    }
                    let frac = elapsed.as_secs_f32() / self.dur.as_secs_f32();
                    *mgr |= self.progress.set_value(frac);
                    Response::None
                }
                fn reset(&mut self, mgr: &mut Manager, _: ()) -> VoidResponse {
                    self.saved = Duration::default();
                    self.start = Some(Instant::now());
                    mgr.update_on_timer(DUR_STEP, self.id(), TIMER_ID);
                    *mgr |= self.progress.set_value(0.0);
                    *mgr |= self.elapsed.set_string("0.0s".to_string());
                    Response::None
                }
            }
        },
    ))
}

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Timer

use kas::event::VoidResponse;
use kas::prelude::*;
use kas::widget::{Label, Slider, TextButton, Window};
use std::time::{Duration, Instant};

pub fn window() -> Box<dyn kas::Window> {
    Box::new(Window::new(
        "Timer",
        make_widget! {
            #[widget(config = noauto)]
            #[layout(column)]
            #[handler(handle = noauto)]
            struct {
                #[widget] _ = Label::new("Elapsed time: GAUGE"), // TODO: progress bar
                #[widget] elapsed: Label<String> = Label::new("0.0s".to_string()),
                #[widget(handler=slider)] _ = make_widget! {
                    // TODO: this layout widget is used only to add a label.
                    // Allow all controls to have labels without this?
                    #[layout(row)]
                    #[handler(msg = u64)]
                    struct {
                        #[widget] _ = Label::new("Duration:"),
                        #[widget] _ = Slider::new_with_direction(0u32, 30_000, 100, kas::Right)
                            .with_value(10_000),
                    }
                },
                #[widget(handler=reset)] _ = TextButton::new("Reset", ()),
                dur: Duration = Duration::from_secs(10),
                saved: Duration = Duration::default(),
                start: Option<Instant> = None,
            }
            impl WidgetConfig {
                fn configure(&mut self, mgr: &mut Manager) {
                    self.start = Some(Instant::now());
                    mgr.update_on_timer(Duration::from_millis(100), self.id());
                }
            }
            impl Handler {
                type Msg = VoidMsg;

                fn handle(&mut self, mgr: &mut Manager, event: Event) -> VoidResponse {
                    match event {
                        Event::TimerUpdate => {
                            if let Some(start) = self.start {
                                let mut dur = self.saved + (Instant::now() - start);
                                if dur < self.dur {
                                    mgr.update_on_timer(Duration::from_millis(100), self.id());
                                } else {
                                    dur = self.dur;
                                    self.saved = dur;
                                    self.start = None;
                                }
                                *mgr += self.elapsed.set_string(format!(
                                    "{}.{}s",
                                    dur.as_secs(),
                                    dur.subsec_millis() / 100
                                ));
                            }
                            Response::None
                        }
                        event => Response::Unhandled(event),
                    }
                }
            }
            impl {
                fn slider(&mut self, mgr: &mut Manager, millis: u64) -> VoidResponse {
                    self.dur = Duration::from_millis(millis);
                    if let Some(start) = self.start {
                        let elapsed = (Instant::now() - start) + self.saved;
                        if elapsed >= self.dur {
                            self.saved = elapsed;
                            self.start = None;
                        }
                    } else if self.saved < self.dur {
                        self.start = Some(Instant::now());
                        mgr.update_on_timer(Duration::from_millis(100), self.id());
                    }
                    Response::None
                }
                fn reset(&mut self, mgr: &mut Manager, _: ()) -> VoidResponse {
                    self.saved = Duration::default();
                    self.start = Some(Instant::now());
                    mgr.update_on_timer(Duration::from_millis(100), self.id());
                    *mgr += self.elapsed.set_string("0.0s".to_string());
                    Response::None
                }
            }
        },
    ))
}

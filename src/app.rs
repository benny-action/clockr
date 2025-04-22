use egui::{FontId, RichText};
use rodio::{Decoder, OutputStream, Sink};

use std::{
    io::Cursor,
    thread,
    time::{Duration, Instant},
};

pub struct ClockrApp {
    //timer fields
    timer_start_time: Instant,
    timer_duration: Duration,
    timer_active: bool,
    timer_just_finished: bool,
    completed_timer_count: u32,
    work_flag: bool,
    //--
    default_timer_work: u64,
    default_timer_break: u64,
    default_timer_long: u64,
    //--
    user_work_length: u64,
    user_break_length: u64,
    user_long_length: u64,
    //#[serde(skip)] // This how you opt-out of serialization of a field
}

impl ClockrApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let default_work_length = 25;
        let default_break_length = 5;
        let default_long_length = 15;
        let counter = 0;

        Self {
            timer_start_time: Instant::now(),
            timer_duration: Duration::from_secs(60),
            timer_active: false,
            timer_just_finished: false,
            completed_timer_count: counter,
            work_flag: false,
            //--
            default_timer_work: default_work_length,
            default_timer_break: default_break_length,
            default_timer_long: default_long_length,
            //--
            user_work_length: default_work_length,
            user_break_length: default_break_length,
            user_long_length: default_long_length,
        }
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        //
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
    }

    pub fn start_timer(&mut self, seconds: u64) {
        self.timer_start_time = Instant::now();
        self.timer_duration = Duration::from_secs(seconds);
        self.timer_active = true;
    }

    pub fn reset_to_default(&mut self) {
        self.user_work_length = self.default_timer_work;
        self.user_break_length = self.default_timer_break;
        self.user_long_length = self.default_timer_long;
    }

    pub fn remaining_time(&self) -> Duration {
        if !self.timer_active {
            return self.timer_duration;
        }

        let elapsed = self.timer_start_time.elapsed();
        if elapsed >= self.timer_duration {
            Duration::ZERO
        } else {
            self.timer_duration - elapsed
        }
    }

    pub fn format_time(&self) -> String {
        let remaining = self.remaining_time();
        let seconds = remaining.as_secs() % 60;
        let minutes = (remaining.as_secs() / 60) % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn is_timer_finished(&self) -> bool {
        self.timer_active && self.remaining_time() == Duration::ZERO
    }

    //notification sounds stuff
    pub fn play_notification_sound(&self) {
        let beep_sound = include_bytes!("../assets/beep.mp3");

        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let cursor = Cursor::new(beep_sound);

                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    }
}

impl eframe::App for ClockrApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // timer update requester

        let finished_now =
            self.is_timer_finished() && self.timer_active && !self.timer_just_finished;

        if finished_now {
            self.timer_just_finished = true;
            self.play_notification_sound();
        }

        if finished_now && self.work_flag {
            self.completed_timer_count += 1;
        }

        //reset just finished flag when restarting timer.
        if self.timer_active && !self.is_timer_finished() {
            self.timer_just_finished = false;
        }

        if self.timer_active {
            ctx.request_repaint();
        }

        //keyboard input stuff.

        if ctx.input(|i| i.key_pressed(egui::Key::W)) {
            if self.timer_active {
                self.timer_active = false;
            } else {
                self.start_timer(60 * self.user_work_length);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::B)) {
            if self.timer_active {
                self.timer_active = false;
            } else {
                self.start_timer(60 * self.user_break_length);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::L)) {
            if self.timer_active {
                self.timer_active = false;
            } else {
                self.start_timer(60 * self.user_long_length);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Options", |ui| {
                        ui.label("Work Duration:");
                        ui.add(
                            egui::Slider::new(&mut self.user_work_length, 1..=50).text("minutes"),
                        );
                        ui.label("Break Duration:");
                        ui.add(
                            egui::Slider::new(&mut self.user_break_length, 1..=50).text("minutes"),
                        );
                        ui.label("Long Duration:");
                        ui.add(
                            egui::Slider::new(&mut self.user_long_length, 1..=50).text("minutes"),
                        );
                        if ui.button("Reset to defaults").clicked() {
                            self.reset_to_default();
                        }
                    });
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading(format!("Pomodoros: {}", self.completed_timer_count))
                    .on_hover_text_at_pointer("Pomos = Work timed, remember to take a break");
                ui.separator();

                if self.timer_active {
                    ui.label(RichText::new(self.format_time()).font(FontId::proportional(80.0)));

                    if self.is_timer_finished() {
                        egui::Frame::none()
                            .fill(egui::Color32::ORANGE)
                            .show(ui, |ui| {
                                ui.heading("Split Finished");
                            });
                        // TODO: Add a pomo counter, plus a logo.
                    }
                } else {
                    ui.label(RichText::new("Clockr").font(FontId::proportional(80.0)))
                        .on_hover_text_at_pointer(
                            "W - Start Work \nB - Start Break \nL - Start Long Break",
                        );
                }

                ui.separator();

                if ui.button("Start Work Timer").clicked() {
                    self.start_timer(60 * self.user_work_length);
                    self.work_flag = true;
                }
                if ui.button("Start Break Timer").clicked() {
                    self.start_timer(60 * self.user_break_length);
                    self.work_flag = false;
                }
                if ui.button("Start Long Timer").clicked() {
                    self.start_timer(60 * self.user_long_length);
                    self.work_flag = false;
                }
            });
        });
    }
}

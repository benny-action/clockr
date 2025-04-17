use egui::{FontId, RichText};

use eframe::{App, Frame};
use egui::{CentralPanel, Context};
use std::time::{Duration, Instant};

pub struct ClockrApp {
    //timer fields
    timer_start_time: Instant,
    timer_duration: Duration,
    timer_active: bool,
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

        Self {
            timer_start_time: Instant::now(),
            timer_duration: Duration::from_secs(60),
            timer_active: false,
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
}

impl eframe::App for ClockrApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // timer update requester
        if self.timer_active {
            ctx.request_repaint();
        }
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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
                ui.heading("Pomodoro Timer");
                ui.separator();

                if self.timer_active {
                    ui.label(RichText::new(self.format_time()).font(FontId::proportional(80.0)));

                    if self.is_timer_finished() {
                        egui::Frame::none()
                            .fill(egui::Color32::ORANGE)
                            .show(ui, |ui| {
                                ui.heading("Split Finished");
                            });
                        // TODO: change the logic here so that the "split finished" memo doesn't
                        // break the size of the ui, instead fills the big timer spot.
                        // TODO: add a notification sound and a logo
                    }
                } else {
                    ui.label(RichText::new("Clockr").font(FontId::proportional(80.0)));
                }

                ui.separator();

                if ui.button("Start Work Timer").clicked() {
                    self.start_timer(60 * self.user_work_length);
                }
                if ui.button("Start Break Timer").clicked() {
                    self.start_timer(60 * self.user_break_length);
                }
                if ui.button("Start Long Timer").clicked() {
                    self.start_timer(60 * self.user_long_length);
                }
            });
        });
    }
}

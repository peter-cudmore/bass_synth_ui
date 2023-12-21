use std::fmt::{Debug};
use std::ops::RangeInclusive;
use std::sync::mpsc;
use egui::{Ui, Visuals};
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::bindings::{WaveformEnum, WaveformEnum_SAW, WaveformEnum_SQR, WaveformEnum_SIN, Patch};
use crate::server::run_server;

pub struct BassSynthUI {
    sender: Sender<Message>,
    rx: Receiver<Patch>,
    patch: PatchUI,
}


impl BassSynthUI {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        let (patch_tx, patch_rx)  = channel();
        std::thread::spawn(move || { run_server(rx, patch_tx); });

        Self {
            sender: tx,
            rx: patch_rx,
            patch: PatchUI::default(),
        }
    }
}


pub struct OscillatorCfg {
    pub waveform: WaveformEnum,
    pub coarse: i32,
    pub fine: i32,
    pub mix: f32,
}

impl Default for OscillatorCfg {
    fn default() -> Self {
        Self {
            waveform: WaveformEnum_SIN,
            coarse: 0,
            fine: 0,
            mix: 1.0,
        }
    }
}

#[derive(Debug)]
pub enum Message {
    SetWaveform(u32, WaveformEnum),
    SetCoarse(u32, i32),
    SetFine(u32, i32),
    SetOscMix(u32, f32),
}

#[derive(Default)]
pub struct PatchUI {
    pub osc_1: OscillatorCfg,
    pub osc_2: OscillatorCfg,
    pub osc_3: OscillatorCfg,
}


fn draw_oscillator_section(oscillator_cfg: &mut OscillatorCfg, index: u32, sender: &mpsc::Sender<Message>, ui: &mut Ui) {
    let OscillatorCfg {
        waveform,
        coarse,
        fine,
        mix
    } = oscillator_cfg;
    ui.heading(format!("Osc {}", index + 1));
    ui.horizontal(|ui| {
        ui.selectable_value(waveform, WaveformEnum_SIN, "Sin")
            .changed()
            .then(|| { sender.send(Message::SetWaveform(index, WaveformEnum_SIN)).unwrap(); });
        ui.selectable_value(waveform, WaveformEnum_SAW, "Saw")
            .changed()
            .then(|| { sender.send(Message::SetWaveform(index, WaveformEnum_SAW)).unwrap(); });
        ui.selectable_value(waveform, WaveformEnum_SQR, "Sqr")
            .changed()
            .then(|| { sender.send(Message::SetWaveform(index, WaveformEnum_SQR)).unwrap(); });
    });
    ui.end_row();

    egui::Grid::new(format!("Osc {}", index + 1))
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Coarse");
            ui.add(egui::DragValue::new(coarse)
                .speed(1.0)
                .clamp_range(RangeInclusive::new(-24, 24)))
                .drag_released()
                .then(|| { sender.send(Message::SetCoarse(index, coarse.clone())) });
            ui.end_row();


            ui.label("Fine");
            ui.add(egui::DragValue::new(fine).speed(1.0).clamp_range(RangeInclusive::new(-50, 50)))
                .drag_released()
                .then(|| { sender.send(Message::SetFine(index, fine.clone())) });
            ui.end_row();

            ui.label("Mix");
            ui.add(egui::DragValue::new(mix).speed(1.0).clamp_range(RangeInclusive::new(0, 100)))
                .drag_released()
                .then(|| { sender.send(Message::SetOscMix(index, mix.clone() / 100.0 )) });
            ui.end_row();
        });
}

impl eframe::App for BassSynthUI {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        ctx.set_visuals(Visuals::dark());
        {
            if let Ok(msg) = self.rx.try_recv() {
                //self.patch = msg;
            }

            let patch = & mut self.patch;

            let sender = &self.sender;

            egui::CentralPanel::default().show(ctx, |mut ui| {
                egui::Grid::new("OscillatorBank")
                    .min_col_width(70.0)
                    .max_col_width(70.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.group(|ui| {
                                draw_oscillator_section(&mut patch.osc_1, 0, sender, ui);
                            });
                        });
                        ui.vertical_centered(|ui| {
                            ui.group(|ui| {
                                draw_oscillator_section(&mut patch.osc_2, 1, sender, ui);
                            });
                        });
                        ui.vertical_centered(|ui| {
                            ui.group(|ui| {
                                draw_oscillator_section(&mut patch.osc_3, 2, sender, ui);
                            });
                        });
                    });
            },
            );
        }
        /*
                let mut label =String::new();
                let mut value:f32 = 0.0;
                egui::CentralPanel::default().show(ctx, |ui| {
                    // The central panel the region left after adding TopPanel's and SidePanel's
                    ui.heading("eframe template");

                    ui.horizontal(|ui| {
                        ui.label("Write something: ");
                        ui.text_edit_singleline(&mut label);
                    });

                    ui.add(egui::Slider::new(&mut value, 0.0..=10.0).text("value"));
                    if ui.button("Increment").clicked() {
                        value += 1.0;
                    }

                    ui.separator();

                    ui.add(egui::github_link_file!(
                        "https://github.com/emilk/eframe_template/blob/master/",
                        "Source code."
                    ));

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        powered_by_egui_and_eframe(ui);
                        egui::warn_if_debug_build(ui);
                    });

              });

         */
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

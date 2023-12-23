use std::fmt::{Debug, Formatter, Write};
use std::ops::RangeInclusive;
use std::sync::mpsc;
use egui::{Ui, Visuals};
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::bindings::{WaveformEnum, WaveformEnum_SAW, WaveformEnum_SQR, WaveformEnum_SIN, Patch, ParameterType, ParameterValue, ParameterType_Attack, Section, Section_Filter, ParameterType_Decay, ParameterType_Sustain, ParameterType_Release, ParameterType_Cutoff, ParameterType_Resonance, ParameterType_Emphasis, FilterModeEnum, FilterModeEnum_HP, FilterModeEnum_LP, ParameterType_Mode, Section_Amp, ParameterType_Gain, Section_Osc1, Section_Osc2, Section_Osc3, Section_Global, ParameterType_Waveform, ParameterType_Coarse, ParameterType_Fine};
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

#[derive(Default)]
pub struct OscillatorCfg {
    pub waveform: WaveformEnum,
    pub coarse: i8,
    pub fine: i8,
    pub gain: i8,
}

pub struct EnvelopeCfg {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32
}

impl Default for EnvelopeCfg {
    fn default() -> Self {
        EnvelopeCfg{
            attack: 20.0,
            decay: 200.0,
            sustain: 0.5,
            release: 500.0
        }
    }
}


const TimeWindow: RangeInclusive<f32> = 20.0..=2000.0;
const FreqWindow: RangeInclusive<f32> = 20.0..=20000.0;
#[derive(Default)]
pub struct FilterCfg {
    pub filter_type: FilterModeEnum,
    pub envelope: EnvelopeCfg,
    pub cutoff: f32,
    pub resonance: u8,
    pub emphasis: f32
}

type Osc = u8;
#[derive(Debug)]
pub enum Message {
    SetWaveform(Osc, WaveformEnum),
    SetCoarse(Osc, i8),
    SetFine(Osc, i8),
    SetOscGain(Osc, i8),
    SetParameter(Section, ParameterType, ParameterValue)
}

impl Debug for ParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {write!(f, "{:?}", self.freq.to_le_bytes())}
    }
    
}

#[derive(Default)]
pub struct AmplConfig {
    pub envelope: EnvelopeCfg,
    pub gain: i8,
}


#[derive(Default)]
pub struct PatchUI {
    pub osc_1: OscillatorCfg,
    pub osc_2: OscillatorCfg,
    pub osc_3: OscillatorCfg,
    pub filter: FilterCfg,
    pub amp: AmplConfig
}

impl From<Patch> for PatchUI {
    fn from(value: Patch) -> Self {
        PatchUI{
            osc_1: OscillatorCfg{
                waveform: value.Osc1_Waveform,
                coarse: value.Osc1_Coarse,
                fine: value.Osc1_Fine,
                gain: value.Osc1_Gain,
            },
            osc_2: OscillatorCfg{
                waveform: value.Osc2_Waveform,
                coarse: value.Osc2_Coarse,
                fine: value.Osc2_Fine,
                gain: value.Osc2_Gain,
            },
            osc_3: OscillatorCfg{
                waveform: value.Osc3_Waveform,
                coarse: value.Osc3_Coarse,
                fine: value.Osc3_Fine,
                gain: value.Osc3_Gain,
            },
            filter: FilterCfg{
                filter_type: value.Filter_Mode,
                envelope:EnvelopeCfg{
                    attack: value.Filter_Attack,
                    decay: value.Filter_Decay,
                    sustain: value.Filter_Sustain,
                    release: value.Filter_Release
                },
                cutoff: value.Filter_Cutoff,
                resonance: value.Filter_Resonance,
                emphasis: value.Filter_Emphasis
            },
            amp: AmplConfig{
                gain: value.Amp_Gain,
                envelope:EnvelopeCfg{
                    attack: value.Amp_Attack,
                    decay: value.Amp_Decay,
                    sustain: value.Amp_Sustain,
                    release: value.Amp_Release
                }
            }
        }
    }
}

fn draw_oscillator_section(oscillator_cfg: &mut OscillatorCfg, index: Osc, sender: &mpsc::Sender<Message>, ui: &mut Ui) {
    let OscillatorCfg {
        waveform,
        coarse,
        fine,
        gain
    } = oscillator_cfg;
    ui.heading(format!("Osc {}", index + 1));
    let section = match index {
        0 => {Section_Osc1},
        1 => {Section_Osc2},
        2 => {Section_Osc3},
        _ => {Section_Global}
    };
    ui.horizontal(|ui| {
        ui.selectable_value(waveform, WaveformEnum_SIN, "Sin")
            .changed()
            .then(|| { sender.send(
                Message::SetParameter(section, ParameterType_Waveform, ParameterValue{value_WaveformEnum: WaveformEnum_SIN})
            ).unwrap(); });
        ui.selectable_value(waveform, WaveformEnum_SAW, "Saw")
            .changed()
            .then(|| { sender.send(Message::SetParameter(section, ParameterType_Waveform, ParameterValue{value_WaveformEnum: WaveformEnum_SAW})
          ).unwrap(); });
        ui.selectable_value(waveform, WaveformEnum_SQR, "Sqr")
            .changed()
            .then(|| { sender.send(Message::SetParameter(section, ParameterType_Waveform, ParameterValue{value_WaveformEnum: WaveformEnum_SQR})).unwrap(); });
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
                .then(|| { sender.send(Message::SetParameter(section, ParameterType_Coarse, ParameterValue{value_int8_t: coarse.clone()})) });
            ui.end_row();


            ui.label("Fine");
            ui.add(egui::DragValue::new(fine).speed(1.0).clamp_range(RangeInclusive::new(-50, 50)))
                .drag_released()
                .then(|| { sender.send(Message::SetParameter(section, ParameterType_Fine, ParameterValue{value_int8_t:fine.clone()}))});
            ui.end_row();

            ui.vertical(|ui| {
                let slider = egui::Slider::new(gain, i8::MIN..=6)
                    .vertical()
                    .custom_formatter(|n, _| {
                        let n = n as i8;
                        if n == i8::MIN {
                            format!("-inf")
                        } else {
                            format!("{}", n)
                        }
                    });
                ui.add(slider)
                    .drag_released()
                    .then(|| { sender.send(Message::SetParameter(section, ParameterType_Gain, ParameterValue{value_int8_t: gain.clone()}))});
                ui.end_row();
            });
        });
}


fn add_slider<T>(value: &mut T, range: RangeInclusive<T>, name: &str, ui: &mut Ui, sender: &Sender<Message>, on_changed: fn(T) -> Message)
where T: egui::emath::Numeric
{
    ui.vertical( |ui| {
        ui.label(name);
        let slider = egui::Slider::new(value, range)
            .vertical();

        ui.add(slider)
            .drag_released()
            .then(|| {
                let msg = on_changed(value.clone());
                sender.send(msg).unwrap();
            });
    });
}


fn draw_filter_section(filter_cfg: &mut FilterCfg, sender: &Sender<Message>, ui: &mut Ui) {


    let FilterCfg{filter_type,  envelope, cutoff, resonance, emphasis} = filter_cfg;
    let EnvelopeCfg{attack,decay,sustain,release} = envelope;
    ui.group(|ui| {
        egui::Grid::new("Filter header")
            .num_columns(2)
            .show(ui, |ui| {
                ui.heading("Filter");
                ui.add_space(200.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(filter_type, FilterModeEnum_HP, "HP")
                        .changed().then(|| sender.send(
                        Message::SetParameter(Section_Filter, ParameterType_Mode, ParameterValue{value_FilterModeEnum: FilterModeEnum_HP})
                    ));
                    ui.selectable_value(filter_type, FilterModeEnum_LP, "LP")
                        .changed().then(|| sender.send(
                        Message::SetParameter(Section_Filter, ParameterType_Mode, ParameterValue{value_FilterModeEnum: FilterModeEnum_LP})
                    ));
                })
            }
            );

        ui.horizontal(
            |ui| {
                add_slider(cutoff, FreqWindow, "Freq", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Cutoff, ParameterValue { value_float: v })
                });
                add_slider(resonance, 0..=u8::MAX, "Reso", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Resonance, ParameterValue { value_uint8_t: v })
                });
                add_slider(emphasis, 0.0..=1.0, "Emph", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Emphasis, ParameterValue { value_float: v })
                });
                add_slider(attack, TimeWindow, "A", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Attack, ParameterValue { value_float: v })
                });
                add_slider(decay, TimeWindow, "D", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Decay, ParameterValue { value_float: v })
                });
                add_slider(sustain, 0.0..=1.0, "S", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Sustain, ParameterValue { value_float: v })
                });
                add_slider(release, TimeWindow, "R", ui, sender, |v| {
                    Message::SetParameter(Section_Filter, ParameterType_Release, ParameterValue { value_float: v })
                });
            }
        );
    });
}


fn draw_amp_section(ampl_config: &mut AmplConfig, sender: &Sender<Message>, ui: &mut Ui){
    ui.group(|ui| {
        ui.heading("Amp");
        let AmplConfig{gain, envelope} = ampl_config;
        let EnvelopeCfg{attack,decay,sustain,release} = envelope;

        ui.horizontal(
            |ui| {
                add_slider(attack, TimeWindow, "A", ui, sender, |v| {
                    Message::SetParameter(Section_Amp, ParameterType_Attack, ParameterValue { value_float: v })
                });
                add_slider(decay, TimeWindow, "D", ui, sender, |v| {
                    Message::SetParameter(Section_Amp, ParameterType_Decay, ParameterValue { value_float: v })
                });
                add_slider(sustain, 0.0..=1.0, "S", ui, sender, |v| {
                    Message::SetParameter(Section_Amp, ParameterType_Sustain, ParameterValue { value_float: v })
                });
                add_slider(release, TimeWindow, "R", ui, sender, |v| {
                    Message::SetParameter(Section_Amp, ParameterType_Release, ParameterValue { value_float: v })
                });
                ui.vertical(|ui| {
                    ui.label("Gain");
                    let slider = egui::Slider::new(gain, i8::MIN..=6)
                        .vertical()
                        .custom_formatter(|n, _| {
                            let n = n as i8;
                            if n == i8::MIN {
                                format!("-inf")
                            } else {
                                format!("{}", n)
                            }
                        });
                    ui.add(slider)
                        .drag_released()
                        .then(|| { sender.send(Message::SetParameter(Section_Amp, ParameterType_Gain, ParameterValue { value_int8_t: gain.clone() })) });
                }
                );
            });
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
                draw_filter_section(&mut patch.filter, sender, ui);
                ui.end_row();
                draw_amp_section(&mut patch.amp, sender, ui);
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

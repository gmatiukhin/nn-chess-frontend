#![warn(clippy::all, rust_2018_idioms)]

use chess::ChessBoard;
use shakmaty::Color;
use tokio::sync::mpsc;

use anyhow::Result;
use egui::{Align2, Grid};
use requests::RequestLoopComm;
use web_types::{EngineDescription, EngineDirectory, EngineRef, EngineVariant};

mod chess;
mod requests;

pub struct App {
    chessboard: chess::ChessBoard,
    fetch_engine_data: bool,
    engine_data: EngineData,
    sender: mpsc::UnboundedSender<requests::RequestLoopComm>,
    engine_dir_receiver: Option<oneshot::Receiver<Result<EngineDirectory>>>,
    engine_desc_receiver: Option<oneshot::Receiver<Result<EngineDescription>>>,
}

#[derive(Default)]
struct EngineData {
    available_engines: Option<EngineDirectory>,
    selected_engine: Option<EngineRef>,
    desc: Option<EngineDescription>,
    variant: Option<EngineVariant>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let req_comm_loop = requests::run_request_loop(cc.egui_ctx.clone());

        Self {
            chessboard: Default::default(),
            fetch_engine_data: true,
            engine_data: EngineData::default(),
            sender: req_comm_loop,
            engine_desc_receiver: None,
            engine_dir_receiver: None,
        }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::right("engine_info").show(ctx, |ui| {
            ui.heading("Select engine");
            if ui.button("Update info").clicked() {
                self.engine_data.available_engines = None;
                // Build a request to the request loop.
                let (sender, receiver) = oneshot::channel();
                let req = RequestLoopComm::FetchEngines(sender);
                self.sender
                    .send(req)
                    .expect("Error communicating with request loop");

                self.engine_dir_receiver = Some(receiver);
            }

            if let Some(recv) = &self.engine_dir_receiver {
                if let Ok(Ok(engines)) = recv.try_recv() {
                    self.engine_data.available_engines = Some(engines.clone());
                    self.engine_data.selected_engine = Some(engines.engines[0].clone());
                }
            }
            if let Some(data) = self.engine_data.selected_engine.as_mut() {
                egui::ComboBox::from_label("engine_selection")
                    .selected_text(data.name.to_string())
                    .show_ui(ui, |ui| {
                        for engine in self
                            .engine_data
                            .available_engines
                            .as_mut()
                            .unwrap()
                            .engines
                            .iter()
                        {
                            ui.selectable_value(data, engine.clone(), engine.name.clone());
                        }
                    });

                Grid::new("current_engine_info").show(ui, |ui| {
                    let selected_engine = data.clone();
                    ui.label("Name");
                    ui.label(selected_engine.name);
                    ui.end_row();
                    ui.label("Id");
                    ui.label(selected_engine.engine_id);
                    ui.end_row();
                    ui.label("URL");
                    ui.hyperlink(selected_engine.entrypoint_url);
                    ui.end_row();
                });
            }

            if let Some(selected_engine) = &self.engine_data.selected_engine {
                ui.heading("Select variant");
                if ui.button("Update info").clicked() {
                    let (sender, receiver) = oneshot::channel();
                    let req =
                        RequestLoopComm::FetchEngineDescription(selected_engine.clone(), sender);
                    self.sender
                        .send(req)
                        .expect("Error communicating with request loop");
                    self.engine_desc_receiver = Some(receiver);
                }
                if let Some(recv) = &self.engine_desc_receiver {
                    if let Ok(Ok(desc)) = recv.try_recv() {
                        self.engine_data.desc = Some(desc.clone());
                    }
                }
                if let Some(desc) = &mut self.engine_data.desc {
                    ui.heading(desc.name.clone());
                    ui.label(desc.text_description.clone());

                    let checkpoint = &mut desc.best_available_variant;
                    egui::ComboBox::from_label("variant_selection")
                        .selected_text(checkpoint.name.to_string())
                        .show_ui(ui, |ui| {
                            for variant in &desc.variants {
                                ui.selectable_value(
                                    checkpoint,
                                    variant.clone(),
                                    variant.name.clone(),
                                );
                            }
                        });
                    self.engine_data.variant = Some(checkpoint.clone());
                }
            }
            if let Some(variant) = &self.engine_data.variant {
                if ui.button("Play").clicked() {
                    self.chessboard =
                        ChessBoard::new(Color::Black, variant.clone(), self.sender.clone())
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Unchessful Games");
            self.chessboard.update();
            egui::Area::new("board_area")
                .anchor(Align2::CENTER_CENTER, [0f32, 0f32])
                .movable(false)
                .show(ctx, |ui| {
                    self.chessboard.show(ctx, ui);
                });
        });
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

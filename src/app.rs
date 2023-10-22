use egui::{Align2, Color32, Image, ImageButton};
use shakmaty::{Chess, Color, Move, Piece, Position, Square};

fn square_size(ctx: &egui::Context) -> f32 {
    // if let Some(area_rect) = ctx.memory(|mem| mem.area_rect("board_area")) {
    //     (area_rect.height().min(area_rect.width()) / 8f32).min(80f32)
    // } else {
    (ctx.screen_rect().height().min(ctx.screen_rect().width()) / 8f32).min(80f32)
    // }
}

fn load_image_for_piece(ctx: &egui::Context, piece: Piece) -> Image<'static> {
    let img = match piece.color {
        shakmaty::Color::Black => match piece.role {
            shakmaty::Role::Pawn => Image::new(egui::include_image!("../assets/bp.svg")),
            shakmaty::Role::Knight => Image::new(egui::include_image!("../assets/bn.svg")),
            shakmaty::Role::Bishop => Image::new(egui::include_image!("../assets/bb.svg")),
            shakmaty::Role::Rook => Image::new(egui::include_image!("../assets/br.svg")),
            shakmaty::Role::Queen => Image::new(egui::include_image!("../assets/bq.svg")),
            shakmaty::Role::King => Image::new(egui::include_image!("../assets/bk.svg")),
        },
        shakmaty::Color::White => match piece.role {
            shakmaty::Role::Pawn => Image::new(egui::include_image!("../assets/wp.svg")),
            shakmaty::Role::Knight => Image::new(egui::include_image!("../assets/wn.svg")),
            shakmaty::Role::Bishop => Image::new(egui::include_image!("../assets/wb.svg")),
            shakmaty::Role::Rook => Image::new(egui::include_image!("../assets/wr.svg")),
            shakmaty::Role::Queen => Image::new(egui::include_image!("../assets/wq.svg")),
            shakmaty::Role::King => Image::new(egui::include_image!("../assets/wk.svg")),
        },
    };

    let square_size = square_size(ctx);
    img.maintain_aspect_ratio(true)
        .fit_to_exact_size([square_size, square_size].into())
}

pub struct ChessApp {
    board: Chess,
    player_color: Color,
    selected_piece: Option<Piece>,
    selected_square: Option<Square>,
}

impl Default for ChessApp {
    fn default() -> Self {
        Self {
            board: Chess::default(),
            player_color: Color::Black,
            selected_piece: None,
            selected_square: None,
        }
    }
}

impl ChessApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn show_board(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let square_size = square_size(ctx);
        egui::Grid::new("chess_board")
            .min_col_width(square_size)
            .min_row_height(square_size)
            .max_col_width(square_size)
            .spacing([0f32, 0f32])
            .show(ui, |ui| {
                let mut legal_moves = self.board.legal_moves();
                if let Some(selected_piece) = self.selected_piece {
                    legal_moves.retain(|m| {
                        m.from() == self.selected_square && m.role() == selected_piece.role
                    });
                }
                let (legal_squares, colors): (Vec<Square>, Vec<Color32>) =
                    if self.selected_piece.is_none() {
                        (vec![], vec![])
                    } else {
                        legal_moves
                            .iter()
                            .map(|m| match m {
                                Move::Normal { to, capture, .. } => {
                                    if capture.is_some() {
                                        (*to, Color32::RED)
                                    } else {
                                        (*to, Color32::GREEN)
                                    }
                                }
                                Move::EnPassant { to, .. } => (*to, Color32::RED),
                                Move::Castle { .. } => (
                                    m.castling_side()
                                        .unwrap()
                                        .king_to(self.selected_piece.unwrap().color),
                                    Color32::GREEN,
                                ),
                                Move::Put { .. } => {
                                    unreachable!("There should be no `put` move in a normal game.")
                                }
                            })
                            .unzip()
                    };
                for row in 0..8 {
                    for column in 0..8 {
                        let idx = row * 8 + column;
                        let square = Square::new(idx);

                        let mut square_color = if self.selected_square.is_some()
                            && square == self.selected_square.unwrap()
                        {
                            Color32::from_rgb(74, 185, 219)
                        } else if square.is_dark() {
                            Color32::from_rgb(200, 133, 69)
                        } else {
                            // square.is_light()
                            Color32::from_rgb(244, 197, 151)
                        };

                        if legal_squares.contains(&square) {
                            square_color =
                                colors[legal_squares.iter().position(|v| *v == square).unwrap()]
                        }
                        if let Some(piece) = self.board.board().piece_at(square) {
                            let piece_img = ImageButton::new(
                                load_image_for_piece(ctx, piece).bg_fill(square_color),
                            )
                            .frame(false);

                            if ui.add(piece_img).clicked() {
                                // TODO: play against yourself
                                if self.board.turn() == piece.color
                                // && self.player_color == piece.color
                                {
                                    // Slecting own piece
                                    self.selected_piece = Some(piece);
                                    self.selected_square = Some(square);
                                } else {
                                    // Attacking an enemy piece
                                    if legal_squares.contains(&square) {
                                        self.board.play_unchecked(
                                            &legal_moves[legal_squares
                                                .iter()
                                                .position(|v| *v == square)
                                                .unwrap()],
                                        );
                                    }
                                }
                            };
                        } else {
                            // Image to be placed on empty squares
                            let texture = ctx.load_texture(
                                "square",
                                egui::ColorImage::new(
                                    [square_size as usize, square_size as usize],
                                    square_color,
                                ),
                                Default::default(),
                            );

                            if legal_squares.contains(&square) {
                                if ui
                                    .add(
                                        ImageButton::new(Image::new((
                                            texture.id(),
                                            texture.size_vec2(),
                                        )))
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    // We can guarantee legality as the square becomes a button
                                    // only when it is a destination of a legal move of the
                                    // selected piece
                                    self.board.play_unchecked(
                                        &legal_moves[legal_squares
                                            .iter()
                                            .position(|v| *v == square)
                                            .unwrap()],
                                    );
                                };
                            } else {
                                ui.image((texture.id(), texture.size_vec2()));
                            };
                        }
                    }
                    ui.end_row();
                }
            });
    }
}

impl eframe::App for ChessApp {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Chess Tournament");

            egui::Area::new("board_area")
                .anchor(Align2::CENTER_CENTER, [0f32, 0f32])
                .movable(false)
                .show(ctx, |ui| {
                    self.show_board(ctx, ui);
                });
        });

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "footer").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
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

use egui::{Align2, Color32, Image, ImageButton, Sense};
use shakmaty::{Board, Chess, Color, Move, MoveList, Piece, Position, Square};

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

struct SquareColor {}

impl SquareColor {
    const SELECTED: Color32 = Color32::from_rgb(74, 185, 219);
    const DARK: Color32 = Color32::from_rgb(200, 133, 69);
    const LIGHT: Color32 = Color32::from_rgb(244, 197, 151);
    // TODO: better colors
    const MOVE_TARGET: Color32 = Color32::LIGHT_GREEN;
    const ATTACK_TARGET: Color32 = Color32::LIGHT_RED;
    const LAST_MOVE: Color32 = Color32::KHAKI;
}

struct PieceSelection {
    piece: Piece,
    position: Square,
    legal_moves: Vec<(Square, Move)>,
}

impl PieceSelection {
    fn new(piece: Piece, position: Square, chess: &Chess) -> Self {
        let mut legal_moves = chess.legal_moves();
        legal_moves.retain(|m| m.from() == Some(position) && m.role() == piece.role);
        let legal_moves = legal_moves
            .iter()
            .map(|m| match m {
                Move::Normal { to, .. } => (*to, m.clone()),
                Move::EnPassant { to, .. } => (*to, m.clone()),
                Move::Castle { .. } => (m.castling_side().unwrap().king_to(piece.color), m.clone()),
                Move::Put { .. } => {
                    unreachable!("There should be no `put` move in a normal game.")
                }
            })
            .collect::<Vec<(Square, Move)>>();

        Self {
            piece,
            position,
            legal_moves,
        }
    }

    fn can_move_to(&self, square: Square) -> bool {
        self.legal_moves.iter().any(|v| v.0 == square)
    }
}

struct LastMove {
    a: Square,
    b: Square,
}

pub struct ChessApp {
    board: Chess,
    player_color: Color,
    selection: Option<PieceSelection>,
    last_move: Option<LastMove>,
}

impl Default for ChessApp {
    fn default() -> Self {
        Self {
            board: Chess::default(),
            player_color: Color::Black,
            selection: None,
            last_move: None,
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
                for row in 0..8 {
                    for column in 0..8 {
                        let idx = row * 8 + column;
                        let curr_square = Square::new(idx);

                        // Figure out the color of the current square
                        let square_color = {
                            let mut color = if Some(curr_square)
                                == self.last_move.as_ref().map(|s| s.a)
                                || Some(curr_square) == self.last_move.as_ref().map(|s| s.b)
                            {
                                SquareColor::LAST_MOVE
                            } else if curr_square.is_dark() {
                                SquareColor::DARK
                            } else {
                                // if curr_square.is_light()
                                SquareColor::LIGHT
                            };

                            if self.selection.is_some() {
                                let selection = self.selection.as_ref().unwrap();
                                if curr_square == selection.position {
                                    color = SquareColor::SELECTED
                                } else if let Some(square_idx) = selection
                                    .legal_moves
                                    .iter()
                                    .position(|v| v.0 == curr_square)
                                {
                                    color = if selection.legal_moves[square_idx].1.is_capture()
                                        || selection.legal_moves[square_idx].1.is_en_passant()
                                    {
                                        SquareColor::ATTACK_TARGET
                                    } else {
                                        SquareColor::MOVE_TARGET
                                    }
                                }
                            }
                            color
                        };

                        enum SquareContent {
                            HasPiece(Piece),
                            Empty,
                        }

                        // Produce square image and figure out if there is a piece on it
                        let (square_img, square_content) = if let Some(piece) =
                            self.board.board().piece_at(curr_square)
                        {
                            let img = ImageButton::new(
                                load_image_for_piece(ctx, piece).bg_fill(square_color),
                            )
                            .frame(false);
                            (img, SquareContent::HasPiece(piece))
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

                            let img =
                                ImageButton::new(Image::new((texture.id(), texture.size_vec2())))
                                    .frame(false)
                                    .sense(
                                        if self
                                            .selection
                                            .as_ref()
                                            .is_some_and(|s| s.can_move_to(curr_square))
                                        {
                                            Sense::click()
                                        } else {
                                            Sense {
                                                click: false,
                                                drag: false,
                                                focusable: false,
                                            }
                                        },
                                    );
                            (img, SquareContent::Empty)
                        };

                        let can_be_moved_to_square = self
                            .selection
                            .as_ref()
                            .and_then(|s| s.legal_moves.iter().position(|m| m.0 == curr_square));

                        // Perform actions based on the input
                        if ui.add(square_img).clicked() {
                            match square_content {
                                SquareContent::HasPiece(piece) => {
                                    if self.board.turn() == piece.color
                                        && self.player_color == piece.color
                                    {
                                        // Selecting own piece
                                        self.selection = Some(PieceSelection::new(
                                            piece,
                                            curr_square,
                                            &self.board,
                                        ));
                                    } else {
                                        // Attacking opponent's piece
                                        if let Some(idx) = can_be_moved_to_square {
                                            let m = &self.selection.as_ref().unwrap().legal_moves
                                                [idx]
                                                .1;
                                            // We can use `play_unchecked` because only the legal
                                            // squares ever become interactable
                                            self.board.play_unchecked(m);
                                            self.last_move = Some(LastMove {
                                                a: m.from().unwrap(),
                                                b: m.to(),
                                            });
                                            self.selection = None;
                                        }
                                    }
                                }
                                SquareContent::Empty => {
                                    if let Some(idx) = can_be_moved_to_square {
                                        let m =
                                            &self.selection.as_ref().unwrap().legal_moves[idx].1;
                                        self.board.play_unchecked(m);
                                        self.last_move = Some(if m.is_castle() {
                                            m.castling_side()
                                                .map(|s| LastMove {
                                                    a: s.king_to(
                                                        self.selection
                                                            .as_ref()
                                                            .unwrap()
                                                            .piece
                                                            .color,
                                                    ),
                                                    b: s.rook_to(
                                                        self.selection
                                                            .as_ref()
                                                            .unwrap()
                                                            .piece
                                                            .color,
                                                    ),
                                                })
                                                .unwrap()
                                        } else {
                                            LastMove {
                                                a: m.from().unwrap(),
                                                b: m.to(),
                                            }
                                        });
                                        self.selection = None;
                                    }
                                }
                            }
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

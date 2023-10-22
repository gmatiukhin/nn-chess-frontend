use egui::{Image, ImageButton, Sense};
use shakmaty::{Chess, Color, Move, Piece, Position, Square};

mod utils;

use utils::*;

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

pub struct ChessBoard {
    chess: Chess,
    player_color: Color,
    selection: Option<PieceSelection>,
    last_move: Option<LastMove>,
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self {
            chess: Chess::default(),
            player_color: Color::Black,
            selection: None,
            last_move: None,
        }
    }
}

impl ChessBoard {
    pub fn new(chess: Chess, player_color: Color) -> Self {
        Self {
            chess,
            player_color,
            ..Default::default()
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                            self.chess.board().piece_at(curr_square)
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
                                    if self.chess.turn() == piece.color
                                        && self.player_color == piece.color
                                    {
                                        // Selecting own piece
                                        self.selection = Some(PieceSelection::new(
                                            piece,
                                            curr_square,
                                            &self.chess,
                                        ));
                                    } else {
                                        // Attacking opponent's piece
                                        if let Some(idx) = can_be_moved_to_square {
                                            let m = &self.selection.as_ref().unwrap().legal_moves
                                                [idx]
                                                .1;
                                            // We can use `play_unchecked` because only the legal
                                            // squares ever become interactable
                                            self.chess.play_unchecked(m);
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
                                        self.chess.play_unchecked(m);
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

use egui::{Color32, ImageButton};
use shakmaty::{fen::Fen, san::San, Chess, Color, Move, Piece, Position, Role, Square};

mod utils;

use tokio::sync::mpsc;
use utils::*;
use web_types::{EngineVariant, GameMoveResponse};

use crate::requests::RequestLoopComm;

#[derive(Debug)]
pub(crate) struct AiGameSettings {
    engine_move_receiver: Option<oneshot::Receiver<anyhow::Result<GameMoveResponse>>>,
    ai_variant: EngineVariant,
    sender: mpsc::Sender<crate::requests::RequestLoopComm>,
}

impl AiGameSettings {
    pub fn new(
        variant: EngineVariant,
        sender: mpsc::Sender<crate::requests::RequestLoopComm>,
    ) -> Self {
        log::info!("Reconfiguring AiGameSettings: variant: {variant:?}");
        AiGameSettings {
            engine_move_receiver: None,
            ai_variant: variant,
            sender,
        }
    }
}

pub(crate) enum GameMode {
    PlayAgainsAI(AiGameSettings),
    PlayAgainsYourself,
}

impl PartialEq for GameMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GameMode::PlayAgainsAI(_), GameMode::PlayAgainsAI(_)) => true,
            (GameMode::PlayAgainsAI(_), GameMode::PlayAgainsYourself) => false,
            (GameMode::PlayAgainsYourself, GameMode::PlayAgainsAI(_)) => false,
            (GameMode::PlayAgainsYourself, GameMode::PlayAgainsYourself) => true,
        }
    }
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
}

struct LastMove {
    a: Square,
    b: Square,
}

pub(crate) struct ChessBoard {
    chess: Chess,
    pub(crate) player_color: Color,
    pub(crate) game_mode: GameMode,
    selection: Option<PieceSelection>,
    last_move: Option<LastMove>,
    last_ai_move: Option<GameMoveResponse>,
    game_is_going: bool,
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self {
            chess: Chess::default(),
            player_color: Color::White,
            game_mode: GameMode::PlayAgainsYourself,
            selection: None,
            last_move: None,
            last_ai_move: None,
            game_is_going: false,
        }
    }
}

impl ChessBoard {
    pub(crate) fn start_game(&mut self) {
        self.chess = Chess::default();
        self.selection = None;
        self.last_move = None;
        self.last_ai_move = None;
        self.game_is_going = true;
    }

    pub fn stop_game(&mut self) {
        self.game_is_going = false;
    }

    pub fn last_ai_move_info(&self) -> Option<GameMoveResponse> {
        self.last_ai_move.clone()
    }

    fn play_move(&mut self, m: &Move) {
        // We can use `play_unchecked` because only the legal
        // squares ever become interactable
        self.chess.play_unchecked(m);
        log::debug!("Move played: {m:?}");
        if let Move::EnPassant { .. } = m {
            log::warn!("Holy Hell!");
        }
        self.last_move = Some(if let Move::Castle { king, .. } = m {
            LastMove {
                a: *king,
                b: Square::from_coords(m.castling_side().unwrap().rook_to_file(), king.rank()),
            }
        } else {
            LastMove {
                a: m.from().unwrap(),
                b: m.to(),
            }
        });
        self.selection = None;

        // If the game is now over, then it is not going.
        if self.chess.is_game_over() {
            self.game_is_going = false;
        }
    }

    pub fn is_waiting_for_ai_move(&self) -> bool {
        if let GameMode::PlayAgainsAI(ai_game_settings) = &self.game_mode {
            return ai_game_settings.engine_move_receiver.is_some();
        }

        false
    }

    pub fn update_ai_move(&mut self) {
        if self.chess.turn() == self.player_color
            || self.game_mode == GameMode::PlayAgainsYourself
            || !self.game_is_going
        {
            return;
        }
        if let GameMode::PlayAgainsAI(ai_game_settings) = &mut self.game_mode {
            if let Some(move_receiver) = &ai_game_settings.engine_move_receiver {
                if let Ok(Ok(m)) = move_receiver.try_recv() {
                    ai_game_settings.engine_move_receiver = None;
                    self.play_move(
                        &San::from_ascii(m.move_san.as_bytes())
                            .unwrap()
                            .to_move(&self.chess)
                            .unwrap(),
                    );
                    self.last_ai_move = Some(m);
                }
            } else {
                let fen = Fen::from_position(self.chess.clone(), shakmaty::EnPassantMode::Legal);
                let (sender, receiver) = oneshot::channel();
                let req =
                    RequestLoopComm::FetchPosEval(ai_game_settings.ai_variant.clone(), fen, sender);
                ai_game_settings
                    .sender
                    .try_send(req)
                    .expect("error communicating with request loop");
                ai_game_settings.engine_move_receiver = Some(receiver);
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Grid::new("chess_board")
            .spacing([0f32, 0f32])
            .show(ui, |ui| {
                for row in 0..8 {
                    for column in 0..8 {
                        let (mut row, mut column) = (row, column);
                        if self.player_color == Color::White {
                            row = 7 - row;
                        } else {
                            column = 7 - column;
                        }
                        let idx = row * 8 + column;
                        let curr_square = Square::new(idx);
                        self.draw_square(curr_square, ctx, ui)
                    }
                    ui.end_row();
                }
            });
    }

    fn draw_square(&mut self, square: Square, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Figure out the color of the current square
        let square_color = {
            let mut color = if Some(square) == self.last_move.as_ref().map(|s| s.a)
                || Some(square) == self.last_move.as_ref().map(|s| s.b)
            {
                SquareColor::LAST_MOVE
            } else if square.is_dark() {
                SquareColor::DARK
            } else {
                // if square.is_light()
                SquareColor::LIGHT
            };

            if self.selection.is_some() {
                let selection = self.selection.as_ref().unwrap();
                if square == selection.position {
                    color = SquareColor::SELECTED
                } else if let Some(square_idx) =
                    selection.legal_moves.iter().position(|v| v.0 == square)
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

        let piece = self.chess.board().piece_at(square);
        let who_is_checkmated = self.chess.is_checkmate().then_some(self.chess.turn());
        if who_is_checkmated.is_some() {
            self.game_is_going = false;
        }

        // To tint: the square must contain a piece.
        let check_tint = if let Some(p) = piece {
            // If the piece is a king,
            if let Piece {
                color,
                role: Role::King,
            } = p
            {
                // and it is in check
                if self
                    .chess
                    .board()
                    .attacks_to(square, color.other(), self.chess.board().occupied())
                    .any()
                {
                    // Then tint it
                    PieceTint::IN_CHECK
                } else {
                    // king but not in check
                    Color32::WHITE
                }
            } else if self.chess.checkers().contains(square) {
                // piece is not a king, but is a checker of the king
                PieceTint::CHECKER
            } else {
                // piece is neither a king nor a checker
                Color32::WHITE
            }
        } else {
            // no piece here
            Color32::WHITE
        };
        let img = ImageButton::new(
            load_image_for_piece(ctx, piece, who_is_checkmated)
                .tint(check_tint)
                .bg_fill(square_color),
        )
        .frame(false);

        let can_be_moved_to_square = self
            .selection
            .as_ref()
            .and_then(|s| s.legal_moves.iter().position(|m| m.0 == square));

        // Perform actions based on the input
        if ui
            .add_enabled(
                self.game_is_going,
                img.sense(egui::Sense {
                    click: self.game_is_going,
                    drag: false,
                    focusable: self.game_is_going,
                }),
            )
            .on_disabled_hover_text("The game is not running; please start it in the menu")
            .clicked()
        {
            if let Some(piece) = piece {
                if self.chess.turn() == piece.color
                    && (self.player_color == piece.color
                        || self.game_mode == GameMode::PlayAgainsYourself)
                {
                    // Selecting own piece
                    self.selection = Some(PieceSelection::new(piece, square, &self.chess));
                    return;
                }
            }
            if let Some(idx) = can_be_moved_to_square {
                let m = &self.selection.as_ref().unwrap().legal_moves[idx].1.clone();
                egui::Window::new("My Window").show(ctx, |ui| {
                    ui.label("Hello World!");
                    log::info!("windo");
                });
                self.play_move(m);
            } else {
                // Clicked on blank square that selected piece can't move onto, so deselect
                self.selection = None;
            }
        }
    }
}

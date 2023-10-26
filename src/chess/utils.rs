use egui::{Color32, Image};
use shakmaty::{Color, Piece};

pub fn square_size(ctx: &egui::Context) -> f32 {
    (ctx.screen_rect().height().min(ctx.screen_rect().width()) / 8f32).min(80f32)
}

pub(super) enum AlphaNum {
    Alpha(char),
    Num(u8),
}

pub(super) fn load_image_for_board_labels(
    ctx: &egui::Context,
    alpha_num: AlphaNum,
) -> Image<'static> {
    let img = match alpha_num {
        AlphaNum::Alpha(l) => match l {
            'a' => Image::new(egui::include_image!("../../assets/wk.svg")),
            'b' => Image::new(egui::include_image!("../../assets/b.svg")),
            'c' => Image::new(egui::include_image!("../../assets/c.svg")),
            'd' => Image::new(egui::include_image!("../../assets/d.svg")),
            'e' => Image::new(egui::include_image!("../../assets/e.svg")),
            'f' => Image::new(egui::include_image!("../../assets/f.svg")),
            'g' => Image::new(egui::include_image!("../../assets/g.svg")),
            'h' => Image::new(egui::include_image!("../../assets/h.svg")),
            _ => unreachable!("There are only 8 columns on a chessboard a-h"),
        },
        AlphaNum::Num(n) => match n {
            1 => Image::new(egui::include_image!("../../assets/1.svg")),
            2 => Image::new(egui::include_image!("../../assets/2.svg")),
            3 => Image::new(egui::include_image!("../../assets/3.svg")),
            4 => Image::new(egui::include_image!("../../assets/4.svg")),
            5 => Image::new(egui::include_image!("../../assets/5.svg")),
            6 => Image::new(egui::include_image!("../../assets/6.svg")),
            7 => Image::new(egui::include_image!("../../assets/7.svg")),
            8 => Image::new(egui::include_image!("../../assets/8.svg")),
            _ => unreachable!("There are only 8 rows on a chessboard 1-8"),
        },
    };

    let square_size = square_size(ctx);
    img.maintain_aspect_ratio(true)
        .bg_fill(Color32::RED)
        .fit_to_exact_size(
            match alpha_num {
                AlphaNum::Alpha(_) => [square_size, square_size / 3f32],
                AlphaNum::Num(_) => [square_size / 3f32, square_size],
            }
            .into(),
        )
}

pub(super) fn load_image_for_piece(
    ctx: &egui::Context,
    piece: Option<Piece>,
    who_is_checkmated: Option<Color>,
) -> Image<'static> {
    let img = if let Some(piece) = piece {
        match piece.color {
            shakmaty::Color::Black => match piece.role {
                shakmaty::Role::Pawn => Image::new(egui::include_image!("../../assets/bp.svg")),
                shakmaty::Role::Knight => Image::new(egui::include_image!("../../assets/bn.svg")),
                shakmaty::Role::Bishop => Image::new(egui::include_image!("../../assets/bb.svg")),
                shakmaty::Role::Rook => Image::new(egui::include_image!("../../assets/br.svg")),
                shakmaty::Role::Queen => Image::new(egui::include_image!("../../assets/bq.svg")),
                shakmaty::Role::King => {
                    if who_is_checkmated == Some(Color::Black) {
                        Image::new(egui::include_image!("../../assets/bk-dead.svg"))
                    } else {
                        Image::new(egui::include_image!("../../assets/bk.svg"))
                    }
                }
            },
            shakmaty::Color::White => match piece.role {
                shakmaty::Role::Pawn => Image::new(egui::include_image!("../../assets/wp.svg")),
                shakmaty::Role::Knight => Image::new(egui::include_image!("../../assets/wn.svg")),
                shakmaty::Role::Bishop => Image::new(egui::include_image!("../../assets/wb.svg")),
                shakmaty::Role::Rook => Image::new(egui::include_image!("../../assets/wr.svg")),
                shakmaty::Role::Queen => Image::new(egui::include_image!("../../assets/wq.svg")),
                shakmaty::Role::King => {
                    if who_is_checkmated == Some(Color::White) {
                        Image::new(egui::include_image!("../../assets/wk-dead.svg"))
                    } else {
                        Image::new(egui::include_image!("../../assets/wk.svg"))
                    }
                }
            },
        }
    } else {
        Image::new(egui::include_image!("../../assets/empty.svg"))
    };

    let square_size = square_size(ctx);
    img.maintain_aspect_ratio(true)
        .fit_to_exact_size([square_size, square_size].into())
}

pub struct SquareColor {}

impl SquareColor {
    pub const SELECTED: Color32 = Color32::from_rgb(74, 185, 219);
    pub const DARK: Color32 = Color32::from_rgb(200, 133, 69);
    pub const LIGHT: Color32 = Color32::from_rgb(244, 197, 151);
    // TODO: better colors
    pub const MOVE_TARGET: Color32 = Color32::LIGHT_GREEN;
    pub const ATTACK_TARGET: Color32 = Color32::LIGHT_RED;
    pub const LAST_MOVE: Color32 = Color32::KHAKI;
}

pub struct PieceTint {}

impl PieceTint {
    pub const IN_CHECK: Color32 = Color32::from_rgba_premultiplied(255, 0, 0, 255);
    pub const CHECKER: Color32 = Color32::from_rgba_premultiplied(255, 0, 255, 255);
}

use egui::{Color32, Image};
use shakmaty::Piece;

pub fn square_size(ctx: &egui::Context) -> f32 {
    (ctx.screen_rect().height().min(ctx.screen_rect().width()) / 8f32).min(80f32)
}

pub fn load_image_for_piece(ctx: &egui::Context, piece: Option<Piece>) -> Image<'static> {
    let img = if let Some(piece) = piece {
        match piece.color {
            shakmaty::Color::Black => match piece.role {
                shakmaty::Role::Pawn => Image::new(egui::include_image!("../../assets/bp.svg")),
                shakmaty::Role::Knight => Image::new(egui::include_image!("../../assets/bn.svg")),
                shakmaty::Role::Bishop => Image::new(egui::include_image!("../../assets/bb.svg")),
                shakmaty::Role::Rook => Image::new(egui::include_image!("../../assets/br.svg")),
                shakmaty::Role::Queen => Image::new(egui::include_image!("../../assets/bq.svg")),
                shakmaty::Role::King => Image::new(egui::include_image!("../../assets/bk.svg")),
            },
            shakmaty::Color::White => match piece.role {
                shakmaty::Role::Pawn => Image::new(egui::include_image!("../../assets/wp.svg")),
                shakmaty::Role::Knight => Image::new(egui::include_image!("../../assets/wn.svg")),
                shakmaty::Role::Bishop => Image::new(egui::include_image!("../../assets/wb.svg")),
                shakmaty::Role::Rook => Image::new(egui::include_image!("../../assets/wr.svg")),
                shakmaty::Role::Queen => Image::new(egui::include_image!("../../assets/wq.svg")),
                shakmaty::Role::King => Image::new(egui::include_image!("../../assets/wk.svg")),
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

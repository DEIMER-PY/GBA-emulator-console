//! Vídeo: la textura con el frame de la GBA y su colocación en la ventana.

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2};
use gba_core::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Tamaño nativo de la imagen de la GBA.
pub const GBA_SIZE: Vec2 = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

/// Cómo se escala la imagen a la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scaling {
    /// Escalar solo en múltiplos enteros (×1, ×2, ×3...), sin píxeles desiguales.
    pub integer: bool,
}

/// Textura de GPU con el último frame emulado.
#[derive(Default)]
pub struct Screen {
    texture: Option<TextureHandle>,
}

impl Screen {
    /// Sube un frame RGBA a la GPU.
    pub fn upload(&mut self, ctx: &egui::Context, frame: &[u8; FRAME_BYTES]) {
        let image = ColorImage::from_rgba_unmultiplied([SCREEN_WIDTH, SCREEN_HEIGHT], frame);
        // NEAREST: sin suavizado, para que los píxeles se vean nítidos al escalar.
        match &mut self.texture {
            Some(texture) => texture.set(image, TextureOptions::NEAREST),
            None => {
                self.texture = Some(ctx.load_texture("gba-frame", image, TextureOptions::NEAREST));
            }
        }
    }

    /// Descarta la imagen: la pantalla queda en negro hasta el próximo frame.
    pub fn clear(&mut self) {
        self.texture = None;
    }

    /// Dibuja la imagen centrada en el espacio disponible de `ui`.
    pub fn show(&self, ui: &mut egui::Ui, scaling: Scaling) {
        let available = ui.available_rect_before_wrap();
        ui.painter().rect_filled(available, 0.0, Color32::BLACK);

        if let Some(texture) = &self.texture {
            let rect = fit_rect(available, ui.pixels_per_point(), scaling);
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
        }
        ui.allocate_rect(available, egui::Sense::hover());
    }
}

/// Calcula dónde se dibuja la imagen dentro de `available`, conservando la proporción.
///
/// Con escalado entero el alto es un múltiplo entero en píxeles físicos (×1, ×2, ×3...).
/// `pixels_per_point` es el factor de escala de la pantalla (por ejemplo 1.5 en un
/// monitor al 150 %).
pub fn fit_rect(available: Rect, pixels_per_point: f32, scaling: Scaling) -> Rect {
    // Tamaño de la imagen a escala ×1, en puntos. Los píxeles de la GBA son cuadrados.
    let base = GBA_SIZE;

    let scale = (available.width() / base.x).min(available.height() / base.y);
    let scale = if scaling.integer {
        let physical = (scale * pixels_per_point).floor();
        if physical >= 1.0 {
            physical / pixels_per_point
        } else {
            // La ventana es más pequeña que la imagen a ×1: se reduce para que quepa.
            scale
        }
    } else {
        scale
    };
    Rect::from_center_size(available.center(), base * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTEGER: Scaling = Scaling { integer: true };
    const FREE: Scaling = Scaling { integer: false };

    fn area(width: f32, height: f32) -> Rect {
        Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(width, height))
    }

    #[test]
    fn integer_scaling_uses_whole_multiples() {
        let rect = fit_rect(area(800.0, 600.0), 1.0, INTEGER);
        // 800/240 = 3.3 y 600/160 = 3.75: cabe ×3.
        assert_eq!(rect.size(), Vec2::new(720.0, 480.0));
        assert_eq!(rect.center(), egui::pos2(400.0, 300.0));
    }

    #[test]
    fn integer_scaling_counts_physical_pixels() {
        // Al 150 %, 480×320 puntos son 720×480 píxeles físicos: ×3 exacto.
        let rect = fit_rect(area(480.0, 320.0), 1.5, INTEGER);
        assert_eq!(rect.size(), Vec2::new(480.0, 320.0));
    }

    #[test]
    fn free_scaling_fills_the_limiting_side() {
        let rect = fit_rect(area(800.0, 600.0), 1.0, FREE);
        assert!((rect.width() - 800.0).abs() < 0.01);
        assert!((rect.height() - 533.33).abs() < 0.01);
    }

    #[test]
    fn shrinks_when_window_is_tiny() {
        let rect = fit_rect(area(120.0, 200.0), 1.0, INTEGER);
        assert_eq!(rect.size(), Vec2::new(120.0, 80.0));
    }
}

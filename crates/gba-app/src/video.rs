//! Vídeo: la textura con el frame de la GBA y su colocación en la ventana.

use eframe::egui::{
    self, Color32, ColorImage, Mesh, Pos2, Rect, TextureHandle, TextureOptions, Vec2, pos2,
};
use gba_core::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH};
use serde::{Deserialize, Serialize};

/// Tamaño nativo de la imagen de la GBA.
pub const GBA_SIZE: Vec2 = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

/// Cómo se escala la imagen a la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scaling {
    /// Escalar solo en múltiplos enteros (×1, ×2, ×3...), sin píxeles desiguales.
    pub integer: bool,
    /// Estirar la imagen hasta llenar la ventana, sin conservar la proporción.
    pub stretch: bool,
    /// Giro de la imagen.
    pub rotation: Rotation,
    /// Suavizar la imagen al escalar, en lugar de mostrar los píxeles nítidos.
    pub smooth: bool,
    /// Oscurecer una franja de cada línea, como en un televisor de tubo.
    pub scanlines: bool,
}

#[cfg(test)]
impl Scaling {
    /// Sin ninguna opción: píxeles cuadrados, escala libre, sin giro ni efectos.
    pub const PLAIN: Self = Scaling {
        integer: false,
        stretch: false,
        rotation: Rotation::None,
        smooth: false,
        scanlines: false,
    };
}

/// Giro de la imagen, en el sentido de las agujas del reloj.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    None,
    /// 90° a la derecha: útil con el monitor girado o con juegos en vertical.
    Right,
    UpsideDown,
    Left,
}

impl Rotation {
    pub const ALL: [Self; 4] = [Self::None, Self::Right, Self::UpsideDown, Self::Left];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "Sin girar",
            Self::Right => "90° a la derecha",
            Self::UpsideDown => "180°",
            Self::Left => "90° a la izquierda",
        }
    }

    /// Intercambia ancho y alto.
    pub const fn is_sideways(self) -> bool {
        matches!(self, Self::Right | Self::Left)
    }

    /// Punto de `rect` donde se ve la posición `(u, v)` de la imagen, con `u` y `v`
    /// entre 0 y 1 (`(0, 0)` es la esquina superior izquierda de la imagen).
    pub fn point(self, rect: Rect, u: f32, v: f32) -> Pos2 {
        let r = rect;
        match self {
            Self::None => pos2(r.min.x + u * r.width(), r.min.y + v * r.height()),
            Self::Right => pos2(r.max.x - v * r.width(), r.min.y + u * r.height()),
            Self::UpsideDown => pos2(r.max.x - u * r.width(), r.max.y - v * r.height()),
            Self::Left => pos2(r.min.x + v * r.width(), r.max.y - u * r.height()),
        }
    }
}

/// Textura de GPU con el último frame emulado.
#[derive(Default)]
pub struct Screen {
    texture: Option<TextureHandle>,
}

/// Opacidad de la franja oscura de cada línea con `scanlines`.
const SCANLINE_ALPHA: u8 = 90;
/// Parte de cada línea que se oscurece.
const SCANLINE_SHARE: f32 = 0.45;

impl Screen {
    /// Sube un frame RGBA a la GPU. Con `smooth` la imagen se suaviza al escalar; sin
    /// él cada píxel se ve nítido.
    pub fn upload(&mut self, ctx: &egui::Context, frame: &[u8; FRAME_BYTES], smooth: bool) {
        let image = ColorImage::from_rgba_unmultiplied([SCREEN_WIDTH, SCREEN_HEIGHT], frame);
        let options = if smooth {
            TextureOptions::LINEAR
        } else {
            TextureOptions::NEAREST
        };
        match &mut self.texture {
            Some(texture) => texture.set(image, options),
            None => self.texture = Some(ctx.load_texture("gba-frame", image, options)),
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
            ui.painter()
                .add(image_mesh(texture.id(), rect, scaling.rotation));
            if scaling.scanlines {
                ui.painter().add(scanlines_mesh(rect, scaling.rotation));
            }
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
    if scaling.stretch {
        return available;
    }
    let base = if scaling.rotation.is_sideways() {
        Vec2::new(base.y, base.x)
    } else {
        base
    };

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

/// Cuadrilátero con la imagen, girado.
fn image_mesh(texture: egui::TextureId, rect: Rect, rotation: Rotation) -> egui::Shape {
    let mut mesh = Mesh::with_texture(texture);
    for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: rotation.point(rect, u, v),
            uv: pos2(u, v),
            color: Color32::WHITE,
        });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    egui::Shape::mesh(mesh)
}

/// Franjas oscuras sobre la parte baja de cada línea de la imagen, siguiendo el giro.
fn scanlines_mesh(rect: Rect, rotation: Rotation) -> egui::Shape {
    let mut mesh = Mesh::default();
    let color = Color32::from_black_alpha(SCANLINE_ALPHA);
    let lines = GBA_SIZE.y;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    for line in 0..lines as u32 {
        #[allow(clippy::cast_precision_loss)]
        let line = line as f32;
        let (v0, v1) = ((line + 1.0 - SCANLINE_SHARE) / lines, (line + 1.0) / lines);
        let base = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
        for (u, v) in [(0.0, v0), (1.0, v0), (1.0, v1), (0.0, v1)] {
            mesh.colored_vertex(rotation.point(rect, u, v), color);
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    egui::Shape::mesh(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTEGER: Scaling = Scaling {
        integer: true,
        ..Scaling::PLAIN
    };
    const FREE: Scaling = Scaling {
        integer: false,
        ..Scaling::PLAIN
    };

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

    #[test]
    fn stretch_fills_the_whole_window() {
        let scaling = Scaling {
            stretch: true,
            ..Scaling::PLAIN
        };
        assert_eq!(
            fit_rect(area(800.0, 300.0), 1.0, scaling),
            area(800.0, 300.0)
        );
    }

    #[test]
    fn sideways_rotation_swaps_width_and_height() {
        let upright = fit_rect(area(2000.0, 2000.0), 1.0, Scaling::PLAIN);
        let turned = Scaling {
            rotation: Rotation::Right,
            ..Scaling::PLAIN
        };
        let rect = fit_rect(area(2000.0, 2000.0), 1.0, turned);
        assert!((rect.width() / rect.height() - upright.height() / upright.width()).abs() < 1e-4);
        // La esquina superior izquierda de la imagen queda arriba a la derecha.
        assert_eq!(Rotation::Right.point(rect, 0.0, 0.0), rect.right_top());
        assert_eq!(Rotation::Left.point(rect, 0.0, 0.0), rect.left_bottom());
        assert_eq!(
            Rotation::UpsideDown.point(rect, 0.0, 0.0),
            rect.right_bottom()
        );
    }
}

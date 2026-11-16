//! API de emulación de Game Boy Advance.
//!
//! Este crate aísla al resto del proyecto del core de emulación concreto
//! ([rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng), copiado en
//! `vendor/`). La app solo conoce [`Emulator`] y los tipos de este crate, de modo que
//! el core se puede sustituir sin tocarla.
//!
//! ```
//! use gba_core::{Button, Emulator};
//!
//! let mut gba = Emulator::new();
//! gba.load_rom("demo", &gba_core::demo_rom())?;
//! gba.set_button(Button::A, true);
//! gba.run_frame()?;
//!
//! assert_eq!(gba.frame().len(), gba_core::FRAME_BYTES);
//! # Ok::<(), gba_core::Error>(())
//! ```

mod demo;
mod emulator;

pub use demo::demo_rom;
pub use emulator::{Button, Emulator, Error, ResetKind, RomInfo};

/// Ancho de la pantalla de la GBA en píxeles.
pub const SCREEN_WIDTH: usize = 240;
/// Alto de la pantalla de la GBA en píxeles.
pub const SCREEN_HEIGHT: usize = 160;
/// Tamaño en bytes de un frame RGBA (4 bytes por píxel).
pub const FRAME_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
/// Frecuencia de muestreo de audio por defecto, en Hz.
pub const DEFAULT_SAMPLE_RATE: f32 = 48_000.0;
/// Canales de audio: la GBA es estéreo (muestras intercaladas izquierda, derecha).
pub const AUDIO_CHANNELS: usize = 2;
/// Frames por segundo de la GBA: 16 777 216 Hz / 280 896 ciclos por frame (~59,73).
pub const FRAME_RATE: f64 = 16_777_216.0 / 280_896.0;
/// Tamaño de una BIOS de GBA en bytes.
pub const BIOS_SIZE: usize = 16 * 1024;

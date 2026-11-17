//! Mide la velocidad de emulación del core.
//!
//! ```sh
//! cargo run -p gba-core --release --example bench              # ROM de demostración
//! cargo run -p gba-core --release --example bench -- juego.gba # una ROM propia
//! ```

use std::time::Instant;

use gba_core::{Emulator, FRAME_RATE};

/// Frames que se emulan en cada medición (unos 30 segundos de juego).
const FRAMES: u32 = 1_800;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (name, rom) = match std::env::args().nth(1) {
        Some(path) => (path.clone(), std::fs::read(path)?),
        None => ("demo".to_owned(), gba_core::demo_rom()),
    };

    let mut gba = Emulator::new();
    gba.load_rom(&name, &rom)?;
    println!("ROM: {name}");

    type Step = fn(&mut Emulator) -> Result<(), gba_core::Error>;
    let modes: [(&str, Step); 3] = [
        ("imagen + audio", Emulator::run_frame),
        ("solo imagen", Emulator::run_frame_silent),
        ("frame saltado", Emulator::skip_frame),
    ];
    for (label, step) in modes {
        let start = Instant::now();
        for _ in 0..FRAMES {
            step(&mut gba)?;
        }
        let fps = f64::from(FRAMES) / start.elapsed().as_secs_f64();
        println!(
            "{label:>15}: {fps:8.1} fps  ({:.1}x velocidad real)",
            fps / FRAME_RATE
        );
    }
    println!(
        "{:>15}: {} KiB",
        "save state",
        gba.save_state()?.len() / 1024
    );
    Ok(())
}

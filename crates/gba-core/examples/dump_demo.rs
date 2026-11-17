//! Escribe la ROM de demostración en un archivo `.gba`.
//!
//! Sirve para probar la carga de ROMs desde disco sin usar ROMs de terceros:
//!
//! ```sh
//! cargo run -p gba-core --example dump_demo -- demo.gba
//! ```

use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("uso: dump_demo <archivo.gba>");
        return ExitCode::FAILURE;
    };
    match fs::write(&path, gba_core::demo_rom()) {
        Ok(()) => {
            println!("ROM de demostración escrita en {}", path.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("no se pudo escribir {}: {err}", path.display());
            ExitCode::FAILURE
        }
    }
}

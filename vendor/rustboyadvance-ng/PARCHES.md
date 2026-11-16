# rustboyadvance-ng (copia local)

Copia de los crates `arm7tdmi`, `core` y `utils` de
[michelhe/rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng) (licencia MIT, ver `LICENSE`),
tomada del commit `1d7ff23a6adb42d803ebd393c3d27d94e4fea15e` (2026-08-27).

`external/gba-tests` contiene las ROMs de prueba de
[jsmolka/gba-tests](https://github.com/jsmolka/gba-tests) (MIT), que usan los tests del core.

## Cambios respecto al original

Todos están marcados en el código con `PARCHE gba-emulator-console`.

| Archivo | Cambio |
|---|---|
| `core/src/cartridge/backup/backup_file.rs` | La memoria de guardado se conserva al clonar y al serializar (antes se releía del archivo), y se puede cargar desde bytes. Permite guardar partida sin disco (web) y dentro de los save states. |
| `core/src/cartridge/backup/eeprom.rs`, `flash.rs`, `core/src/cartridge/mod.rs`, `core/src/gba.rs` | `backup_bytes()` / `load_backup_bytes()`: partida guardada como bytes. |
| `core/src/sound/dsp.rs`, `core/src/sound/mod.rs`, `core/src/gba.rs` | `set_audio_sample_rate()`: cambiar la frecuencia de salida en caliente. |
| `core/src/gba.rs` | El desfase de ciclos entre frames pasa de una `static mut` global a un campo de `GameBoyAdvance`: varias consolas (o tests en paralelo) ya no se pisan. |
| `rustfmt.toml` (nuevo) | Desactiva `cargo fmt` en esta carpeta para conservar el formato original. |
| `core/Cargo.toml` | Se quita el benchmark de `criterion` (no se copia `benches/`). |

No se copian los frontends (`platform/`), `fps_bench` ni los benchmarks.

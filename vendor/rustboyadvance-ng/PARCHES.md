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
| `core/src/cartridge/mod.rs` | La memoria de guardado se comporta como un bus de 8 bits: leer 16 o 32 bits repite el byte, escribirlos guarda un solo byte, y sin memoria de guardado se lee `0xFF`. |
| `core/src/gba_tests.rs` | Tests añadidos con la BIOS libre: guardado SRAM, Flash 64K y 128K, cartucho sin guardado, y la suite ARM. Queda documentado un límite conocido: escrituras anchas no alineadas en la memoria de guardado. |
| `core/src/sound/psg.rs` (nuevo), `core/src/sound/mod.rs` | **Canales de sonido PSG** (dos de onda cuadrada, onda programable y ruido), que el original no sintetizaba: registros, secuenciador de 512 Hz, mezcla con el audio DirectSound y bits de estado en `SOUNDCNT_X`. Se corrigen además los anchos de los volúmenes maestros (3 bits) y del volumen PSG (2 bits). |
| `core/src/sound/dsp.rs` | El resampler interpolaba al revés (del valor nuevo al antiguo) y añadía un pico en cada transición de la señal. Afectaba a todo el audio. |
| `rustfmt.toml` (nuevo) | Desactiva `cargo fmt` en esta carpeta para conservar el formato original. |
| `core/Cargo.toml` | Se quita el benchmark de `criterion` (no se copia `benches/`). |

No se copian los frontends (`platform/`), `fps_bench` ni los benchmarks.

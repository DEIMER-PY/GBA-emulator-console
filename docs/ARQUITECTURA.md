# Arquitectura

## Principio

La emulación la aporta un core existente ([rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng), copiado y parcheado en `vendor/`). El proyecto se divide en capas para que la app nunca dependa directamente de ese core.

```
┌─────────────────────────────────────────────┐
│ gba-app  (binario: escritorio y web)        │   ← pendiente
│  ventana, interfaz, audio, controles,       │
│  configuración, guardado en disco           │
└──────────────────────┬──────────────────────┘
                       │ API propia y pequeña
┌──────────────────────┴──────────────────────┐
│ gba-core  (librería)                        │
│  Emulator: cargar ROM, avanzar un frame,    │
│  imagen, audio, botones, save states        │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────┴──────────────────────┐
│ vendor/rustboyadvance-ng  (terceros, MIT)   │
│  core:     PPU, APU, DMA, timers, cartucho  │
│  arm7tdmi: CPU ARM7TDMI (ARM + Thumb)       │
│  utils:    utilidades compartidas           │
└─────────────────────────────────────────────┘
```

Ventajas de esta separación:

- **Sustituible:** se puede cambiar de core, o escribir uno propio, sin tocar la app.
- **Testeable:** `gba-core` se prueba sin ventana ni audio (tests headless).
- **Portable:** `gba-core` no sabe nada de la plataforma.
- **Preparado para la DS:** `arm7tdmi` es un crate independiente que solo conoce un bus de memoria abstracto; la Nintendo DS usa esa misma CPU como procesador secundario.

## Estructura del repositorio

```
├── Cargo.toml                 workspace (solo el código propio)
├── crates/
│   └── gba-core/              API de emulación
│       ├── bios/              BIOS libre de Cult-of-GBA (MIT)
│       ├── examples/bench.rs  medidor de velocidad
│       ├── src/
│       └── tests/
├── vendor/
│   └── rustboyadvance-ng/     core de terceros (fuera del workspace)
│       ├── PARCHES.md         origen y cambios aplicados
│       ├── arm7tdmi/  core/  utils/
│       └── external/gba-tests ROMs de prueba (MIT)
└── docs/
```

`vendor/` está excluido del workspace: se compila como dependencia, pero `cargo fmt`, `cargo clippy` y `cargo test --workspace` solo actúan sobre el código propio.

## `gba-core`

| Módulo | Contenido |
|---|---|
| `emulator.rs` | `Emulator` y sus tipos: `Button`, `ResetKind`, `RomInfo`, `Error` |
| `demo.rs` | `demo_rom()`: ROM propia generada en memoria, para tests y pantalla de prueba |
| `lib.rs` | Constantes: `SCREEN_WIDTH`, `SCREEN_HEIGHT`, `FRAME_BYTES`, `FRAME_RATE`, `DEFAULT_SAMPLE_RATE`, `AUDIO_CHANNELS`, `BIOS_SIZE` |

### API de `Emulator`

| Método | Qué hace |
|---|---|
| `new()` | Consola sin cartucho, BIOS libre incluida, audio a 48 kHz |
| `set_bios(bytes)` / `use_open_bios()` | Usar un volcado de la BIOS original, o volver a la incluida |
| `load_rom(nombre, bytes)` | Carga una ROM `.gba` (o un `.zip` que la contenga) y devuelve su `RomInfo` |
| `unload_rom()` | Extrae el cartucho |
| `run_frame()` | Emula un frame completo de vídeo y su audio |
| `run_frame_silent()` | Emula un frame con imagen pero sin entregar sonido |
| `skip_frame()` | Emula un frame sin entregar imagen ni sonido |
| `frame()` | Imagen del último frame: 240×160 RGBA |
| `audio_samples(&mut Vec)` | Muestras estéreo intercaladas `f32` del último frame |
| `set_sample_rate(hz)` | Ajusta el audio a la frecuencia del dispositivo de salida, también en marcha |
| `set_button(botón, pulsado)` | Estado de un botón |
| `reset(tipo)` | Reinicio suave o duro (conserva la partida guardada) |
| `save_state()` / `load_state(bytes)` | Estado completo de la consola como bytes |
| `battery_ram()` / `set_battery_ram(bytes)` | Partida guardada del cartucho (SRAM, Flash o EEPROM) |

### Diferencias con `nes-core`

| | NES | GBA |
|---|---|---|
| Pantalla | 256×240 | 240×160 |
| Frames por segundo | 60,10 (NTSC) / 50 (PAL) | 59,73, sin regiones |
| Audio | mono | estéreo |
| Botones | 8, dos mandos | 10 (añade L y R), un mando |
| BIOS | no tiene | 16 KiB; se incluye una libre |
| Guardado | SRAM | SRAM, Flash 64K/128K o EEPROM 512B/8K (se detecta por la ROM) |

### Decisiones

- **Sin acceso a disco.** ROMs, estados y partidas entran y salen como bytes: el mismo crate sirve en escritorio y en la web.
- **Tipos propios.** `gba-core` no reexporta ningún tipo del core de terceros.
- **Los `panic!` del core se capturan.** El core se detiene con `panic!` ante hardware que no soporta. `gba-core` lo convierte en `Error::Emulation` para que la app muestre un aviso en lugar de cerrarse. Por eso el perfil `release` no usa `panic = "abort"`.
- **Arranque directo.** Se salta la animación de la BIOS (`skip_bios`); la BIOS sigue atendiendo las llamadas del juego (SWI) y las interrupciones.
- **La partida viaja en el save state.** Cargar un estado restaura también la memoria de guardado de ese momento.
- **No es `Send`.** El core usa `Rc` internamente: el `Emulator` vive en el hilo que lo crea.

### Flujo de un frame

```
app                         gba-core                 core (vendor)
 │  set_button(...)  ───────►  máscara de botones
 │  run_frame()      ───────►  KEYINPUT + frame()  ─►  CPU + PPU + APU
 │                             ◄─ push_sample() ────  audio (remuestreado)
 │  frame()          ◄───────  240×160 RGBA
 │  audio_samples()  ◄───────  ~804 pares de muestras (48 kHz / 59,73 fps)
 ▼
 textura en pantalla + cola de audio
```

### ROM de demostración

`demo_rom()` devuelve un programa ARM de 11 instrucciones ensamblado a mano: pantalla azul con el botón A suelto y roja mientras se mantiene pulsado. Con ella los tests comprueban vídeo, mandos, save states, reset y guardado sin depender de ninguna ROM externa.

## Tests

| Dónde | Qué comprueba | Comando |
|---|---|---|
| `crates/gba-core/tests` | La API propia, con la ROM de demostración | `cargo test --workspace` |
| `vendor/rustboyadvance-ng/core` | CPU ARM/Thumb y memoria con [gba-tests](https://github.com/jsmolka/gba-tests); EEPROM, RTC, planificador | `cargo test --release` dentro de esa carpeta |

La CI ejecuta ambos.

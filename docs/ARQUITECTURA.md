# Arquitectura

## Principio

La emulación la aporta un core existente ([rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng), copiado y parcheado en `vendor/`). El proyecto se divide en capas para que la app nunca dependa directamente de ese core.

```
┌─────────────────────────────────────────────┐
│ gba-app  (binario: escritorio y web)        │
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
│   ├── gba-app/               aplicación (ventana, audio, controles)
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

## `gba-app`

Portada de `nes-app` ([replica-NES](https://github.com/DEIMER-PY/replica-NES)). Aplicación `eframe` (ventana con `winit`, render con `wgpu`, interfaz con `egui`).

| Módulo | Contenido |
|---|---|
| `main.rs` | Punto de entrada nativo: crea la ventana y arranca `GbaApp` |
| `app.rs` | `GbaApp` (estado y lógica), `Action` (órdenes del usuario) y sus atajos; implementa `eframe::App` |
| `ui.rs` | Interfaz: barra de menús, textos sobre la imagen y ventanas |
| `home.rs` | Pantalla de inicio (sin juego cargado): botones para abrir, biblioteca, recientes y aviso al arrastrar un archivo |
| `clock.rs` | `FrameClock`: cuántos frames de la GBA tocan en cada repintado |
| `video.rs` | `Screen` (textura del frame) y `fit_rect` (colocación y escalado) |
| `rom.rs` | `RomFile` (lectura de una ROM del disco), `RecentRoms` (lista de recientes) y `scan_library` (juegos de una carpeta) |
| `config.rs` | `Config`: ajustes persistentes, guardados con el almacenamiento de `eframe` |
| `audio.rs` | `AudioOutput`: salida de sonido con `cpal` y cola de muestras |
| `settings.rs` | Ventana de ajustes (controles, vídeo, audio) y reasignación de teclas |
| `storage_web.rs` | Versión web de `SaveStore`, sobre `localStorage` (ver [WEB.md](WEB.md)) |
| `web.rs` | Solo web: `RomInbox`, lectura asíncrona de ROMs (ver [WEB.md](WEB.md)) |
| `storage.rs` | `SaveStore`: partidas con batería y save states en disco |
| `input.rs` | `KeyBindings` (teclado), `PadBindings` y `Gamepads` (mandos con `gilrs`) y `PadState` (botones pulsados) |

### Ciclo de la app

`eframe` llama a dos métodos en cada repintado de la ventana:

1. **`logic`** — emula. `FrameClock` convierte el tiempo transcurrido en frames de la GBA, se llama a `Emulator::run_frame` esas veces y el último frame se sube a la textura.
2. **`ui`** — dibuja. `Screen::show` pinta la textura centrada sobre fondo negro.

Al final de `logic` se pide siempre otro repintado (`request_repaint`), porque la emulación es continua y no depende de eventos del usuario.

### Acciones

Menús y atajos de teclado no llaman a la lógica directamente: producen un valor de `Action` (`OpenRom`, `TogglePause`, `Reset(kind)`, `SaveState`, `SelectSlot(n)`...) y `GbaApp::perform` lo ejecuta. Así cada orden tiene una sola implementación, y el atajo que muestra el menú es siempre el que de verdad funciona, porque ambos salen de la misma tabla (`Action::SHORTCUTS`).

Los atajos de una sola tecla (`P`, `M`, `F5`...) se ignoran mientras la interfaz usa el teclado, y un test comprueba que ninguno coincide con una tecla de juego por defecto.

### Reasignación de teclas

Al hacer clic en una tecla de la tabla, `SettingsWindow` (en `settings.rs`) entra en modo captura para ese botón. Mientras dura, la app no trata las pulsaciones como atajos ni las envía al juego. La siguiente tecla pulsada pasa por `capture_key`:

| Tecla | Resultado |
|---|---|
| `Escape` | Cancela sin cambios |
| Reservada (`Action::is_reserved_key`) | Se rechaza con un aviso y se sigue esperando |
| Cualquier otra | `KeyBindings::rebind` la asigna |

`rebind` mantiene un invariante: nunca hay una tecla en dos botones ni un botón sin tecla. Si la tecla ya estaba asignada, los dos botones intercambian sus teclas.

Las teclas reservadas se derivan de la tabla de atajos, así que un atajo nuevo queda protegido sin tocar los ajustes.

### Pausa y avance rápido

- **Pausa:** no se emula y se reinicia `FrameClock`, para que al reanudar no se intente recuperar el tiempo que duró la pausa.
- **Avance rápido:** mientras se mantiene `F`, `FrameClock` trabaja a ×4 con un tope de 8 frames por repintado. El audio no se genera, porque sonaría acelerado y desbordaría la cola. De los frames de cada repintado solo se dibuja el último (`run_frame_silent`); los anteriores se emulan con `skip_frame`.
- **En reposo:** en pausa o sin juego la ventana no se repinta de forma continua, sino cada 50 ms, lo justo para atender a los mandos y retirar los avisos. Así la app no ocupa la CPU ni la gráfica mientras espera.

### Rendimiento

Una GBA real produce 59,73 frames por segundo; el core emula unas diez veces más con la ROM de demostración en el equipo de desarrollo. Para medirlo sin ventana ni sonido:

```sh
cargo run -p gba-core --release --example bench                 # ROM de demostración
cargo run -p gba-core --release --example bench -- juego.gba    # una ROM propia
```

### Partidas y estados guardados

`SaveStore` guarda dos tipos de archivo en la carpeta de datos de la app, con el identificador de la ROM (`nombre-huella`, donde la huella es un FNV-1a del contenido) como nombre:

| Tipo | Archivo | Cuándo se escribe |
|---|---|---|
| Partida del cartucho | `saves/<rom>.sav` | Cada 30 s (`App::save`), al cambiar o cerrar la ROM y al salir; solo si cambió |
| Save state | `states/<rom>.<ranura>.state` | Cuando el usuario lo pide |

La escritura es atómica (archivo temporal y renombrado): si la app se cierra a mitad, la partida anterior queda intacta.

Al cargar un estado se emula un frame antes de mostrar la imagen, porque el estado serializado no incluye el frame buffer.

### Carga de ROMs

Todas las vías (menú, `Ctrl+O`, arrastrar y soltar, argumento de línea de comandos, recientes) acaban en el mismo camino:

```
ruta ──► RomFile::read ──► GbaApp::load_rom ──► Emulator::load_rom
              │                    │
              └── error de E/S     └── error de formato
                         └──────────────┴──► ventana de error
```

`gba-core` solo recibe bytes; leer el disco es responsabilidad de la app. Una ROM que carga bien se anota en recientes; una ruta que ya no existe se elimina de la lista.

### Controles

Antes de emular, en cada repintado, la app calcula el estado de los diez botones de la GBA:

```
teclado ──► KeyBindings::read ──► PadState ─┐
                                            ├─ merge (OR) ──► Emulator::set_button × 10
mandos  ──► Gamepads::read    ──► PadState ─┘
```

Los tres tipos son el mismo genérico, `ButtonMap<T>` (un valor por botón): `PadState = ButtonMap<bool>`, `KeyBindings = ButtonMap<Key>` y `PadBindings = ButtonMap<PadInput>`. Buscar, reasignar e intercambiar se escriben una sola vez.

- **Un solo jugador.** Todos los mandos conectados se unen: cualquiera controla la consola.
- **Estado, no eventos.** Se pregunta qué teclas están pulsadas ahora, en lugar de procesar pulsaciones y sueltas. Así no se pierde ningún cambio aunque un repintado emule varios frames.
- **El teclado se cede a la interfaz.** Si `egui` está usando el teclado, o hay `Ctrl`/`Alt` pulsado, no se envía nada al juego.
- **Los mandos son opcionales.** Si `gilrs` no puede inicializarse, la app sigue funcionando solo con teclado.
- **Teclas y botones configurables.** `KeyBindings` (teclado) y `PadBindings` (mandos) forman parte de `Config` y se guardan en disco.
- **Botones de mando por posición o por código.** Un `PadInput` es un botón que `gilrs` sabe situar (`Button::South`, `Start`...) o, en mandos genéricos sin mapeo conocido, el código nativo que envía el mando. `Gamepads::poll` consume los eventos una vez por repintado: actualiza el estado que lee `Gamepads::read` y anota las pulsaciones nuevas, que usa la ventana de ajustes para reasignar un botón.

### Audio

Hay dos hilos con ritmos distintos: el de la interfaz emula un frame cada ~16,7 ms y produce de golpe ~804 pares de muestras; el de la tarjeta de sonido (creado por `cpal`) pide bloques pequeños de forma continua. Los une una cola protegida por un `Mutex`:

```
hilo de la interfaz                              hilo de audio (cpal)
run_frame() ─► audio_samples() ─► push ─► [ cola ] ─► pop ─► × volumen ─► altavoces
                                             │
                          fill_error() ◄─────┘  (ajusta la velocidad de emulación)
```

- **Frecuencia del dispositivo.** Al arrancar se le pide al emulador que genere a la frecuencia del dispositivo (44,1 o 48 kHz normalmente); el core remuestrea desde los 32 768 Hz nativos de la GBA.
- **Estéreo.** La cola guarda pares (izquierda, derecha). Si el dispositivo es mono se mezclan; si tiene más de dos canales, los demás quedan en silencio.
- **Latencia objetivo de 60 ms.** La cola tiene un tope de 150 ms; si se supera se descartan las muestras más antiguas, porque es peor que el sonido se retrase respecto a la imagen.
- **Control dinámico de velocidad.** El reloj de la pantalla y el de la tarjeta de sonido nunca coinciden exactamente, y sin corregirlo la cola acabaría vaciándose (chasquidos) o llenándose (retraso). Según lo llena que esté la cola, la app emula hasta un 0,5 % más rápido o más lento, que no se percibe.
- **Sin chasquidos al vaciarse.** Si la cola se queda sin muestras, la salida baja suavemente a silencio en lugar de cortar en seco.
- **El audio es opcional.** Si no se puede abrir el dispositivo, `GbaApp::audio` es `None` y todo lo demás funciona.

### Configuración persistente

`Config` se serializa con `serde` y se guarda mediante `eframe::Storage` (un archivo RON en la carpeta de datos del usuario; `localStorage` en la web). `eframe` llama a `App::save` periódicamente y al cerrar. Todos los campos tienen valor por defecto, así que una configuración antigua sigue cargando cuando se añaden ajustes nuevos.

### BIOS

`Config::bios_path` guarda la ruta de un volcado propio; `GbaApp::apply_bios` lo lee al arrancar y al cambiarlo, y lo entrega a `Emulator::set_bios`. Si falla (archivo ausente o de otro tamaño) se borra la ruta, se avisa y se sigue con la BIOS incluida: una ruta rota nunca impide arrancar.

### Ritmo de la emulación

El monitor y la consola no van a la misma frecuencia: un monitor puede ir a 60, 75 o 144 Hz y la GBA a 59,73 fps. `FrameClock` acumula el tiempo real entre repintados y devuelve los frames enteros que caben, guardando la fracción sobrante. Así la velocidad del juego es correcta en cualquier monitor.

Si la app se retrasa más de 3 frames (ventana minimizada, tirón del sistema), el retraso se descarta en lugar de emularse de golpe.

### Escalado

`fit_rect` conserva la proporción 240×160 (3:2, píxeles cuadrados) y centra la imagen. Con escalado entero (por defecto) el factor es un múltiplo entero en píxeles físicos, teniendo en cuenta la escala de pantalla del sistema, para que todos los píxeles de la GBA midan lo mismo. La textura usa filtrado `NEAREST` para que no se vea borrosa.

## Tests

| Dónde | Qué comprueba | Comando |
|---|---|---|
| `crates/gba-core/tests` | La API propia, con la ROM de demostración | `cargo test --workspace` |
| `crates/gba-app/src` (tests unitarios) | Controles, cola de audio, escalado, reloj, configuración, biblioteca y guardado | `cargo test --workspace` |
| `vendor/rustboyadvance-ng/core` | CPU ARM/Thumb y memoria con [gba-tests](https://github.com/jsmolka/gba-tests); EEPROM, RTC, planificador | `cargo test --release` dentro de esa carpeta |

La CI ejecuta ambos.

# Roadmap

Cada fila es una rama que se integra en `develop` con su pull request.

## v0.1.0 — en curso

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `chore/vendor-core` | Core de rustboyadvance-ng copiado y parcheado en `vendor/` |
| ✅ | `feature/core-api` | Workspace, `Emulator` de `gba-core` con tests headless, benchmark, CI y documentación base |
| ⬜ | `feature/video-window` | Ventana con la imagen del juego, escalado entero y ~59,73 FPS |
| ⬜ | `feature/rom-loading` | Abrir ROM: menú, arrastrar y soltar, CLI, recientes |
| ⬜ | `feature/input` | Teclado y mandos (10 botones, con L y R) |
| ⬜ | `feature/audio` | Salida de sonido estéreo, volumen y silencio |
| ⬜ | `feature/emulation-controls` | Pausa, reset, avance rápido, save states, partidas guardadas |
| ⬜ | `feature/settings` | Ajustes: teclas, escala, pantalla completa, BIOS propia; persistencia |
| ⬜ | `feature/home-screen` | Pantalla de inicio con biblioteca de juegos |
| ⬜ | `feature/web-wasm` | Versión web con `trunk` |

Las ramas de la app portan el código de [replica-NES](https://github.com/DEIMER-PY/replica-NES) (`nes-app`), adaptado a las diferencias de la GBA descritas en [ARQUITECTURA.md](ARQUITECTURA.md).

## Pendiente de validar

Lo hecho hasta ahora se verificó con la ROM de demostración del proyecto y con `gba-tests`. Falta:

- Juegos comerciales u homebrew reales: imagen, sonido y guardado (SRAM, Flash, EEPROM).
- Sonido: comprobar de oído los canales PSG y DirectSound frente a mGBA.
- El resto de `gba-tests` (`ppu`, `save`, `bios` con BIOS original) y la suite de mGBA.
- Compilación del core a WebAssembly dentro de este workspace.

## Más adelante (GBA)

- Incorporar [mgba-emu/suite](https://github.com/mgba-emu/suite) a los tests.
- Rebobinado y filtros de imagen (corrección de color de la pantalla LCD).
- Visores de depuración: CPU, tiles, paletas, sprites.
- Reloj en tiempo real (RTC) en la web.
- Sustituir progresivamente piezas del core por código propio, empezando por las que se compartirán con la DS.

## Fase 2: Nintendo DS

| Paso | Descripción |
|---|---|
| 1 | Extraer `arm7tdmi` a `crates/` como CPU compartida y validarla de forma aislada (SingleStepTests, FuzzARM). |
| 2 | Extenderla a ARMv5TE (ARM946E-S): `BLX`, `CLZ`, instrucciones DSP, coprocesador CP15, ITCM/DTCM. |
| 3 | `nds-core`: dos CPUs sincronizadas, buses separados, IPC FIFO, bancos de VRAM. |
| 4 | Dos motores 2D (reutilizando el PPU de GBA) y pantalla táctil. |
| 5 | Motor 3D, audio de 16 canales, cartucho y guardado. |

Antes de empezar hay que decidir la licencia: ver el último apartado de [REFERENCIAS.md](REFERENCIAS.md).

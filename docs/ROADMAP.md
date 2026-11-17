# Roadmap

Cada fila es una rama que se integra en `develop` con su pull request.

## v0.1.0 — en curso

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `chore/vendor-core` | Core de rustboyadvance-ng copiado y parcheado en `vendor/` |
| ✅ | `feature/core-api` | Workspace, `Emulator` de `gba-core` con tests headless, benchmark, CI y documentación base |
| ✅ | `feature/desktop-app` | App de escritorio: ventana, carga de ROM, teclado y mandos, sonido estéreo, pausa, reset, avance rápido, save states, partidas, ajustes y pantalla de inicio |
| ⬜ | `feature/web-wasm` | Versión web con `trunk` (el código está portado; falta compilarlo y probarlo) |
| ⬜ | `feature/bios-setting` | Elegir un volcado propio de la BIOS desde los ajustes |
| ⬜ | `chore/release-v0.1.0` | Workflow de ejecutables y publicación |

La app se portó de una vez desde [replica-NES](https://github.com/DEIMER-PY/replica-NES) (`nes-app`), adaptada a las diferencias de la GBA descritas en [ARQUITECTURA.md](ARQUITECTURA.md), en lugar de repetir rama a rama el camino de la NES.

## Pendiente de validar

El core se verificó con la ROM de demostración y con `gba-tests`; la app, abriendo `ppu/hello.gba` de `gba-tests` y comprobando la imagen en la ventana. Falta:

- Juegos comerciales u homebrew reales: imagen, sonido y guardado (SRAM, Flash, EEPROM).
- Sonido: comprobar de oído los canales PSG y DirectSound frente a mGBA.
- El resto de `gba-tests` (`ppu`, `save`, `bios` con BIOS original) y la suite de mGBA.
- Mandos físicos, diálogos de archivo, arrastrar y soltar y pantalla completa en la app.
- Linux y macOS (la CI compila y pasa los tests en Linux, pero la app no se ha ejecutado allí).

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

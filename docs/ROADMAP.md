# Roadmap

Cada fila es una rama que se integra en `develop` con su pull request.

## v0.1.0 — preparada el 2026-11-16

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `chore/vendor-core` | Core de rustboyadvance-ng copiado y parcheado en `vendor/` |
| ✅ | `feature/core-api` | Workspace, `Emulator` de `gba-core` con tests headless, benchmark, CI y documentación base |
| ✅ | `feature/desktop-app` | App de escritorio: ventana, carga de ROM, teclado y mandos, sonido estéreo, pausa, reset, avance rápido, save states, partidas, ajustes y pantalla de inicio |
| ✅ | `feature/web-wasm` | Versión web con `trunk` |
| ✅ | `feature/bios-setting` | Elegir un volcado propio de la BIOS desde los ajustes |
| ✅ | `feature/psg-audio` | Canales de sonido PSG y corrección del resampler del core |
| ✅ | `chore/release-v0.1.0` | Workflow de ejecutables y publicación |
| ✅ | `chore/release-hardening` | Ejecutable de Linux compilado en Ubuntu 22.04 (glibc 2.35) y prueba de arranque de 15 s en la CI |
| ✅ | `fix/bios-calls` | Los juegos se colgaban en la primera llamada a la BIOS en los ejecutables optimizados |
| ✅ | `feature/app-icon` | Icono propio en la ventana, el `.exe` y la web |
| ✅ | `feature/display-options` | Estirar, suavizado, líneas de barrido y giro |
| ✅ | `docs/showcase` | Capturas, pruebas con juegos en Windows y Linux, tecnologías y diagramas |
| ⬜ | `release v0.1.0` | Publicación en `main` con el tag `v0.1.0` |

La app se portó de una vez desde [replica-NES](https://github.com/DEIMER-PY/replica-NES) (`nes-app`), adaptada a las diferencias de la GBA descritas en [ARQUITECTURA.md](ARQUITECTURA.md), en lugar de repetir rama a rama el camino de la NES.

## Pendiente de validar

El core se verificó con la ROM de demostración y con `gba-tests`; la app, abriendo `ppu/hello.gba` de `gba-tests` y comprobando la imagen en la ventana. Falta:

- Guardado en juegos reales (SRAM, Flash, EEPROM). Crash Bandicoot, Super Mario Bros., Need for Speed: Most Wanted y Sonic Advance 3 arrancan y se juegan en Windows y Linux: ver [PRUEBAS.md](PRUEBAS.md).
- Sonido: comprobar de oído frente a mGBA. La frecuencia y la amplitud del tono de la demo se verifican en los tests, pero nadie lo ha escuchado todavía.
- El resto de `gba-tests` (`ppu`, `save`, `bios` con BIOS original) y la suite de mGBA.
- Mandos físicos, diálogos de archivo, arrastrar y soltar y pantalla completa en la app.
- macOS (sin compilar ni probar).

## Más adelante (GBA)

- Filtrar los canales PSG (hoy se muestrean sin suavizado y los tonos muy agudos producen *aliasing*).
- Escrituras anchas no alineadas en la memoria de guardado (requiere que la CPU no alinee la dirección antes del bus).
- Save states de la web en IndexedDB en lugar de `localStorage` (hoy solo caben unos cuatro).
- Publicar la versión web en GitHub Pages (requiere repositorio público o plan de pago).
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

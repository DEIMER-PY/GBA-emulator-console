# Referencias y herramientas

Resultado de la investigación previa al proyecto (noviembre de 2026): qué existe ya, qué se reutiliza y qué queda como consulta. Los datos de actividad y licencia se comprobaron en GitHub y crates.io el 2026-11-16.

## Decisión: sobre qué core se construye

En replica-NES la emulación la aporta una librería (`tetanes-core`) y el proyecto pone la app. Aquí se sigue la misma idea, con una diferencia: el core se **copia dentro del repo** (`vendor/`) en lugar de usarse como dependencia externa.

| Candidato | Licencia | Estado | Veredicto |
|---|---|---|---|
| [rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng) | MIT | 675 ★, activo (ago. 2026), Rust puro, edición 2024 | **Elegido.** CPU en un crate aparte (`arm7tdmi`), PPU, audio DirectSound (los canales PSG no los traía: se añadieron aquí), DMA, timers, SRAM/Flash/EEPROM, RTC, save states, ya compila a WebAssembly y pasa `gba-tests`. No está en crates.io. |
| [`tuiba-core`](https://crates.io/crates/tuiba-core) | MIT | 28 ★, muy activo, en crates.io | Descartado: joven (sept. 2026) y sin audio en su descripción. A vigilar. |
| [`termboy-gba`](https://crates.io/crates/termboy-gba) | MIT | 4 ★, en crates.io | Descartado: muy poco uso, BIOS solo HLE. |
| [`mgba`](https://crates.io/crates/mgba) / `mgba-sys` | MIT + MPL-2.0 | Bindings a libmgba (C) | Descartado: la máxima precisión, pero exige CMake y un compilador de C, complica la versión web y no deja nada reutilizable para la DS. |
| [`armv4t_emu`](https://crates.io/crates/armv4t_emu) | MIT | Solo CPU ARMv4T | Descartado: solo cubre la CPU; habría que escribir todo lo demás. |
| Core propio desde cero | — | — | Aplazado: meses de trabajo antes de ver un juego. Queda como objetivo detrás de la API de `gba-core`. |

Por qué copiarlo en `vendor/` y no depender por git:

1. Hay que modificarlo (partidas sin disco para la web, frecuencia de audio en caliente, quitar un estado global). Ver [`vendor/rustboyadvance-ng/PARCHES.md`](../vendor/rustboyadvance-ng/PARCHES.md).
2. El crate `arm7tdmi` queda en el repo como base de la CPU secundaria de la Nintendo DS.
3. Compilaciones reproducibles sin depender de que el repositorio de origen siga existiendo.

## BIOS

| Recurso | Licencia | Uso |
|---|---|---|
| [Cult-of-GBA/BIOS](https://github.com/Cult-of-GBA/BIOS) | MIT | **Incluida** en `crates/gba-core/bios/`. BIOS de reemplazo escrita desde cero; permite arrancar sin la BIOS de Nintendo. |
| BIOS original | Nintendo | No se distribuye. `Emulator::set_bios` acepta un volcado propio (SHA-256 `fd2547724b505f487e6dcb29ec2ecff3af35a841a77ab2e85fd87350abd36570`) para la máxima compatibilidad. |

## ROMs de prueba

| Recurso | Licencia | Qué valida | Estado aquí |
|---|---|---|---|
| [jsmolka/gba-tests](https://github.com/jsmolka/gba-tests) | MIT | CPU ARM y Thumb, memoria, BIOS, PPU, guardado | **Incluido** en `vendor/…/external/gba-tests`; lo ejecuta la CI. |
| [mgba-emu/suite](https://github.com/mgba-emu/suite) | MIT | Temporización, DMA, timers, E/S, vídeo | Pendiente de incorporar. |
| [SingleStepTests/ARM7TDMI](https://github.com/SingleStepTests/ARM7TDMI) | MIT | Cada instrucción con estado antes/después (JSON) | Experimental. Útil si se escribe una CPU propia. |
| [DenSinH/FuzzARM](https://github.com/DenSinH/FuzzARM) | GPL-3.0 | Instrucciones ARM/Thumb con entradas aleatorias | Solo ejecutar; no copiar al repo (GPL). |
| [destoer/armwrestler-gba](https://github.com/destoer/armwrestler-gba) | — | Instrucciones ARM/Thumb, interactivo | Prueba manual. |
| [NanoBoyAdvance hw-test](https://github.com/nba-emu/hw-test) | — | Detalles finos de hardware | Para la fase de precisión. |
| AGS Aging Cartridge | Nintendo | Cartucho de diagnóstico oficial | No distribuible. |

Tabla comparativa de emuladores frente a estos tests: [GBA Tests en Emulation General Wiki](https://emulation.gametechwiki.com/index.php/GBA_Tests_(Debug)).

## Documentación de hardware

| Recurso | Contenido |
|---|---|
| [GBATEK](https://problemkaputt.de/gbatek.htm) | La referencia técnica de GBA y Nintendo DS. |
| [awesome-gbadev](https://github.com/gbadev-org/awesome-gbadev) | Lista de recursos; sección "Emulator Development". |
| [Tonc](https://gbadev.net/tonc/) | Tutorial de programación de GBA; explica el vídeo y el audio desde el lado del juego. |

## Emuladores de referencia (para comparar comportamiento)

| Emulador | Licencia | Para qué |
|---|---|---|
| [mGBA](https://mgba.io/) | MPL-2.0 | Comparar imagen y sonido; visores de tiles, paletas y mapas; servidor GDB. |
| [NanoBoyAdvance](https://github.com/nba-emu/NanoBoyAdvance) | GPL-3.0 | Referencia de precisión de ciclo (archivado en 2026, sigue siendo la mejor lectura). |
| [SkyEmu](https://github.com/skylersaleh/SkyEmu) | MIT | GBA + DS en un solo proyecto, licencia permisiva. |

> Los proyectos GPL se pueden **leer y ejecutar** como referencia, pero no se copia código suyo: este repo es MIT.

## Librerías de la app (las mismas que replica-NES)

| Crate | Uso |
|---|---|
| `eframe` / `egui` | Ventana, render (`wgpu` en escritorio, WebGL en web) e interfaz. |
| `cpal` | Salida de audio. |
| `gilrs` | Mandos. |
| `rfd` | Diálogos de archivo. |
| `trunk` | Compilar y servir la versión web. |

## Desarrollo de juegos (homebrew) en Rust

No se usan para el emulador, pero sirven para escribir ROMs de prueba propias.

| Crate | Uso |
|---|---|
| [`agb`](https://github.com/agbrs/agb) | Librería completa para hacer juegos de GBA en Rust. |
| [`gba`](https://github.com/rust-console/gba) | Acceso de bajo nivel a los registros de la GBA. |
| [`gba_test`](https://crates.io/crates/gba_test) | Framework de tests que corren dentro de la GBA (o de un emulador). |

## Nintendo DS (siguiente fase)

| Recurso | Licencia | Notas |
|---|---|---|
| [kelpsyberry/dust](https://github.com/kelpsyberry/dust) | GPL-3.0 | Emulador de DS en Rust, escritorio y web, orientado a precisión. La mejor referencia de arquitectura. |
| [Grarak/DSVita](https://github.com/Grarak/DSVita) | GPL-3.0 | Emulador de DS en Rust orientado a velocidad (JIT para ARM32). |
| [`desmume-rs`](https://crates.io/crates/desmume-rs) | GPL-3.0 | Bindings a DeSmuME (C++). |
| [melonDS](https://melonds.kuribo64.net/) | GPL-3.0 | Emulador de referencia para comparar. |
| [BlocksDS](https://github.com/blocksds/sdk) | — | SDK de homebrew para escribir ROMs de prueba. |

**Punto a decidir antes de empezar la DS:** todos los cores de DS en Rust son GPL-3.0. Usar uno obliga a publicar el proyecto de DS bajo GPL-3.0; mantener MIT obliga a escribir el core propio (partiendo del `arm7tdmi` de este repo para la CPU secundaria y extendiéndolo a ARMv5TE para la principal).

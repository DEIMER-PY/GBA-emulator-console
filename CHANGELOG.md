# Changelog

Todos los cambios relevantes del proyecto se registran aquí.
El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

## [0.1.0] - 2026-11-16

Primera versión: emulador de Game Boy Advance para escritorio y web.

### Añadido

- `vendor/rustboyadvance-ng`: core de emulación de GBA (MIT) copiado en el repo, con las ROMs de prueba `gba-tests`. Todos los cambios respecto al original están en `vendor/rustboyadvance-ng/PARCHES.md`.
- `gba-core`: `Emulator` con carga de ROM, emulación por frames, imagen RGBA 240×160, audio estéreo, botones, reinicio, save states y partidas guardadas (SRAM, Flash, EEPROM) como bytes.
- `gba-core`: BIOS libre de Cult-of-GBA incluida, y `set_bios` para usar un volcado propio.
- `gba-core`: ROM de demostración generada en memoria y 14 tests headless.
- `gba-core`: ejemplos `bench` (velocidad de emulación y tamaño de un save state) y `dump_demo` (escribe la demo en un archivo).
- `gba-app`: aplicación portada de replica-NES — ventana con escalado entero, pantalla de inicio con biblioteca y recientes, carga de ROM (menú, arrastrar, línea de comandos), teclado y mandos reasignables con L y R, sonido estéreo, pausa, reinicio, avance rápido, cuatro ranuras de save state, partidas guardadas en disco y ajustes persistentes.
- `gba-app`: pestaña **Consola** en los ajustes para usar un volcado propio de la BIOS original.
- Versión web: la app se compila a WebAssembly con `trunk`.
- CI: formato, lints y tests del código propio (nativo y WebAssembly) y suite de tests del core de terceros.
- CI: workflow `release` que compila los ejecutables de Windows y Linux y los adjunta a la release al subir un tag.
- Documentación: instalación, uso, versión web, arquitectura, referencias evaluadas, guía de desarrollo y roadmap (GBA y Nintendo DS).

### Cambiado respecto al core original

- La partida guardada vive en memoria y viaja en los save states, en lugar de escribirse en un archivo junto a la ROM.
- La frecuencia de salida del audio se puede cambiar con la consola en marcha.
- El desfase de ciclos entre frames es propio de cada consola, no una variable global.

### Corregido

- Core: la memoria de guardado se comporta como un bus de 8 bits (las lecturas de 16 y 32 bits repiten el byte, las escrituras guardan uno solo) y un cartucho sin guardado lee `0xFF`. Detectado al añadir las ROMs de guardado de `gba-tests` a la suite.
- `gba-app`: la interfaz usa siempre el tema oscuro; con el sistema o el navegador en tema claro, los textos de la pantalla de inicio no se leían.

### Limitaciones conocidas

- **Sonido incompleto:** el core solo sintetiza los dos canales DirectSound (audio muestreado, que usan la música y las voces de la mayoría de juegos). Los cuatro canales PSG heredados de Game Boy (dos de onda cuadrada, onda programable y ruido) no suenan.
- Sin probar con juegos comerciales: solo con la ROM de demostración y las ROMs de `gba-tests`.
- Escrituras de 16 y 32 bits no alineadas en la memoria de guardado (no afecta al acceso byte a byte que usan los juegos).
- En la web caben unos cuatro save states en total, y un fallo del core cierra la página.

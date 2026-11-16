# Changelog

Todos los cambios relevantes del proyecto se registran aquí.
El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido

- `vendor/rustboyadvance-ng`: core de emulación de GBA (MIT) copiado en el repo, con parches para guardar partida sin disco, cambiar la frecuencia de audio en marcha y eliminar un estado global. Incluye las ROMs de prueba `gba-tests`.
- `gba-core`: `Emulator` con carga de ROM, emulación por frames, imagen RGBA 240×160, audio estéreo, botones, reinicio, save states y partidas guardadas (SRAM, Flash, EEPROM) como bytes.
- `gba-core`: BIOS libre de Cult-of-GBA incluida, y `set_bios` para usar un volcado propio.
- `gba-core`: ROM de demostración generada en memoria y 14 tests headless.
- `gba-core`: ejemplo `bench` para medir la velocidad de emulación.
- CI: formato, lints y tests del código propio, y tests del core de terceros.
- Documentación: arquitectura, referencias y herramientas, guía de desarrollo y roadmap (GBA y Nintendo DS).

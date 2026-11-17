# GBA-emulator-console

Emulador de **Game Boy Advance** escrito en Rust, para escritorio y web. Es el segundo paso de una serie
(NES → GBA → Nintendo DS): continúa el trabajo de [replica-NES](https://github.com/DEIMER-PY/replica-NES)
y prepara el terreno para la DS, que reutiliza la misma CPU (ARM7TDMI).

> **Estado:** la app funciona en escritorio y en el navegador (imagen, sonido, controles, save states y
> partidas). Falta probarla con juegos comerciales. Ver [docs/ROADMAP.md](docs/ROADMAP.md).

## Qué hay

- **`crates/gba-app`** — la aplicación: ventana, pantalla de inicio con biblioteca, teclado y mandos
  reasignables, sonido, pausa, avance rápido, save states y ajustes. Portada de replica-NES.
- **`crates/gba-core`** — API de emulación: cargar una ROM, emular frame a frame, imagen 240×160, audio
  estéreo, botones, save states y partidas guardadas. Sin acceso a disco, lista para escritorio y web.
- **`vendor/rustboyadvance-ng`** — el core que hace el trabajo (CPU ARM7TDMI, PPU, APU, DMA, timers,
  cartucho), tomado de [rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng) y parcheado.
- **BIOS libre incluida** ([Cult-of-GBA](https://github.com/Cult-of-GBA/BIOS)): no hace falta la BIOS de Nintendo.

## Usarlo

Requiere [Rust](https://rustup.rs) estable (≥ 1.95).

```sh
cargo run -p gba-app --release                      # abre la app
cargo run -p gba-app --release -- ruta/juego.gba    # ...con un juego
cargo test --workspace                              # tests
cargo run -p gba-core --release --example bench     # velocidad del core (~10x la real)
```

Controles por defecto: flechas, `X` = A, `Z` = B, `A` = L, `S` = R, `Enter` = Start, `Retroceso` = Select.
Todo lo demás está en [docs/USO.md](docs/USO.md).

## Documentación

| Documento | Contenido |
|---|---|
| [docs/INSTALACION.md](docs/INSTALACION.md) | Ejecutable o compilación, requisitos y dónde conseguir ROMs |
| [docs/USO.md](docs/USO.md) | Cómo usar la app: cargar juegos, controles, guardado, sonido e imagen |
| [docs/WEB.md](docs/WEB.md) | Versión web: compilar con `trunk`, diferencias con el escritorio |
| [docs/ARQUITECTURA.md](docs/ARQUITECTURA.md) | Capas, API de `gba-core`, funcionamiento de la app, decisiones de diseño |
| [docs/REFERENCIAS.md](docs/REFERENCIAS.md) | Cores, tests, documentación y herramientas evaluados; por qué se eligió cada uno |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Qué está hecho, qué falta y el plan hacia la Nintendo DS |
| [docs/GUIA-DESARROLLO.md](docs/GUIA-DESARROLLO.md) | Entorno, comandos, flujo de ramas y convención de commits |
| [CHANGELOG.md](CHANGELOG.md) | Cambios por versión |

## Licencia

[MIT](LICENSE). El código de terceros conserva su licencia (también MIT): ver
[`vendor/rustboyadvance-ng`](vendor/rustboyadvance-ng/PARCHES.md) y
[`crates/gba-core/bios`](crates/gba-core/bios/LICENSE-cult-of-gba).

No se incluyen ni se distribuyen ROMs comerciales ni la BIOS de Nintendo.

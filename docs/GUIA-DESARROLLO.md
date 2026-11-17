# Guía de desarrollo

## Entorno

| Herramienta | Para qué | Instalación |
|---|---|---|
| Rust estable (≥ 1.95) | Compilar | `winget install Rustlang.Rustup` o <https://rustup.rs> |
| C++ Build Tools (Windows) | Enlazador MSVC | Visual Studio Installer → "Desarrollo para el escritorio con C++" |
| `gh` | Pull requests desde la terminal | `winget install GitHub.cli` |

En Linux hacen falta además las librerías de sistema que usa la CI (ver `.github/workflows/ci.yml`).

Si `cargo` no se reconoce en la terminal, añade `%USERPROFILE%\.cargoin` al `PATH`.

## Comandos habituales

```sh
cargo run -p gba-app                 # ejecutar la app (desarrollo)
cargo run -p gba-app --release       # ejecutar optimizada
cargo build -p gba-app --release     # generar el ejecutable: target/release/gba-app(.exe)
cargo test --workspace               # tests del código propio
cargo fmt --all                      # formatear
cargo clippy --workspace --all-targets -- -D warnings   # lints
cargo run -p gba-core --release --example bench          # medir la velocidad del core
cargo run -p gba-core --release --example bench -- juego.gba   # ...con una ROM propia
```

Tests del core de terceros (CPU y memoria con `gba-tests`):

```sh
cd vendor/rustboyadvance-ng/core
cargo test --release
```

El core de emulación se compila optimizado incluso en modo desarrollo (ver `[profile.dev.package.*]` en `Cargo.toml`), porque sin optimizar no alcanza la velocidad real.

## Código de terceros (`vendor/`)

- No forma parte del workspace y `cargo fmt` no lo toca (`vendor/rustboyadvance-ng/rustfmt.toml`).
- Cada cambio se marca en el código con `PARCHE gba-emulator-console` y se anota en `vendor/rustboyadvance-ng/PARCHES.md`.
- Cambios mínimos: lo que se pueda resolver en `gba-core` se resuelve allí.

## Flujo de ramas

```
main      ← solo releases, siempre estable y etiquetada (v0.1.0, ...)
develop   ← rama por defecto; integración de todo el trabajo
feature/* ← una rama por funcionalidad, sale de develop y vuelve a develop
```

1. `git checkout develop && git pull`
2. `git checkout -b feature/<nombre>`
3. Commits pequeños y atómicos.
4. `git push -u origin feature/<nombre>` y `gh pr create --base develop`.
5. Con la CI en verde, merge a `develop` (merge commit, se conserva el historial de la feature).
6. Para publicar: PR de `develop` a `main` y tag `vX.Y.Z`.

Los PR los fusiona el propietario del repositorio.

Prefijos de rama: `feature/`, `fix/`, `docs/`, `chore/`, `refactor/`.

## Convención de commits

Formato (en inglés, imperativo, sin punto final, máximo ~72 caracteres):

```
<emoji> <tipo>(<ámbito>): <descripción>
```

| Emoji | Tipo | Uso |
|---|---|---|
| ✨ | `feat` | Funcionalidad nueva |
| 🐛 | `fix` | Corrección de un error |
| 📝 | `docs` | Solo documentación |
| ♻️ | `refactor` | Cambio interno sin alterar comportamiento |
| ⚡ | `perf` | Mejora de rendimiento |
| ✅ | `test` | Añadir o corregir tests |
| 🔧 | `chore` | Configuración, dependencias, mantenimiento |
| 👷 | `ci` | Integración continua |
| 💄 | `style` | Interfaz o formato, sin cambio de lógica |
| 🎉 | `chore` | Commit inicial |
| 🔖 | `chore(release)` | Publicación de versión |
| 📦 | `chore(vendor)` | Incorporar o actualizar código de terceros |

Ámbitos habituales: `core`, `vendor`, `app`, `video`, `audio`, `input`, `ui`, `config`, `web`, `docs`, `ci`.

Ejemplos:

```
✨ feat(core): add Emulator wrapper around rustboyadvance-core
🐛 fix(audio): avoid buffer underrun when window is unfocused
📝 docs(architecture): describe the frame loop
```

## Documentación

Cada PR actualiza la documentación que toca: `docs/USO.md` si cambia algo visible para el usuario, `docs/ARQUITECTURA.md` si cambia la estructura, `docs/ROADMAP.md` al completar una funcionalidad y `CHANGELOG.md` siempre.

## ROMs para probar

No se versionan ROMs. Para pruebas manuales usa homebrew de libre distribución o volcados de tus propios cartuchos, guardados fuera del repositorio (`*.gba` y `*.sav` están ignorados). Los tests automáticos generan en memoria una ROM mínima propia; las únicas ROMs versionadas son las de prueba con licencia MIT de `vendor/rustboyadvance-ng/external/gba-tests`.

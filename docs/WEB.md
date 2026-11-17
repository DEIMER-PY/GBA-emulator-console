# Versión web

El mismo código de `gba-app` se compila a WebAssembly y se ejecuta en el navegador, dibujando en un `<canvas>`. No hay servidor: el resultado es un sitio estático (HTML, JavaScript y un archivo `.wasm`).

## Requisitos

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

[`trunk`](https://trunkrs.dev) compila el binario a WebAssembly, genera el JavaScript que lo carga y sirve la página.

## Desarrollo

```sh
cd crates/gba-app
trunk serve
```

Abre <http://127.0.0.1:8080>. Al guardar un archivo, `trunk` recompila y recarga la página.

## Compilar para publicar

```sh
cd crates/gba-app
trunk build --release
```

El sitio queda en `crates/gba-app/dist/`. Se puede publicar en cualquier alojamiento de archivos estáticos (GitHub Pages, Cloudflare Pages, Netlify...). Si no se sirve desde la raíz del dominio, hay que indicarlo: `trunk build --release --public-url /GBA-emulator-console/`.

> GitHub Pages en un repositorio **privado** requiere un plan de pago. Con el repositorio público es gratuito.

## Diferencias con la versión de escritorio

| | Escritorio | Web |
|---|---|---|
| Render | `wgpu` (DirectX 12, Vulkan, Metal) | WebGL (`glow`) |
| Abrir ROM | Diálogo del sistema, arrastrar, línea de comandos | Selector del navegador, arrastrar |
| ROMs recientes | Sí | No: el navegador no da acceso a rutas de disco |
| Partidas y estados | Archivos en la carpeta de datos | `localStorage` del navegador |
| Configuración | `app.ron` | `localStorage` del navegador |
| Sonido | Desde el arranque | Tras la primera tecla o clic |
| Pantalla completa | `F11` desde la app | La del navegador (`F11`) |
| Salir | Menú **Archivo → Salir** | Cerrar la pestaña |

Notas:

- **El sonido empieza con la primera interacción.** Los navegadores no dejan que una página reproduzca audio hasta que el usuario pulsa una tecla o hace clic. La app abre la salida de sonido en ese momento.
- **Los datos guardados son de ese navegador.** `localStorage` es propio de cada navegador y sitio, tiene un límite de unos 5 MB y se pierde si se borran los datos de navegación. Un save state de GBA ocupa unos 800 KB (algo más de 1 MB en base64), así que **caben unos cuatro estados en total** entre todos los juegos; al llenarse, guardar da un error y hay que sobrescribir una ranura usada. Las partidas del cartucho ocupan como mucho 128 KB.
- **Un fallo del core cierra la página.** En escritorio, si el core se detiene ante hardware no soportado, la app muestra un error; en WebAssembly no se puede capturar y hay que recargar la página.
- **Sin reloj en tiempo real.** Los juegos con reloj (RTC) no están probados en la web.
- **Las ROMs no salen del equipo.** El archivo se lee en la propia página; no se sube a ningún servidor.

## Cómo está hecho

El código específico de cada plataforma se elige en compilación con `#[cfg(target_arch = "wasm32")]`:

| Archivo | Papel |
|---|---|
| `src/main.rs` | Dos puntos de entrada: `native::run` (ventana) y `wasm::run` (`eframe::WebRunner` sobre el `<canvas>`) |
| `src/web.rs` | `RomInbox`: lectura asíncrona de ROMs elegidas o soltadas |
| `src/storage_web.rs` | `SaveStore` sobre `localStorage`; sustituye a `storage.rs` |
| `index.html` | Página con el `<canvas>` y la instrucción de compilación para `trunk` |
| `Trunk.toml` | Configuración de `trunk` |

### Lectura asíncrona de ROMs

En escritorio, el diálogo de archivos bloquea hasta que el usuario elige. En el navegador eso no es posible: todo lo que espera al usuario o al disco es asíncrono. Por eso la versión web usa un buzón:

```
menú o arrastre ──► RomInbox::pick_file / read_dropped ──► tarea asíncrona (spawn_local)
                                                                   │ lee los bytes
GbaApp::receive_roms (cada repintado) ◄── canal mpsc ◄─────────────┘
        └──► GbaApp::load_rom
```

El resto de la app no distingue de dónde viene la ROM: `load_rom` recibe un `RomFile` en ambos casos.

### Dependencias por plataforma

`crates/gba-app/Cargo.toml` separa las dependencias con tablas `[target.'cfg(...)'.dependencies]`. En la web, `eframe` usa el backend `glow` en lugar de `wgpu`y `cpal` activa `wasm-bindgen`.

`gba-core` no necesita ningún cambio: no accede a disco ni a nada del sistema. El core de terceros se parcheó para guardar la partida en memoria en lugar de en un archivo (ver `vendor/rustboyadvance-ng/PARCHES.md`).

## Verificado

Compilada con `trunk build` y abierta en Chromium: carga la pantalla de inicio, ejecuta la ROM de demostración y responde al teclado. Sin probar: sonido, arrastrar archivos, guardado en `localStorage` y otros navegadores.

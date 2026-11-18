# Instalación

Hay dos formas de tener el emulador: descargar el **ejecutable** ya compilado (Windows y Linux) o **compilarlo** desde el código fuente (Windows, Linux y macOS). También existe una [versión web](WEB.md) que funciona en el navegador.

| Sistema | Ejecutable descargable | Compilar desde el código |
|---|---|---|
| Windows 10 / 11 (64 bits) | Sí | Sí |
| Linux (64 bits) | Sí | Sí |
| macOS | No | Sí |

> **Estado de las pruebas.** La app se ha ejecutado y probado en Windows 11. En Linux la integración continua la compila y pasa los tests, pero no se ha usado con ventana. En macOS no se ha compilado ni probado: los pasos son los estándar de Rust y de las librerías que usa el proyecto.

## Requisitos mínimos

Orientativos: salen de las librerías que usa la app, no de probar en equipos antiguos. Emular una GBA exige más que una NES (CPU de 32 bits a 16,78 MHz), pero sigue siendo poco para un equipo actual: en el equipo de desarrollo el core emula unas diez veces más rápido que la consola real.

| | Mínimo |
|---|---|
| Sistema | Windows 10 u 11, Linux con X11 o Wayland, o macOS 11 o posterior; siempre de 64 bits |
| Procesador | Cualquiera de dos núcleos de los últimos 10 años |
| Gráfica | Integrada, compatible con DirectX 12, Vulkan, Metal u OpenGL 3.3 |
| Memoria | 4 GB en el equipo |
| Disco | 30 MB para el ejecutable; unos 5 GB si se compila desde el código |
| Sonido | Opcional: sin dispositivo de audio la app funciona en silencio |
| Mando | Opcional: USB o Bluetooth; se puede jugar solo con teclado |

Para medir la velocidad en tu equipo: `cargo run -p gba-core --release --example bench`.

## Opción A: ejecutable

1. Abre la página de [Releases](https://github.com/DEIMER-PY/GBA-emulator-console/releases) del repositorio. El repositorio es privado: hay que iniciar sesión en GitHub con una cuenta que tenga acceso.
2. Descarga el archivo de tu sistema de la versión más reciente.
3. Ábrelo. No hay instalador: es un único archivo que puedes guardar donde quieras.

### Windows

1. Descarga `GBA-emulator-console-vX.Y.Z-windows-x64.exe`.
2. Haz doble clic. La primera vez, Windows puede mostrar el aviso «Windows protegió su PC», porque el ejecutable no está firmado: pulsa **Más información → Ejecutar de todas formas**.
3. Para abrir un juego directamente, arrastra el archivo `.gba` sobre el ejecutable o sobre la ventana.

Si la ventana no aparece o se cierra al abrir, consulta [Solución de problemas](USO.md#solución-de-problemas).

### Linux

1. Descarga `GBA-emulator-console-vX.Y.Z-linux-x64`.
2. Instala las librerías que necesita. Suelen venir ya en un escritorio normal:

   ```sh
   # Debian, Ubuntu, Linux Mint
   sudo apt install libasound2 libudev1 libxkbcommon0 libgtk-3-0

   # Fedora
   sudo dnf install alsa-lib systemd-libs libxkbcommon gtk3

   # Arch, Manjaro
   sudo pacman -S alsa-lib systemd-libs libxkbcommon gtk3
   ```

3. Dale permiso de ejecución y ábrelo:

   ```sh
   chmod +x GBA-emulator-console-vX.Y.Z-linux-x64
   ./GBA-emulator-console-vX.Y.Z-linux-x64            # pantalla de inicio
   ./GBA-emulator-console-vX.Y.Z-linux-x64 juego.gba  # con un juego
   ```

El ejecutable se compila en Ubuntu reciente: en distribuciones antiguas puede fallar por la versión de `glibc`. En ese caso, compílalo (opción B).

## Opción B: compilar desde el código

Los pasos son los mismos en los tres sistemas; solo cambian las herramientas previas.

### 1. Herramientas previas

**Windows**

```powershell
winget install Rustlang.Rustup
winget install Git.Git
```

Además hace falta el enlazador de C++: en el [instalador de Visual Studio](https://visualstudio.microsoft.com/es/downloads/) (vale «Build Tools for Visual Studio»), marca **Desarrollo para el escritorio con C++**, que incluye el SDK de Windows. Abre una terminal nueva al terminar.

**Linux**

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Debian, Ubuntu, Linux Mint
sudo apt install git build-essential pkg-config libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev libgtk-3-dev

# Fedora
sudo dnf install git gcc pkgconf-pkg-config alsa-lib-devel systemd-devel libxkbcommon-devel wayland-devel gtk3-devel

# Arch, Manjaro
sudo pacman -S git base-devel alsa-lib systemd-libs libxkbcommon wayland gtk3
```

**macOS**

```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Descargar y compilar

```sh
git clone https://github.com/DEIMER-PY/GBA-emulator-console.git
cd GBA-emulator-console
cargo build -p gba-app --release
```

La primera compilación descarga las dependencias y tarda entre 10 y 20 minutos según el equipo. Hace falta Rust 1.95 o posterior; `rustup update` lo actualiza.

### 3. Ejecutar

El ejecutable queda en `target/release/gba-app` (`gba-app.exe` en Windows). Se puede copiar a cualquier carpeta.

```sh
./target/release/gba-app              # pantalla de inicio
./target/release/gba-app juego.gba    # con un juego
```

Para actualizar más adelante: `git pull` y de nuevo `cargo build -p gba-app --release`.

## Comprobar que funciona

Al abrir la app aparece la pantalla de inicio. Pulsa **Probar la demo**: carga una ROM de demostración propia y la pantalla se ve azul. Al mantener la tecla `X` (botón A) pasa a rojo y suena un tono. Con eso quedan comprobados el vídeo, los controles y el sonido. El resto de teclas y funciones está en [USO.md](USO.md).

## Si no arranca

- **En Linux con Wayland la ventana no se abre** (error de conexión con el compositor): fuerza X11 con `env -u WAYLAND_DISPLAY ./GBA-emulator-console-*-linux-x64`.
- **Se cierra al abrirla o no dibuja nada:** prueba otro motor gráfico con `WGPU_BACKEND=gl` (Linux) o `set WGPU_BACKEND=dx12` (Windows) antes de abrirla.

## Dónde conseguir ROMs

El emulador abre archivos `.gba` (también dentro de un `.zip`). **El proyecto no incluye ningún juego**, y este repositorio no enlaza a sitios de descarga de juegos comerciales: esos juegos tienen derechos de autor y descargarlos de internet es ilegal en la mayoría de países, aunque se tenga el cartucho original.

Formas legítimas de conseguir juegos:

- **Homebrew: juegos nuevos hechos para la GBA.** Hay una comunidad activa que sigue publicando juegos, muchos gratuitos, que se descargan como archivo `.gba`:
  - [itch.io, juegos etiquetados Game Boy Advance](https://itch.io/games/tag-game-boy-advance): filtra por los que ofrecen descarga de la ROM.
  - [gbadev.net](https://gbadev.net/): la comunidad de desarrollo de GBA, que organiza la GBA Jam anual.
- **Volcados de tus propios cartuchos.** Con un lector de cartuchos (*dumper*) conectado por USB se copia el contenido de un cartucho propio a un archivo `.gba`. La legalidad de la copia privada depende de cada país.
- **ROMs de prueba.** El repositorio incluye las de [gba-tests](https://github.com/jsmolka/gba-tests) (MIT) en `vendor/rustboyadvance-ng/external/gba-tests`; por ejemplo `ppu/hello.gba`.
- **La demo del proyecto.** `cargo run -p gba-core --example dump_demo -- demo.gba` la escribe como archivo, para probar la carga desde disco.

Reúne tus ROMs en una carpeta y elígela con **Carpeta de juegos…** en la pantalla de inicio: aparecerán en la biblioteca. Guárdalas fuera de la carpeta del repositorio, o ten en cuenta que los `.gba` están en `.gitignore` para que no se suban por error.

La BIOS de Nintendo no hace falta: ver [USO.md](USO.md#roms-y-bios).

## Dónde guarda sus datos la app

La configuración, las partidas del cartucho y los estados guardados no se escriben junto al ejecutable ni junto a la ROM:

| Sistema | Carpeta |
|---|---|
| Windows | `%APPDATA%\GBA-emulator-console\data` |
| Linux | `~/.local/share/gba-emulator-console` |
| macOS | `~/Library/Application Support/GBA-emulator-console` |

Para desinstalar, borra el ejecutable y esa carpeta.

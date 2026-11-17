# Uso

## Ejecutar

Con el ejecutable (`gba-app.exe` en Windows): doble clic. No necesita instalación ni terminal.

Desde el código fuente:

```sh
cargo run -p gba-app --release
```

## Pantalla de inicio

Al abrir la app sin juego aparece la pantalla de inicio, con todo lo necesario para empezar:

| Elemento | Qué hace |
|---|---|
| **Abrir ROM…** | Abre el diálogo del sistema para elegir un archivo `.gba` |
| **Carpeta de juegos…** | Elige la carpeta donde guardas tus juegos; se recuerda entre sesiones |
| **Probar la demo** | Carga la ROM de demostración incluida en el proyecto |
| **Biblioteca** | Lista los archivos `.gba` de la carpeta de juegos y de sus subcarpetas (hasta 3 niveles), por orden alfabético. Un clic abre el juego |
| **Actualizar** | Vuelve a leer la carpeta, por si has añadido juegos con la app abierta |
| **Recientes** | Las últimas 8 ROMs abiertas, también con un clic |

Además, se puede arrastrar un archivo `.gba` a la ventana en cualquier momento: mientras se arrastra, la ventana indica «Suelta el archivo para abrirlo».

**Archivo → Cerrar ROM e ir al inicio** vuelve a esta pantalla.

No hay pantalla de carga porque no hay espera: un juego de GBA ocupa entre 1 y 32 MB y se carga en unos milisegundos.

La **ROM de demostración** es una pantalla azul que pasa a rojo mientras se mantiene el botón A (`X`). Confirma que la emulación, el vídeo y los controles funcionan. No tiene sonido.

## Cargar una ROM

| Forma | Cómo |
|---|---|
| Pantalla de inicio | Botón **Abrir ROM…** o un clic en un juego de la biblioteca o de recientes |
| Menú | **Archivo → Abrir ROM…** (`Ctrl+O`) |
| Arrastrar y soltar | Suelta un archivo `.gba` sobre la ventana |
| Línea de comandos | `cargo run -p gba-app --release -- ruta/al/juego.gba` |
| Recientes | **Archivo → Recientes** guarda las últimas 8 ROMs abiertas |

El título de la ventana muestra el nombre de la ROM cargada. **Archivo → Cerrar ROM e ir al inicio** extrae el cartucho y **Archivo → Cargar demo** abre la demostración.

Si el archivo no es una ROM válida, aparece una ventana de error con el motivo. Si un juego usa hardware que el core no soporta, la emulación se detiene con un aviso en lugar de cerrar la app.

Para generar la demo como archivo y probar la carga desde disco:

```sh
cargo run -p gba-core --example dump_demo -- demo.gba
```

## Controles

### Teclado

| Botón GBA | Tecla |
|---|---|
| Arriba / Abajo / Izquierda / Derecha | Flechas |
| A | `X` |
| B | `Z` |
| L | `A` |
| R | `S` |
| Select | `Retroceso` |
| Start | `Enter` |

Son las teclas por defecto. Se cambian en **Archivo → Ajustes… → Controles** (`Ctrl+,`): se hace clic en la tecla de la tabla y se pulsa la nueva. `Escape` cancela.

- Si la tecla nueva ya estaba en otro botón, los dos botones intercambian sus teclas.
- Las teclas que usa la app como atajo (`P`, `M`, `F`, `F5`, `F8`, `F11`, `1` a `4`) no se pueden asignar.
- **Restaurar teclas por defecto** vuelve a la tabla de arriba.

El teclado no llega al juego mientras hay un menú abierto o se mantiene `Ctrl` o `Alt`, para que los atajos de la app no muevan al personaje.

### Mandos

Los mandos USB o Bluetooth se detectan solos, también si se conectan con la app abierta. La GBA es de un solo jugador: cualquier mando conectado la controla, y teclado y mando funcionan a la vez.

| Botón GBA | Mando |
|---|---|
| Dirección | Cruceta o stick izquierdo |
| A | Botón inferior (A en Xbox, ✕ en PlayStation) |
| B | Botón izquierdo (X en Xbox, □ en PlayStation) |
| L / R | Gatillos superiores (L1 / R1) |
| Select / Start | Select / Start |

Son los botones por defecto. Se cambian en **Archivo → Ajustes… → Controles**, en la tabla **Mandos**: se hace clic en el botón de la tabla y se pulsa el nuevo en el mando. `Escape` cancela.

- La tabla solo se puede editar con un mando conectado.
- Si el botón nuevo ya estaba asignado, los dos intercambian su sitio.
- El stick izquierdo siempre funciona como dirección, además de los botones asignados.
- **Restaurar botones por defecto** vuelve a la tabla de arriba.

En los **mandos genéricos** el sistema a veces no sabe qué botón es cada uno, y pueden salir cambiados de sitio. Se arregla reasignándolos; los botones sin posición conocida aparecen en la tabla como `Código …`.

Con la ROM de demostración se puede comprobar: al mantener el botón A la pantalla pasa de azul a rojo.

## Emulación

Todo está en el menú **Emulación** y tiene atajo:

| Acción | Atajo | Qué hace |
|---|---|---|
| Pausar / Reanudar | `P` | Detiene la consola; se indica con **PAUSA** en pantalla |
| Reiniciar | `Ctrl+R` | Reinicia la CPU y vuelve a arrancar el juego conservando la memoria |
| Apagar y encender | `Ctrl+Shift+R` | Reinicio completo: borra la memoria (no la partida guardada) |
| Avance rápido | mantener `F` | Emula a ×4, sin sonido, mientras se mantiene la tecla |
| Guardar estado | `F5` | Guarda el instante exacto del juego en la ranura seleccionada |
| Cargar estado | `F8` | Vuelve al instante guardado en la ranura seleccionada |
| Elegir ranura | `1` a `4` | Cuatro ranuras independientes por juego |

### Partidas y estados guardados

Hay dos formas de guardar, y son independientes:

- **Partida del juego.** Los juegos que guardaban en el cartucho (SRAM, Flash o EEPROM) lo hacen igual aquí. La app escribe esa memoria en disco sola: cada 30 segundos, al cambiar o cerrar la ROM y al salir.
- **Estados (save states).** Una foto completa de la consola que se guarda y se carga cuando el usuario quiere, en cualquier juego. El estado incluye la partida del cartucho de ese instante: al cargarlo, la partida vuelve también a ese momento.

Ambos se guardan en la carpeta de datos de la app, no junto a la ROM. En Windows:

```
%APPDATA%\GBA-emulator-console\data\saves\<juego>.sav
%APPDATA%\GBA-emulator-console\data\states\<juego>.<ranura>.state
```

El nombre incluye una huella del contenido de la ROM, así que dos versiones distintas del mismo juego no se pisan la partida.

## Sonido

El sonido sale por el dispositivo de audio por defecto del sistema. En el menú **Audio**:

- **Silenciar** (tecla `M`).
- **Volumen**, de 0 a 100 %.

Ambos ajustes se recuerdan. Si no hay ningún dispositivo de audio, la app funciona igual pero en silencio, y el menú **Audio** indica el motivo.

El sonido es estéreo. Si el dispositivo de salida es mono, los dos canales se mezclan.

## Imagen

La imagen se centra y se escala al redimensionar la ventana. Las opciones están en el menú **Ver** y en **Ajustes… → Vídeo**:

| Opción | Qué hace |
|---|---|
| Escalado entero | Escala solo ×1, ×2, ×3..., para que todos los píxeles midan lo mismo. Activado por defecto |
| Pantalla completa | `F11` |

## Configuración

La lista de recientes, las teclas, los botones de mando y las opciones de imagen y sonido se guardan automáticamente y se recuperan al abrir la app. En Windows el archivo está en `%APPDATA%\GBA-emulator-console\data\app.ron`.

## Solución de problemas

**La app se cierra al abrir o no muestra ventana.** Suele ser el driver de la gráfica. En Windows la app usa DirectX 12 por defecto (Vulkan falla con algunos drivers de gráficas integradas Intel). Se puede forzar otro backend con la variable de entorno `WGPU_BACKEND`:

```powershell
$env:WGPU_BACKEND = "gl"      # o "dx12", "vulkan"
cargo run -p gba-app --release
```

## ROMs y BIOS

El emulador abre archivos `.gba` (también dentro de un `.zip`). El proyecto no incluye juegos: usa volcados de tus propios cartuchos o juegos homebrew de libre distribución.

No hace falta la BIOS de Nintendo: la app incluye una BIOS libre ([Cult-of-GBA](https://github.com/Cult-of-GBA/BIOS)), suficiente para la mayoría de juegos.

Si un juego falla con ella, se puede usar un volcado de la BIOS original de tu propia consola (un archivo de 16 KiB) en **Archivo → Ajustes… → Consola → Elegir BIOS…**. La ruta se recuerda, y el cambio se aplica al cargar un juego o al apagar y encender (`Ctrl+Shift+R`). **Usar la incluida** vuelve a la BIOS libre. Si el archivo deja de existir o no tiene el tamaño de una BIOS, la app avisa y usa la incluida. En la versión web solo está disponible la BIOS incluida.

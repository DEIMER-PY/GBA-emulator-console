# Pruebas con juegos

Registro de las pruebas manuales con juegos comerciales de la Game Boy Advance. Complementan los tests
automáticos (`cargo test --workspace`), que usan la ROM de demostración del proyecto, y la prueba de la CI,
que abre cada ejecutable publicado durante 15 segundos antes de subirlo a la release.

Las ROMs son volcados propios y **no están en el repositorio**.

## Resultado

| Juego | Hasta dónde se probó | Windows | Linux |
|---|---|:---:|:---:|
| [Crash Bandicoot: The Huge Adventure](capturas/crash.png) | Intro, mapa de niveles y primer nivel jugando (en Linux, hasta el menú de partidas) | ✅ | ✅ |
| [Classic NES Series: Super Mario Bros.](capturas/smb.png) | Nivel 1-1 jugando | ✅ | ✅ |
| [Need for Speed: Most Wanted](capturas/nfs.png) | Selección de idioma, título y creación de perfil | ✅ | — |
| [Sonic Advance 3](capturas/sonic3.png) | Arranque y registro de nombre | ✅ | — |

✅ funciona hasta ese punto · — no probado en ese sistema

## Cómo se probó

- **Windows 11:** el ejecutable compilado por la CI (`release.yml`), con la ROM pasada por línea de comandos.
  Un script de PowerShell pulsa teclas con `keybd_event` y captura la ventana.
- **Linux:** el mismo binario de Linux de la CI, en Ubuntu 24.04 (WSL2) con un servidor X virtual (`Xvfb`).
  Las teclas se envían con `xdotool` y las capturas se hacen con ImageMagick (`import`).
- Los mandos no se probaron con hardware: solo teclado.

## Capturas

| <img src="capturas/crash.png" alt="Crash Bandicoot" width="220"><br><sub>Crash Bandicoot</sub> | <img src="capturas/smb.png" alt="Super Mario Bros." width="220"><br><sub>Super Mario Bros.</sub> | <img src="capturas/nfs.png" alt="Need for Speed: Most Wanted" width="220"><br><sub>Need for Speed: Most Wanted</sub> |
|:---:|:---:|:---:|
| <img src="capturas/crash-mapa.png" alt="Crash — mapa de niveles" width="220"><br><sub>Crash — mapa de niveles</sub> | <img src="capturas/nfs-perfil.png" alt="NFS — crear perfil" width="220"><br><sub>NFS — crear perfil</sub> | <img src="capturas/opciones-estirado.png" alt="Estirada y suavizada" width="220"><br><sub>Estirada y suavizada</sub> |


## Limitaciones conocidas

- Need for Speed: Most Wanted y Sonic Advance 3 se probaron hasta sus menús (perfil y registro de nombre); no se llegó a una carrera ni a un nivel.
- Los mandos están implementados (`gilrs`, con reasignación en **Ajustes → Controles**) pero no se han probado con un mando físico.
- La BIOS libre no muestra la animación de arranque de Nintendo: los juegos empiezan directamente.

## Fallo encontrado y corregido

Las primeras pruebas con los ejecutables publicados fallaron con **los cuatro juegos**: se colgaban en la
primera llamada a la BIOS con «error de emulación». En modo debug funcionaban.

La protección de lectura de la BIOS consultaba el contador de programa de la CPU a través de un puntero
crudo mientras la CPU tenía un `&mut self` activo. Con LTO completo el compilador reordenaba la lectura y
el bus veía un PC antiguo, así que la CPU ejecutaba la instrucción equivocada en el vector de la SWI. La
corrección (en `vendor/rustboyadvance-ng`, ver `PARCHES.md`) hace que la CPU publique su estado de búsqueda
en el bus de forma explícita. Tres tests (`swi`) lo cubren y la CI los ejecuta en modo release.

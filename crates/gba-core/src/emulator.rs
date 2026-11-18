//! [`Emulator`]: la consola emulada.

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use rustboyadvance_core::keypad::{KEYINPUT_ALL_RELEASED, Keys};
use rustboyadvance_core::prelude::{AudioInterface, GameBoyAdvance, GamepakBuilder};

use crate::{BIOS_SIZE, DEFAULT_SAMPLE_RATE, FRAME_BYTES};

/// BIOS libre de [Cult-of-GBA](https://github.com/Cult-of-GBA/BIOS) (MIT), incluida
/// para que el emulador funcione sin la BIOS original de Nintendo.
const OPEN_BIOS: &[u8; BIOS_SIZE] = include_bytes!("../bios/cult_of_gba_bios.bin");

/// Errores de la emulación.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// La ROM no es válida.
    #[error("no se pudo cargar la ROM: {0}")]
    LoadRom(String),
    /// La BIOS no tiene el tamaño de una BIOS de GBA.
    #[error("la BIOS debe ocupar {BIOS_SIZE} bytes, y ocupa {0}")]
    InvalidBios(usize),
    /// Se pidió una operación que necesita una ROM cargada.
    #[error("no hay ninguna ROM cargada")]
    NoRom,
    /// El core se detuvo ante hardware o instrucciones que no soporta.
    #[error("error de emulación: {0}")]
    Emulation(String),
    /// No se pudo guardar o restaurar el estado.
    #[error("error en el estado guardado: {0}")]
    SaveState(String),
}

/// Botones de la GBA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    L,
    R,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    /// Todos los botones, en el orden en que se muestran en la interfaz.
    pub const ALL: [Self; 10] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::A,
        Self::B,
        Self::L,
        Self::R,
        Self::Select,
        Self::Start,
    ];

    /// Bit del botón en el registro `KEYINPUT`.
    const fn bit(self) -> u16 {
        let key = match self {
            Self::A => Keys::ButtonA,
            Self::B => Keys::ButtonB,
            Self::L => Keys::ButtonL,
            Self::R => Keys::ButtonR,
            Self::Select => Keys::Select,
            Self::Start => Keys::Start,
            Self::Up => Keys::Up,
            Self::Down => Keys::Down,
            Self::Left => Keys::Left,
            Self::Right => Keys::Right,
        };
        1 << key as u16
    }
}

/// Tipo de reinicio de la consola.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetKind {
    /// Reinicia la CPU y conserva la RAM.
    Soft,
    /// Apagar y encender: borra la RAM. La partida guardada del cartucho se conserva.
    Hard,
}

/// Datos de la ROM cargada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomInfo {
    /// Nombre con el que se cargó la ROM.
    pub name: String,
    /// Título interno del cartucho (hasta 12 caracteres).
    pub title: String,
    /// Código de juego del cartucho (4 caracteres, p. ej. `AXVE`).
    pub game_code: String,
    /// Si el cartucho guarda partida (SRAM, Flash o EEPROM).
    pub battery_backed: bool,
}

/// Salidas que comparte [`Emulator`] con el core mientras emula un frame.
#[derive(Default)]
struct AudioSink {
    /// Muestras estéreo intercaladas del frame en curso.
    samples: RefCell<Vec<f32>>,
    /// Si es `true`, las muestras se descartan.
    muted: Cell<bool>,
    sample_rate: Cell<f32>,
}

/// Extremo del [`AudioSink`] que se entrega al core.
struct AudioOutput(Rc<AudioSink>);

impl AudioInterface for AudioOutput {
    fn get_sample_rate(&self) -> i32 {
        // Las frecuencias de audio son enteros pequeños: la conversión es exacta.
        #[allow(clippy::cast_possible_truncation)]
        let rate = self.0.sample_rate.get().round() as i32;
        rate
    }

    fn push_sample(&mut self, sample: &[i16; 2]) {
        if !self.0.muted.get() {
            let mut samples = self.0.samples.borrow_mut();
            samples.push(f32::from(sample[0]) / 32_768.0);
            samples.push(f32::from(sample[1]) / 32_768.0);
        }
    }
}

/// Un cartucho insertado y la consola que lo ejecuta.
struct Session {
    gba: GameBoyAdvance,
    info: RomInfo,
    /// Copia de la ROM, para poder apagar y encender la consola.
    rom: Vec<u8>,
}

/// Una consola Game Boy Advance emulada.
///
/// El ciclo de uso es: cargar una ROM con [`Emulator::load_rom`], y por cada frame
/// de pantalla fijar los botones con [`Emulator::set_button`], llamar a
/// [`Emulator::run_frame`] y recoger la imagen ([`Emulator::frame`]) y el audio
/// ([`Emulator::audio_samples`]).
///
/// No accede al disco: las partidas y estados se entregan como bytes para que la
/// app decida dónde guardarlos (también en la web, donde no hay sistema de archivos).
pub struct Emulator {
    session: Option<Session>,
    bios: Box<[u8]>,
    audio: Rc<AudioSink>,
    frame: Box<[u8; FRAME_BYTES]>,
    /// Botones pulsados, como máscara con los bits de `KEYINPUT`.
    pressed: u16,
}

impl std::fmt::Debug for Emulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Emulator")
            .field("rom", &self.rom())
            .field("pressed", &self.pressed)
            .finish_non_exhaustive()
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Emulator {
    /// Crea una consola sin cartucho, con la BIOS libre incluida y audio a
    /// [`DEFAULT_SAMPLE_RATE`].
    #[must_use]
    pub fn new() -> Self {
        let audio = Rc::new(AudioSink::default());
        audio.sample_rate.set(DEFAULT_SAMPLE_RATE);
        Self {
            session: None,
            bios: Box::from(OPEN_BIOS.as_slice()),
            audio,
            frame: Box::new([0; FRAME_BYTES]),
            pressed: 0,
        }
    }

    /// Usa una BIOS distinta de la incluida (p. ej. un volcado de la original, que da
    /// la máxima compatibilidad). Se aplica a partir de la siguiente carga de ROM o
    /// reinicio duro.
    ///
    /// # Errors
    ///
    /// Si `bios` no ocupa [`BIOS_SIZE`] bytes.
    pub fn set_bios(&mut self, bios: &[u8]) -> Result<(), Error> {
        if bios.len() != BIOS_SIZE {
            return Err(Error::InvalidBios(bios.len()));
        }
        self.bios = Box::from(bios);
        Ok(())
    }

    /// Vuelve a la BIOS libre incluida.
    pub fn use_open_bios(&mut self) {
        self.bios = Box::from(OPEN_BIOS.as_slice());
    }

    /// Carga una ROM de GBA desde memoria (`.gba`, o un `.zip` que la contenga) y
    /// enciende la consola.
    ///
    /// # Errors
    ///
    /// Si los datos no son una ROM de GBA válida.
    pub fn load_rom(&mut self, name: &str, data: &[u8]) -> Result<&RomInfo, Error> {
        self.session = None;
        self.frame.fill(0);
        self.audio.samples.borrow_mut().clear();

        let gba = self.power_on(data)?;
        let info = RomInfo {
            name: name.to_owned(),
            title: clean_header_text(&gba.get_game_title()),
            game_code: clean_header_text(&gba.get_game_code()),
            battery_backed: gba.backup_bytes().is_some(),
        };
        let session = self.session.insert(Session {
            gba,
            info,
            rom: data.to_vec(),
        });
        Ok(&session.info)
    }

    /// Construye una consola recién encendida con `rom` insertada.
    fn power_on(&self, rom: &[u8]) -> Result<GameBoyAdvance, Error> {
        let bios = self.bios.clone();
        let audio = Box::new(AudioOutput(Rc::clone(&self.audio)));
        // El core valida con `panic!` algunas cabeceras corruptas.
        guard(|| {
            let cartridge = GamepakBuilder::new()
                .buffer(rom)
                .without_backup_to_file()
                .build()
                .map_err(|err| Error::LoadRom(err.to_string()))?;
            let mut gba = GameBoyAdvance::new(bios, cartridge, audio);
            gba.skip_bios();
            Ok(gba)
        })
        .map_err(Error::LoadRom)?
    }

    /// Extrae el cartucho.
    pub fn unload_rom(&mut self) {
        self.session = None;
    }

    /// Datos de la ROM cargada, si la hay.
    #[must_use]
    pub fn rom(&self) -> Option<&RomInfo> {
        self.session.as_ref().map(|session| &session.info)
    }

    /// Emula un frame completo de vídeo (y el audio correspondiente).
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada o el core se detiene ante algo que no soporta. En ese
    /// último caso la consola queda en un estado indefinido: hay que reiniciarla con
    /// [`Emulator::reset`] ([`ResetKind::Hard`]) o cargar otra ROM.
    pub fn run_frame(&mut self) -> Result<(), Error> {
        self.clock(true, true)
    }

    /// Emula un frame sin generar sonido: [`Emulator::audio_samples`] queda vacío.
    ///
    /// # Errors
    ///
    /// Los mismos que [`Emulator::run_frame`].
    pub fn run_frame_silent(&mut self) -> Result<(), Error> {
        self.clock(true, false)
    }

    /// Emula un frame sin entregar imagen ni sonido.
    ///
    /// Para los frames que no se van a mostrar (avance rápido): el juego avanza
    /// exactamente igual, pero [`Emulator::frame`] conserva la imagen anterior.
    ///
    /// # Errors
    ///
    /// Los mismos que [`Emulator::run_frame`].
    pub fn skip_frame(&mut self) -> Result<(), Error> {
        self.clock(false, false)
    }

    /// Emula un frame entregando solo las salidas indicadas.
    fn clock(&mut self, video: bool, audio: bool) -> Result<(), Error> {
        let session = self.session.as_mut().ok_or(Error::NoRom)?;
        self.audio.samples.borrow_mut().clear();
        self.audio.muted.set(!audio);

        let pressed = self.pressed;
        let gba = &mut session.gba;
        guard(|| {
            *gba.get_key_state_mut() = KEYINPUT_ALL_RELEASED & !pressed;
            gba.frame();
        })
        .map_err(Error::Emulation)?;

        if video {
            let pixels = session.gba.get_frame_buffer();
            let (rgba_pixels, _) = self.frame.as_chunks_mut::<4>();
            for (rgba, pixel) in rgba_pixels.iter_mut().zip(pixels) {
                // El core entrega 0x00RRGGBB.
                let [_, r, g, b] = pixel.to_be_bytes();
                *rgba = [r, g, b, 0xFF];
            }
        }
        Ok(())
    }

    /// Imagen del último frame: [`FRAME_BYTES`] bytes RGBA, 240×160, fila a fila.
    #[must_use]
    pub fn frame(&self) -> &[u8; FRAME_BYTES] {
        &self.frame
    }

    /// Copia en `out` las muestras de audio generadas por el último
    /// [`Emulator::run_frame`]: estéreo intercalado (izquierda, derecha), `f32` de -1
    /// a 1, a la frecuencia configurada. `out` se vacía antes.
    pub fn audio_samples(&self, out: &mut Vec<f32>) {
        out.clear();
        out.extend_from_slice(&self.audio.samples.borrow());
    }

    /// Cambia la frecuencia de muestreo del audio, en Hz.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.audio.sample_rate.set(sample_rate);
        if let Some(session) = &mut self.session {
            session.gba.set_audio_sample_rate(sample_rate);
        }
    }

    /// Pulsa o suelta un botón.
    pub const fn set_button(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.pressed |= button.bit();
        } else {
            self.pressed &= !button.bit();
        }
    }

    /// Reinicia la consola.
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada.
    pub fn reset(&mut self, kind: ResetKind) -> Result<(), Error> {
        let Some(mut session) = self.session.take() else {
            return Err(Error::NoRom);
        };
        match kind {
            ResetKind::Soft => {
                session.gba.soft_reset();
                session.gba.skip_bios();
            }
            ResetKind::Hard => {
                let battery = session.gba.backup_bytes();
                session.gba = self.power_on(&session.rom)?;
                if let Some(battery) = battery {
                    session.gba.load_backup_bytes(&battery);
                }
            }
        }
        self.session = Some(session);
        Ok(())
    }

    /// Serializa el estado completo de la consola (save state).
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada o falla la serialización.
    pub fn save_state(&self) -> Result<Vec<u8>, Error> {
        let session = self.session.as_ref().ok_or(Error::NoRom)?;
        session
            .gba
            .save_state()
            .map_err(|err| Error::SaveState(err.to_string()))
    }

    /// Restaura un estado creado con [`Emulator::save_state`] para la misma ROM.
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada o los datos están corruptos.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), Error> {
        let session = self.session.as_mut().ok_or(Error::NoRom)?;
        let gba = &mut session.gba;
        guard(|| gba.restore_state(data))
            .map_err(Error::SaveState)?
            .map_err(|err| Error::SaveState(err.to_string()))?;
        // El estado guarda la frecuencia de audio con la que se creó.
        gba.set_audio_sample_rate(self.audio.sample_rate.get());
        Ok(())
    }

    /// Memoria de guardado del cartucho (la partida), o `None` si la ROM cargada no
    /// guarda partida.
    #[must_use]
    pub fn battery_ram(&self) -> Option<Vec<u8>> {
        self.session.as_ref()?.gba.backup_bytes()
    }

    /// Restaura la memoria de guardado del cartucho. Se llama justo después de cargar
    /// la ROM.
    pub fn set_battery_ram(&mut self, data: &[u8]) {
        if let Some(session) = &mut self.session {
            session.gba.load_backup_bytes(data);
        }
    }
}

/// Ejecuta `f` convirtiendo un `panic!` del core en un error con su mensaje.
fn guard<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    panic::catch_unwind(AssertUnwindSafe(f)).map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
            .unwrap_or_else(|| "fallo interno del core".to_owned())
    })
}

/// Limpia un texto de la cabecera del cartucho: sin relleno ni caracteres de control.
fn clean_header_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_owned()
}

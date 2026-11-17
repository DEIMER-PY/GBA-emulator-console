//! Controles: teclado y mandos convertidos en botones de la GBA.

use eframe::egui::{self, Key};
use gba_core::Button;
use gilrs::{Axis, Button as PadButton, EventType, Gilrs, ev::Code};
use serde::{Deserialize, Serialize};

/// Inclinación mínima del stick analógico para contar como dirección.
const STICK_THRESHOLD: f32 = 0.5;

/// Un valor por cada botón de la GBA.
///
/// Según el tipo sirve de estado del mando ([`PadState`]), de teclas asignadas
/// ([`KeyBindings`]) o de botones de mando asignados ([`PadBindings`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonMap<T> {
    pub up: T,
    pub down: T,
    pub left: T,
    pub right: T,
    pub a: T,
    pub b: T,
    pub l: T,
    pub r: T,
    pub select: T,
    pub start: T,
}

impl<T: Copy> ButtonMap<T> {
    /// Valor de un botón.
    pub const fn get(&self, button: Button) -> T {
        match button {
            Button::Up => self.up,
            Button::Down => self.down,
            Button::Left => self.left,
            Button::Right => self.right,
            Button::A => self.a,
            Button::B => self.b,
            Button::L => self.l,
            Button::R => self.r,
            Button::Select => self.select,
            Button::Start => self.start,
        }
    }

    const fn get_mut(&mut self, button: Button) -> &mut T {
        match button {
            Button::Up => &mut self.up,
            Button::Down => &mut self.down,
            Button::Left => &mut self.left,
            Button::Right => &mut self.right,
            Button::A => &mut self.a,
            Button::B => &mut self.b,
            Button::L => &mut self.l,
            Button::R => &mut self.r,
            Button::Select => &mut self.select,
            Button::Start => &mut self.start,
        }
    }

    /// Botones pulsados según `is_down`, que indica si la tecla o el botón físico
    /// asignado está pulsado.
    fn state(&self, is_down: impl Fn(T) -> bool) -> PadState {
        PadState {
            up: is_down(self.up),
            down: is_down(self.down),
            left: is_down(self.left),
            right: is_down(self.right),
            a: is_down(self.a),
            b: is_down(self.b),
            l: is_down(self.l),
            r: is_down(self.r),
            select: is_down(self.select),
            start: is_down(self.start),
        }
    }
}

impl<T: Copy + PartialEq> ButtonMap<T> {
    /// Qué botón tiene asignado `value`, si alguno.
    pub fn find(&self, value: T) -> Option<Button> {
        Button::ALL
            .into_iter()
            .find(|&button| self.get(button) == value)
    }

    /// Asigna `value` a un botón.
    ///
    /// Si ya estaba en otro botón, los dos intercambian su asignación: así nunca queda
    /// una tecla en dos botones ni un botón sin tecla.
    pub fn rebind(&mut self, button: Button, value: T) {
        let previous = self.get(button);
        if let Some(other) = self.find(value) {
            *self.get_mut(other) = previous;
        }
        *self.get_mut(button) = value;
    }
}

/// Botones pulsados.
pub type PadState = ButtonMap<bool>;

impl Default for PadState {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            l: false,
            r: false,
            select: false,
            start: false,
        }
    }
}

impl PadState {
    /// Une dos fuentes: un botón está pulsado si lo está en cualquiera de ellas.
    pub const fn merge(self, other: Self) -> Self {
        Self {
            up: self.up || other.up,
            down: self.down || other.down,
            left: self.left || other.left,
            right: self.right || other.right,
            a: self.a || other.a,
            b: self.b || other.b,
            l: self.l || other.l,
            r: self.r || other.r,
            select: self.select || other.select,
            start: self.start || other.start,
        }
    }
}

/// Tecla asignada a cada botón.
pub type KeyBindings = ButtonMap<Key>;

impl Default for KeyBindings {
    /// Flechas, `X`/`Z` para A/B, `A`/`S` para L/R, `Enter` y `Retroceso`.
    fn default() -> Self {
        Self {
            up: Key::ArrowUp,
            down: Key::ArrowDown,
            left: Key::ArrowLeft,
            right: Key::ArrowRight,
            a: Key::X,
            b: Key::Z,
            l: Key::A,
            r: Key::S,
            select: Key::Backspace,
            start: Key::Enter,
        }
    }
}

impl KeyBindings {
    /// Lee el teclado.
    ///
    /// Si la interfaz está usando el teclado (un campo de texto, un menú abierto), no
    /// se pulsa nada: así escribir o navegar por los menús no mueve al personaje.
    pub fn read(&self, ctx: &egui::Context) -> PadState {
        if ctx.egui_wants_keyboard_input() {
            return PadState::default();
        }
        ctx.input(|input| {
            // Con Ctrl/Alt pulsado es un atajo de la app (Ctrl+O...), no un botón.
            if input.modifiers.command || input.modifiers.alt {
                return PadState::default();
            }
            self.state(|key| input.key_down(key))
        })
    }
}

/// Botón físico de un mando.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadInput {
    /// Botón con posición estándar (inferior, izquierdo, Start...).
    Button(PadButton),
    /// Botón que el sistema no sabe situar, habitual en mandos genéricos: se
    /// identifica por el código que envía el propio mando.
    Code(Code),
}

impl PadInput {
    /// Crea la entrada a partir de un evento de pulsación de `gilrs`.
    fn from_event(button: PadButton, code: Code) -> Self {
        if button == PadButton::Unknown {
            Self::Code(code)
        } else {
            Self::Button(button)
        }
    }

    /// Nombre para la interfaz.
    pub fn label(self) -> String {
        let name = match self {
            Self::Code(code) => return format!("Código {code}"),
            Self::Button(button) => match button {
                PadButton::South => "Inferior (A / ✕)",
                PadButton::East => "Derecho (B / ○)",
                PadButton::North => "Superior (Y / △)",
                PadButton::West => "Izquierdo (X / □)",
                PadButton::LeftTrigger => "L1",
                PadButton::LeftTrigger2 => "L2",
                PadButton::RightTrigger => "R1",
                PadButton::RightTrigger2 => "R2",
                PadButton::Select => "Select",
                PadButton::Start => "Start",
                PadButton::Mode => "Home",
                PadButton::LeftThumb => "Stick izquierdo",
                PadButton::RightThumb => "Stick derecho",
                PadButton::DPadUp => "Cruceta arriba",
                PadButton::DPadDown => "Cruceta abajo",
                PadButton::DPadLeft => "Cruceta izquierda",
                PadButton::DPadRight => "Cruceta derecha",
                other => return format!("{other:?}"),
            },
        };
        name.to_owned()
    }
}

/// Botón físico del mando asignado a cada botón de la GBA.
///
/// El stick izquierdo siempre funciona además como dirección.
pub type PadBindings = ButtonMap<PadInput>;

impl Default for PadBindings {
    /// Cruceta, botón inferior (A en Xbox, ✕ en PlayStation) = A, izquierdo (X, □) = B,
    /// y los gatillos superiores = L y R.
    fn default() -> Self {
        Self {
            up: PadInput::Button(PadButton::DPadUp),
            down: PadInput::Button(PadButton::DPadDown),
            left: PadInput::Button(PadButton::DPadLeft),
            right: PadInput::Button(PadButton::DPadRight),
            a: PadInput::Button(PadButton::South),
            b: PadInput::Button(PadButton::West),
            l: PadInput::Button(PadButton::LeftTrigger),
            r: PadInput::Button(PadButton::RightTrigger),
            select: PadInput::Button(PadButton::Select),
            start: PadInput::Button(PadButton::Start),
        }
    }
}

/// Mandos conectados (USB, Bluetooth).
///
/// La GBA tiene un solo jugador: cualquier mando conectado la controla.
pub struct Gamepads {
    /// `None` si el sistema no ofrece acceso a mandos; la app sigue con teclado.
    gilrs: Option<Gilrs>,
    /// Botones pulsados desde la última llamada a [`Gamepads::poll`].
    pressed: Vec<PadInput>,
}

impl Default for Gamepads {
    fn default() -> Self {
        Self {
            gilrs: Gilrs::new().ok(),
            pressed: Vec::new(),
        }
    }
}

impl Gamepads {
    /// Procesa los eventos pendientes de los mandos. Se llama una vez por repintado,
    /// antes de [`Gamepads::read`] y [`Gamepads::first_pressed`].
    pub fn poll(&mut self) {
        self.pressed.clear();
        let Some(gilrs) = &mut self.gilrs else {
            return;
        };
        // Consumir los eventos es además lo que actualiza el estado de cada mando.
        while let Some(event) = gilrs.next_event() {
            if let EventType::ButtonPressed(button, code) = event.event {
                self.pressed.push(PadInput::from_event(button, code));
            }
        }
    }

    /// Primer botón pulsado en cualquier mando desde el último [`Gamepads::poll`].
    pub fn first_pressed(&self) -> Option<PadInput> {
        self.pressed.first().copied()
    }

    /// Estado de los botones según su asignación, uniendo todos los mandos.
    pub fn read(&self, bindings: &PadBindings) -> PadState {
        let mut state = PadState::default();
        let Some(gilrs) = &self.gilrs else {
            return state;
        };
        for (_, gamepad) in gilrs.gamepads() {
            let (stick_left, stick_right) = stick_direction(gamepad.value(Axis::LeftStickX));
            // El eje Y de gilrs es positivo hacia arriba.
            let (stick_down, stick_up) = stick_direction(gamepad.value(Axis::LeftStickY));
            let stick = PadState {
                up: stick_up,
                down: stick_down,
                left: stick_left,
                right: stick_right,
                ..PadState::default()
            };
            let buttons = bindings.state(|input| match input {
                PadInput::Button(button) => gamepad.is_pressed(button),
                PadInput::Code(code) => gamepad.state().is_pressed(code),
            });
            state = state.merge(buttons).merge(stick);
        }
        state
    }

    /// Nombres de los mandos conectados.
    pub fn names(&self) -> Vec<String> {
        self.gilrs.as_ref().map_or_else(Vec::new, |gilrs| {
            gilrs
                .gamepads()
                .map(|(_, gamepad)| gamepad.name().to_owned())
                .collect()
        })
    }
}

/// Convierte un eje analógico en dos direcciones digitales: (negativa, positiva).
fn stick_direction(value: f32) -> (bool, bool) {
    (value < -STICK_THRESHOLD, value > STICK_THRESHOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keys_do_not_collide() {
        let bindings = KeyBindings::default();
        for button in Button::ALL {
            assert_eq!(bindings.find(bindings.get(button)), Some(button));
        }
    }

    #[test]
    fn key_map_reads_pressed_keys() {
        let state = KeyBindings::default().state(|key| matches!(key, Key::X | Key::A));
        for button in Button::ALL {
            let expected = matches!(button, Button::A | Button::L);
            assert_eq!(state.get(button), expected, "{button:?}");
        }
    }

    #[test]
    fn merge_is_a_logical_or() {
        let keyboard = PadState {
            a: true,
            ..PadState::default()
        };
        let gamepad = PadState {
            r: true,
            ..PadState::default()
        };
        let merged = keyboard.merge(gamepad);
        assert!(merged.a && merged.r);
        assert!(!merged.b);
        assert_eq!(merged.merge(PadState::default()), merged);
    }

    #[test]
    fn rebind_assigns_a_free_key() {
        let mut bindings = KeyBindings::default();
        bindings.rebind(Button::A, Key::Space);
        assert_eq!(bindings.a, Key::Space);
        assert_eq!(bindings.find(Key::X), None, "la tecla anterior queda libre");
    }

    #[test]
    fn rebind_swaps_with_the_button_that_had_the_key() {
        let mut bindings = KeyBindings::default();
        // `Z` es el botón B; al dársela a A, B se queda con la que tenía A.
        bindings.rebind(Button::A, Key::Z);
        assert_eq!(bindings.find(Key::Z), Some(Button::A));
        assert_eq!(bindings.find(Key::X), Some(Button::B));
    }

    #[test]
    fn rebind_to_the_same_key_changes_nothing() {
        let mut bindings = KeyBindings::default();
        bindings.rebind(Button::A, Key::X);
        assert_eq!(bindings, KeyBindings::default());
    }

    #[test]
    fn stick_needs_to_pass_the_threshold() {
        assert_eq!(stick_direction(0.0), (false, false));
        assert_eq!(stick_direction(0.3), (false, false));
        assert_eq!(stick_direction(0.9), (false, true));
        assert_eq!(stick_direction(-0.9), (true, false));
    }

    #[test]
    fn default_pad_buttons_do_not_collide() {
        let bindings = PadBindings::default();
        for button in Button::ALL {
            assert_eq!(bindings.find(bindings.get(button)), Some(button));
        }
    }

    #[test]
    fn pad_map_reads_pressed_buttons() {
        let shoulder = PadInput::Button(PadButton::LeftTrigger);
        let state = PadBindings::default().state(|input| input == shoulder);
        assert_eq!(
            state,
            PadState {
                l: true,
                ..PadState::default()
            }
        );
    }

    #[test]
    fn pad_rebind_swaps_with_the_button_that_had_it() {
        let mut bindings = PadBindings::default();
        // El botón inferior es A; al dárselo a B, A se queda con el que tenía B.
        bindings.rebind(Button::B, PadInput::Button(PadButton::South));
        assert_eq!(bindings.b, PadInput::Button(PadButton::South));
        assert_eq!(bindings.a, PadInput::Button(PadButton::West));
    }

    #[test]
    fn pad_buttons_have_readable_labels() {
        assert_eq!(
            PadInput::Button(PadButton::South).label(),
            "Inferior (A / ✕)"
        );
        assert_eq!(
            PadInput::from_event(PadButton::Start, sample_code()),
            PadInput::Button(PadButton::Start)
        );
    }

    #[test]
    fn unknown_buttons_are_identified_by_code() {
        let code = sample_code();
        let input = PadInput::from_event(PadButton::Unknown, code);
        assert_eq!(input, PadInput::Code(code));
        assert!(input.label().starts_with("Código "));

        let mut bindings = PadBindings::default();
        bindings.rebind(Button::A, input);
        assert_eq!(bindings.find(input), Some(Button::A));
    }

    #[test]
    fn bindings_round_trip_through_ron() {
        let keys = KeyBindings::default();
        let text = ron::to_string(&keys).unwrap();
        assert_eq!(ron::from_str::<KeyBindings>(&text).unwrap(), keys);

        let mut pads = PadBindings::default();
        pads.rebind(Button::A, PadInput::Button(PadButton::East));
        pads.rebind(Button::B, PadInput::Code(sample_code()));
        let text = ron::to_string(&pads).unwrap();
        assert_eq!(ron::from_str::<PadBindings>(&text).unwrap(), pads);
    }

    /// Un código de botón cualquiera, obtenido de un evento serializado: `gilrs` no
    /// permite construirlos directamente.
    fn sample_code() -> Code {
        let text = ron::to_string(&gilrs::ev::Button::South.to_nec()).unwrap();
        ron::from_str::<Option<Code>>(&text)
            .unwrap()
            .expect("el botón inferior tiene código nativo")
    }
}

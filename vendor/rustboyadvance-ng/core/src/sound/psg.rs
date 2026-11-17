//! PARCHE gba-emulator-console (archivo nuevo): los cuatro canales PSG de la GBA,
//! heredados de Game Boy. El core original guardaba algunos de sus registros pero no
//! generaba su sonido.
//!
//! - Canal 1: onda cuadrada con barrido de frecuencia y envolvente.
//! - Canal 2: onda cuadrada con envolvente.
//! - Canal 3: onda programable de 4 bits (dos bancos de 32 muestras).
//! - Canal 4: ruido (registro de desplazamiento) con envolvente.
//!
//! Referencia: GBATEK, «GBA Sound Channel 1-4». Los tiempos se expresan en ciclos de
//! la CPU de la GBA (16 777 216 Hz).

use serde::{Deserialize, Serialize};

use crate::iodev::consts::*;

/// Ciclos entre pasos del secuenciador de frames (512 Hz).
const FRAME_SEQUENCER_PERIOD: i32 = 32_768;

/// Patrones de la onda cuadrada (un bit por paso; 12,5 %, 25 %, 50 % y 75 %).
const DUTY_PATTERNS: [u8; 4] = [0b0000_0001, 0b1000_0001, 0b1000_0111, 0b0111_1110];

/// Envolvente de volumen (64 Hz).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Envelope {
    initial: u8,
    increase: bool,
    period: u8,
    volume: u8,
    timer: u8,
}

impl Envelope {
    /// Escribe el byte de envolvente (NRx2): periodo, dirección y volumen inicial.
    fn write(&mut self, value: u8) {
        self.period = value & 7;
        self.increase = value & 8 != 0;
        self.initial = value >> 4;
    }

    /// El canal solo puede sonar si el volumen inicial o la dirección no son cero.
    const fn dac_enabled(&self) -> bool {
        self.initial != 0 || self.increase
    }

    fn trigger(&mut self) {
        self.volume = self.initial;
        self.timer = self.period;
    }

    fn clock(&mut self) {
        if self.period == 0 {
            return;
        }
        self.timer = self.timer.saturating_sub(1);
        if self.timer == 0 {
            self.timer = self.period;
            if self.increase && self.volume < 15 {
                self.volume += 1;
            } else if !self.increase && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

/// Contador de duración (256 Hz): apaga el canal al agotarse.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Length {
    enabled: bool,
    counter: u16,
}

impl Length {
    /// Avanza el contador. Devuelve `true` si el canal debe apagarse.
    fn clock(&mut self) -> bool {
        if self.enabled && self.counter > 0 {
            self.counter -= 1;
            return self.counter == 0;
        }
        false
    }

    fn trigger(&mut self, max: u16) {
        if self.counter == 0 {
            self.counter = max;
        }
    }
}

/// Canales 1 y 2: onda cuadrada. El barrido solo lo usa el canal 1.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Square {
    enabled: bool,
    duty: u8,
    step: u8,
    frequency: u16,
    timer: i32,
    envelope: Envelope,
    length: Length,

    sweep_period: u8,
    sweep_decrease: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,
}

impl Square {
    /// Ciclos de CPU por cada uno de los 8 pasos de la onda.
    fn period(&self) -> i32 {
        (2048 - i32::from(self.frequency)) * 16
    }

    fn write_sweep(&mut self, value: u16) {
        self.sweep_shift = (value & 7) as u8;
        self.sweep_decrease = value & 8 != 0;
        self.sweep_period = ((value >> 4) & 7) as u8;
    }

    fn write_duty_envelope(&mut self, value: u16) {
        self.length.counter = 64 - (value & 0x3f);
        self.duty = ((value >> 6) & 3) as u8;
        self.envelope.write((value >> 8) as u8);
        if !self.envelope.dac_enabled() {
            self.enabled = false;
        }
    }

    fn write_frequency(&mut self, value: u16) {
        self.frequency = value & 0x7ff;
        self.length.enabled = value & 0x4000 != 0;
        if value & 0x8000 != 0 {
            self.trigger();
        }
    }

    fn trigger(&mut self) {
        self.enabled = self.envelope.dac_enabled();
        self.length.trigger(64);
        self.timer = self.period();
        self.envelope.trigger();

        self.sweep_shadow = self.frequency;
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        self.sweep_enabled = self.sweep_period != 0 || self.sweep_shift != 0;
        if self.sweep_shift != 0 && self.sweep_target() > 2047 {
            self.enabled = false;
        }
    }

    /// Frecuencia a la que llevaría el siguiente paso del barrido.
    fn sweep_target(&self) -> u16 {
        let delta = self.sweep_shadow >> self.sweep_shift;
        if self.sweep_decrease {
            self.sweep_shadow.saturating_sub(delta)
        } else {
            self.sweep_shadow + delta
        }
    }

    fn clock_sweep(&mut self) {
        self.sweep_timer = self.sweep_timer.saturating_sub(1);
        if self.sweep_timer != 0 {
            return;
        }
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        if !self.sweep_enabled || self.sweep_period == 0 {
            return;
        }
        let target = self.sweep_target();
        if target > 2047 {
            self.enabled = false;
        } else if self.sweep_shift != 0 {
            self.sweep_shadow = target;
            self.frequency = target;
            if self.sweep_target() > 2047 {
                self.enabled = false;
            }
        }
    }

    fn advance(&mut self, cycles: i32) {
        self.timer -= cycles;
        while self.timer <= 0 {
            self.timer += self.period();
            self.step = (self.step + 1) & 7;
        }
    }

    /// Muestra actual, de -15 a 15.
    fn output(&self) -> i32 {
        if !self.enabled {
            return 0;
        }
        let high = DUTY_PATTERNS[usize::from(self.duty)] >> self.step & 1 != 0;
        let volume = i32::from(self.envelope.volume);
        if high { volume } else { -volume }
    }
}

/// Canal 3: onda programable.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Wave {
    enabled: bool,
    dac_enabled: bool,
    /// `true`: los dos bancos se reproducen seguidos (64 muestras).
    two_banks: bool,
    /// Banco que se reproduce; las escrituras van al otro.
    bank: u8,
    volume_code: u8,
    force_75: bool,
    frequency: u16,
    timer: i32,
    position: u8,
    length: Length,
    ram: [[u8; 16]; 2],
}

impl Wave {
    /// Ciclos de CPU por muestra de la onda.
    fn period(&self) -> i32 {
        (2048 - i32::from(self.frequency)) * 8
    }

    fn write_control(&mut self, value: u16) {
        self.two_banks = value & 0x20 != 0;
        self.bank = ((value >> 6) & 1) as u8;
        self.dac_enabled = value & 0x80 != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    fn write_length_volume(&mut self, value: u16) {
        self.length.counter = 256 - (value & 0xff);
        self.volume_code = ((value >> 13) & 3) as u8;
        self.force_75 = value & 0x8000 != 0;
    }

    fn write_frequency(&mut self, value: u16) {
        self.frequency = value & 0x7ff;
        self.length.enabled = value & 0x4000 != 0;
        if value & 0x8000 != 0 {
            self.enabled = self.dac_enabled;
            self.length.trigger(256);
            self.timer = self.period();
            self.position = 0;
        }
    }

    /// Byte de la RAM de onda para escribir: siempre en el banco que no se reproduce.
    fn ram_byte(&mut self, index: usize) -> &mut u8 {
        &mut self.ram[usize::from(self.bank ^ 1)][index & 15]
    }

    fn advance(&mut self, cycles: i32) {
        self.timer -= cycles;
        let samples = if self.two_banks { 64 } else { 32 };
        while self.timer <= 0 {
            self.timer += self.period();
            self.position = (self.position + 1) % samples;
        }
    }

    /// Muestra actual, de -15 a 15.
    fn output(&self) -> i32 {
        if !self.enabled {
            return 0;
        }
        let position = usize::from(self.position);
        // Con dos bancos, la segunda mitad sale del otro banco.
        let bank = usize::from(self.bank) ^ (position >> 5);
        let byte = self.ram[bank & 1][(position & 31) >> 1];
        let nibble = if position & 1 == 0 { byte >> 4 } else { byte & 0xf };
        let centered = i32::from(nibble) * 2 - 15;
        if self.force_75 {
            return centered * 3 / 4;
        }
        match self.volume_code {
            0 => 0,
            1 => centered,
            2 => centered / 2,
            _ => centered / 4,
        }
    }
}

/// Canal 4: ruido.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Noise {
    enabled: bool,
    divisor: u8,
    short_mode: bool,
    shift: u8,
    lfsr: u16,
    timer: i32,
    envelope: Envelope,
    length: Length,
}

impl Noise {
    /// Ciclos de CPU entre desplazamientos del registro.
    fn period(&self) -> i32 {
        let base = if self.divisor == 0 {
            32
        } else {
            64 * i32::from(self.divisor)
        };
        // Los desplazamientos 14 y 15 no reciben reloj en hardware.
        base << self.shift.min(13)
    }

    fn write_length_envelope(&mut self, value: u16) {
        self.length.counter = 64 - (value & 0x3f);
        self.envelope.write((value >> 8) as u8);
        if !self.envelope.dac_enabled() {
            self.enabled = false;
        }
    }

    fn write_control(&mut self, value: u16) {
        self.divisor = (value & 7) as u8;
        self.short_mode = value & 8 != 0;
        self.shift = ((value >> 4) & 0xf) as u8;
        self.length.enabled = value & 0x4000 != 0;
        if value & 0x8000 != 0 {
            self.enabled = self.envelope.dac_enabled();
            self.length.trigger(64);
            self.timer = self.period();
            self.envelope.trigger();
            self.lfsr = if self.short_mode { 0x40 } else { 0x4000 };
        }
    }

    fn advance(&mut self, cycles: i32) {
        self.timer -= cycles;
        while self.timer <= 0 {
            self.timer += self.period();
            let carry = self.lfsr & 1 != 0;
            self.lfsr >>= 1;
            if carry {
                self.lfsr ^= if self.short_mode { 0x60 } else { 0x6000 };
            }
        }
    }

    /// Muestra actual, de -15 a 15.
    fn output(&self) -> i32 {
        if !self.enabled {
            return 0;
        }
        let volume = i32::from(self.envelope.volume);
        if self.lfsr & 1 != 0 { volume } else { -volume }
    }
}

/// Los cuatro canales PSG y el secuenciador que mueve sus contadores.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Psg {
    square1: Square,
    square2: Square,
    wave: Wave,
    noise: Noise,
    sequencer_timer: i32,
    sequencer_step: u8,
    /// Último valor escrito en cada registro (0x60..0x80), para poder releerlo.
    registers: [u16; 16],
}

impl Psg {
    /// Si `io_addr` es un registro de los canales PSG o la RAM de onda.
    pub fn handles(io_addr: u32) -> bool {
        (REG_SOUND1CNT_L..REG_SOUNDCNT_L).contains(&io_addr)
            || (REG_WAVE_RAM..REG_WAVE_RAM + 16).contains(&io_addr)
    }

    pub fn read(&self, io_addr: u32) -> u16 {
        if io_addr >= REG_WAVE_RAM {
            let index = (io_addr - REG_WAVE_RAM) as usize;
            let bank = &self.wave.ram[usize::from(self.wave.bank ^ 1)];
            return u16::from(bank[index & 15]) | u16::from(bank[(index + 1) & 15]) << 8;
        }
        // Los bits de disparo son de solo escritura.
        let mask = match io_addr {
            REG_SOUND1CNT_X | REG_SOUND2CNT_H | REG_SOUND3CNT_X | REG_SOUND4CNT_H => 0x7fff,
            _ => 0xffff,
        };
        self.registers[Self::register_index(io_addr)] & mask
    }

    pub fn write(&mut self, io_addr: u32, value: u16) {
        if io_addr >= REG_WAVE_RAM {
            let index = (io_addr - REG_WAVE_RAM) as usize;
            *self.wave.ram_byte(index) = value as u8;
            *self.wave.ram_byte(index + 1) = (value >> 8) as u8;
            return;
        }
        self.registers[Self::register_index(io_addr)] = value;
        match io_addr {
            REG_SOUND1CNT_L => self.square1.write_sweep(value),
            REG_SOUND1CNT_H => self.square1.write_duty_envelope(value),
            REG_SOUND1CNT_X => self.square1.write_frequency(value),
            REG_SOUND2CNT_L => self.square2.write_duty_envelope(value),
            REG_SOUND2CNT_H => self.square2.write_frequency(value),
            REG_SOUND3CNT_L => self.wave.write_control(value),
            REG_SOUND3CNT_H => self.wave.write_length_volume(value),
            REG_SOUND3CNT_X => self.wave.write_frequency(value),
            REG_SOUND4CNT_L => self.noise.write_length_envelope(value),
            REG_SOUND4CNT_H => self.noise.write_control(value),
            _ => {}
        }
    }

    fn register_index(io_addr: u32) -> usize {
        ((io_addr - REG_SOUND1CNT_L) / 2) as usize & 15
    }

    /// Apaga los cuatro canales (al desactivar el sonido en `SOUNDCNT_X`).
    pub fn power_off(&mut self) {
        let ram = self.wave.ram;
        *self = Self::default();
        self.wave.ram = ram;
    }

    /// Bits 0-3 de `SOUNDCNT_X`: qué canales están sonando.
    pub fn status(&self) -> u16 {
        u16::from(self.square1.enabled)
            | u16::from(self.square2.enabled) << 1
            | u16::from(self.wave.enabled) << 2
            | u16::from(self.noise.enabled) << 3
    }

    /// Avanza los canales `cycles` ciclos de CPU.
    pub fn advance(&mut self, cycles: usize) {
        let cycles = cycles as i32;
        self.square1.advance(cycles);
        self.square2.advance(cycles);
        self.wave.advance(cycles);
        self.noise.advance(cycles);

        self.sequencer_timer -= cycles;
        while self.sequencer_timer <= 0 {
            self.sequencer_timer += FRAME_SEQUENCER_PERIOD;
            self.clock_sequencer();
        }
    }

    /// Secuenciador de 512 Hz: duración a 256 Hz, barrido a 128 Hz, envolvente a 64 Hz.
    fn clock_sequencer(&mut self) {
        let step = self.sequencer_step;
        self.sequencer_step = (step + 1) & 7;

        if step % 2 == 0 {
            if self.square1.length.clock() {
                self.square1.enabled = false;
            }
            if self.square2.length.clock() {
                self.square2.enabled = false;
            }
            if self.wave.length.clock() {
                self.wave.enabled = false;
            }
            if self.noise.length.clock() {
                self.noise.enabled = false;
            }
        }
        if step == 2 || step == 6 {
            self.square1.clock_sweep();
        }
        if step == 7 {
            self.square1.envelope.clock();
            self.square2.envelope.clock();
            self.noise.envelope.clock();
        }
    }

    /// Muestras actuales de los canales 1 a 4, cada una de -15 a 15.
    pub fn outputs(&self) -> [i32; 4] {
        [
            self.square1.output(),
            self.square2.output(),
            self.wave.output(),
            self.noise.output(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canal 2 con volumen máximo, onda al 50 % y la frecuencia indicada.
    fn tone(frequency: u16) -> Psg {
        let mut psg = Psg::default();
        psg.write(REG_SOUND2CNT_L, 0xF080);
        psg.write(REG_SOUND2CNT_H, 0x8000 | frequency);
        psg
    }

    /// Flancos de subida del canal 2 durante un segundo, muestreando a 32 768 Hz.
    fn rising_edges(psg: &mut Psg) -> u32 {
        let mut edges = 0;
        let mut previous = psg.outputs()[1];
        for _ in 0..32_768 {
            psg.advance(512);
            let current = psg.outputs()[1];
            if previous < 0 && current > 0 {
                edges += 1;
            }
            previous = current;
        }
        edges
    }

    #[test]
    fn square_wave_has_the_programmed_pitch() {
        // 131072 / (2048 - 1750) = 439,8 Hz (un la).
        let mut psg = tone(1750);
        assert_eq!(psg.status(), 0b0010);
        let edges = rising_edges(&mut psg);
        assert!((438..=442).contains(&edges), "{edges} Hz");
    }

    #[test]
    fn square_wave_swings_around_zero_at_full_volume() {
        let mut psg = tone(1750);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..1000 {
            psg.advance(512);
            seen.insert(psg.outputs()[1]);
        }
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), [-15, 15]);
    }

    #[test]
    fn length_counter_silences_the_channel() {
        let mut psg = Psg::default();
        // Duración 64 - 60 = 4 pasos de 1/256 s, con el contador activado.
        psg.write(REG_SOUND2CNT_L, 0xF080 | 60);
        psg.write(REG_SOUND2CNT_H, 0xC000 | 1750);
        assert_eq!(psg.status(), 0b0010);
        // 5/256 s bastan para agotarla.
        psg.advance(16_777_216 * 5 / 256);
        assert_eq!(psg.status(), 0);
        assert_eq!(psg.outputs()[1], 0);
    }

    #[test]
    fn envelope_fades_out() {
        let mut psg = Psg::default();
        // Volumen 15, decreciente, un paso cada 1/64 s.
        psg.write(REG_SOUND2CNT_L, 0xF180);
        psg.write(REG_SOUND2CNT_H, 0x8000 | 1750);
        // 16/64 s: el volumen llega a 0.
        psg.advance(16_777_216 / 4 + 32_768);
        for _ in 0..100 {
            psg.advance(512);
            assert_eq!(psg.outputs()[1], 0);
        }
    }

    #[test]
    fn zero_envelope_keeps_the_channel_off() {
        let mut psg = Psg::default();
        psg.write(REG_SOUND2CNT_L, 0x0080);
        psg.write(REG_SOUND2CNT_H, 0x8000 | 1750);
        assert_eq!(psg.status(), 0);
    }

    #[test]
    fn sweep_overflow_turns_channel_one_off() {
        let mut psg = Psg::default();
        // Barrido ascendente rápido: 1000 -> 1500 -> fuera de rango.
        psg.write(REG_SOUND1CNT_L, 0x0011);
        psg.write(REG_SOUND1CNT_H, 0xF080);
        psg.write(REG_SOUND1CNT_X, 0x8000 | 1000);
        assert_eq!(psg.status(), 0b0001);
        psg.advance(16_777_216 / 4);
        assert_eq!(psg.status(), 0);
    }

    #[test]
    fn wave_channel_plays_the_written_samples() {
        let mut psg = Psg::default();
        // Se escribe en el banco 1 (el que no suena) y luego se selecciona.
        psg.write(REG_SOUND3CNT_L, 0x0000);
        psg.write(REG_WAVE_RAM, 0xF00F);
        psg.write(REG_SOUND3CNT_L, 0x00C0); // DAC activo, banco 1
        psg.write(REG_SOUND3CNT_H, 0x2000); // volumen 100 %
        psg.write(REG_SOUND3CNT_X, 0x8000 | 2047);
        assert_eq!(psg.status(), 0b0100);

        // Muestras 0 a 3: 0, F, F, 0 -> -15, 15, 15, -15.
        let mut seen = vec![psg.outputs()[2]];
        for _ in 0..3 {
            psg.advance(8);
            seen.push(psg.outputs()[2]);
        }
        assert_eq!(seen, [-15, 15, 15, -15]);
    }

    #[test]
    fn noise_channel_produces_both_levels() {
        let mut psg = Psg::default();
        psg.write(REG_SOUND4CNT_L, 0xF000);
        psg.write(REG_SOUND4CNT_H, 0x8000);
        assert_eq!(psg.status(), 0b1000);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..2000 {
            psg.advance(512);
            seen.insert(psg.outputs()[3]);
        }
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), [-15, 15]);
    }

    #[test]
    fn trigger_bit_is_write_only() {
        let mut psg = tone(1750);
        assert_eq!(psg.read(REG_SOUND2CNT_H), 1750);
        assert_eq!(psg.read(REG_SOUND2CNT_L), 0xF080);
    }

    #[test]
    fn power_off_keeps_the_wave_ram() {
        let mut psg = tone(1750);
        psg.write(REG_WAVE_RAM, 0x1234);
        psg.power_off();
        assert_eq!(psg.status(), 0);
        assert_eq!(psg.read(REG_WAVE_RAM), 0x1234);
    }
}

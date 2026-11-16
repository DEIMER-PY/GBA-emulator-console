//! ROM de demostración propia, para tests y pantalla de prueba.

/// Programa ARM de la demo, cargado en `0x0800_00C0` (justo tras la cabecera).
///
/// ```text
///         mov   r0, #0x04000000   ; registros de E/S
///         mov   r1, #0x05000000   ; paleta
///         mov   r2, #0
///         strh  r2, [r0]          ; DISPCNT = 0: modo 0 sin fondos (solo color de fondo)
/// loop:   add   r3, r0, #0x130    ; KEYINPUT
///         ldrh  r2, [r3]
///         tst   r2, #1            ; bit 0 = botón A (0 = pulsado)
///         moveq r4, #0x001F       ; pulsado: rojo
///         movne r4, #0x7C00       ; suelto: azul
///         strh  r4, [r1]          ; color 0 de la paleta
///         b     loop
/// ```
const PROGRAM: [u32; 11] = [
    0xE3A0_0301,
    0xE3A0_1405,
    0xE3A0_2000,
    0xE1C0_20B0,
    0xE280_3F4C,
    0xE1D3_20B0,
    0xE312_0001,
    0x03A0_401F,
    0x13A0_4B1F,
    0xE1C1_40B0,
    0xEAFF_FFF8,
];

/// Tamaño de la cabecera de un cartucho de GBA.
const HEADER_SIZE: usize = 0xC0;

/// Devuelve una ROM de GBA mínima generada en memoria.
///
/// Pinta la pantalla de azul con el botón A suelto y de rojo mientras se mantiene
/// pulsado. Con ella los tests comprueban vídeo, mandos, save states y reset sin
/// depender de ninguna ROM externa.
#[must_use]
pub fn demo_rom() -> Vec<u8> {
    let mut rom = vec![0u8; HEADER_SIZE];

    // Punto de entrada: `b 0x080000C0`.
    rom[0..4].copy_from_slice(&0xEA00_002E_u32.to_le_bytes());
    rom[0xA0..0xAC].copy_from_slice(b"DEMO GBA\0\0\0\0");
    rom[0xAC..0xB0].copy_from_slice(b"ZDME");
    rom[0xB0..0xB2].copy_from_slice(b"00");
    rom[0xB2] = 0x96; // valor fijo que exige la cabecera

    // Complemento de la cabecera (0xA0..=0xBC).
    let sum = rom[0xA0..=0xBC]
        .iter()
        .fold(0u8, |acc, byte| acc.wrapping_add(*byte));
    rom[0xBD] = 0u8.wrapping_sub(sum).wrapping_sub(0x19);

    for word in PROGRAM {
        rom.extend_from_slice(&word.to_le_bytes());
    }
    rom
}

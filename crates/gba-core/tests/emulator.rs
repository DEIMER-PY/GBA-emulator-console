//! Tests headless de la API de `gba-core`, con la ROM de demostración.

use gba_core::{
    AUDIO_CHANNELS, Button, DEFAULT_SAMPLE_RATE, Emulator, Error, FRAME_BYTES, FRAME_RATE,
    ResetKind, demo_rom,
};

/// Color RGB del primer píxel del último frame.
fn first_pixel(gba: &Emulator) -> [u8; 3] {
    let frame = gba.frame();
    [frame[0], frame[1], frame[2]]
}

fn is_blue([r, g, b]: [u8; 3]) -> bool {
    b > 200 && r < 50 && g < 50
}

fn is_red([r, g, b]: [u8; 3]) -> bool {
    r > 200 && g < 50 && b < 50
}

/// Consola con la demo cargada y unos frames ya emulados.
fn booted() -> Emulator {
    let mut gba = Emulator::new();
    gba.load_rom("demo", &demo_rom()).expect("la demo carga");
    run_frames(&mut gba, 3);
    gba
}

fn run_frames(gba: &mut Emulator, frames: usize) {
    for _ in 0..frames {
        gba.run_frame().expect("el frame se emula");
    }
}

#[test]
fn sin_rom_no_se_puede_emular() {
    let mut gba = Emulator::new();
    assert!(gba.rom().is_none());
    assert!(matches!(gba.run_frame(), Err(Error::NoRom)));
    assert!(matches!(gba.save_state(), Err(Error::NoRom)));
    assert!(matches!(gba.reset(ResetKind::Hard), Err(Error::NoRom)));
    assert!(gba.battery_ram().is_none());
}

#[test]
fn una_rom_demasiado_corta_se_rechaza() {
    let mut gba = Emulator::new();
    assert!(matches!(
        gba.load_rom("rota", &[0; 16]),
        Err(Error::LoadRom(_))
    ));
    assert!(gba.rom().is_none());
}

#[test]
fn la_demo_expone_los_datos_del_cartucho() {
    let gba = booted();
    let rom = gba.rom().expect("hay ROM");
    assert_eq!(rom.name, "demo");
    assert_eq!(rom.title, "DEMO GBA");
    assert_eq!(rom.game_code, "ZDME");
    assert!(!rom.battery_backed);
}

#[test]
fn el_frame_es_rgba_opaco() {
    let gba = booted();
    let frame = gba.frame();
    assert_eq!(frame.len(), FRAME_BYTES);
    assert!(
        frame
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 0xFF)
    );
}

#[test]
fn el_boton_a_cambia_el_color_de_la_pantalla() {
    let mut gba = booted();
    assert!(is_blue(first_pixel(&gba)), "{:?}", first_pixel(&gba));

    gba.set_button(Button::A, true);
    run_frames(&mut gba, 2);
    assert!(is_red(first_pixel(&gba)), "{:?}", first_pixel(&gba));

    // Otro botón no afecta, y soltar A vuelve al azul.
    gba.set_button(Button::A, false);
    gba.set_button(Button::B, true);
    run_frames(&mut gba, 2);
    assert!(is_blue(first_pixel(&gba)), "{:?}", first_pixel(&gba));
}

#[test]
fn cada_frame_genera_su_audio_estereo() {
    let mut gba = booted();
    let mut samples = Vec::new();
    let mut total = 0;
    let frames = 60;
    for _ in 0..frames {
        gba.run_frame().expect("el frame se emula");
        gba.audio_samples(&mut samples);
        assert_eq!(samples.len() % AUDIO_CHANNELS, 0);
        assert!(samples.iter().all(|s| (-1.0..=1.0).contains(s)));
        total += samples.len() / AUDIO_CHANNELS;
    }
    // 60 frames duran 60 / FRAME_RATE segundos.
    let expected = f64::from(DEFAULT_SAMPLE_RATE) * f64::from(frames) / FRAME_RATE;
    let ratio = total as f64 / expected;
    assert!(
        (0.98..=1.02).contains(&ratio),
        "muestras: {total}, esperadas: {expected}"
    );
}

#[test]
fn la_frecuencia_de_audio_se_puede_cambiar_en_marcha() {
    let mut gba = booted();
    gba.set_sample_rate(24_000.0);
    let mut samples = Vec::new();
    let mut total = 0;
    for _ in 0..60 {
        gba.run_frame().expect("el frame se emula");
        gba.audio_samples(&mut samples);
        total += samples.len() / AUDIO_CHANNELS;
    }
    let expected = 24_000.0 * 60.0 / FRAME_RATE;
    let ratio = total as f64 / expected;
    assert!(
        (0.98..=1.02).contains(&ratio),
        "muestras: {total}, esperadas: {expected}"
    );
}

#[test]
fn los_frames_silenciosos_y_saltados_no_entregan_audio() {
    let mut gba = booted();
    let mut samples = Vec::new();

    gba.run_frame_silent().expect("el frame se emula");
    gba.audio_samples(&mut samples);
    assert!(samples.is_empty());

    // Un frame saltado avanza el juego pero conserva la imagen anterior.
    gba.set_button(Button::A, true);
    gba.skip_frame().expect("el frame se emula");
    gba.skip_frame().expect("el frame se emula");
    gba.audio_samples(&mut samples);
    assert!(samples.is_empty());
    assert!(is_blue(first_pixel(&gba)));

    gba.run_frame().expect("el frame se emula");
    assert!(is_red(first_pixel(&gba)));
}

#[test]
fn el_save_state_restaura_la_consola() {
    let mut gba = booted();
    let state = gba.save_state().expect("el estado se guarda");

    gba.set_button(Button::A, true);
    run_frames(&mut gba, 2);
    assert!(is_red(first_pixel(&gba)));

    gba.set_button(Button::A, false);
    gba.load_state(&state).expect("el estado se restaura");
    gba.run_frame().expect("el frame se emula");
    assert!(is_blue(first_pixel(&gba)));
}

#[test]
fn un_save_state_corrupto_se_rechaza() {
    let mut gba = booted();
    assert!(matches!(
        gba.load_state(&[1, 2, 3]),
        Err(Error::SaveState(_))
    ));
}

#[test]
fn el_reinicio_vuelve_a_arrancar_el_juego() {
    for kind in [ResetKind::Soft, ResetKind::Hard] {
        let mut gba = booted();
        gba.reset(kind).expect("la consola se reinicia");
        run_frames(&mut gba, 3);
        assert!(is_blue(first_pixel(&gba)), "{kind:?}");
        assert_eq!(gba.rom().expect("hay ROM").name, "demo");
    }
}

#[test]
fn la_bios_debe_tener_el_tamano_correcto() {
    let mut gba = Emulator::new();
    assert!(matches!(
        gba.set_bios(&[0; 100]),
        Err(Error::InvalidBios(100))
    ));
    gba.set_bios(&[0; gba_core::BIOS_SIZE])
        .expect("tamaño válido");
    gba.use_open_bios();
}

#[test]
fn extraer_el_cartucho_detiene_la_emulacion() {
    let mut gba = booted();
    gba.unload_rom();
    assert!(gba.rom().is_none());
    assert!(matches!(gba.run_frame(), Err(Error::NoRom)));
}

/// La demo con la marca que identifica a un cartucho con SRAM.
fn demo_rom_with_sram() -> Vec<u8> {
    let mut rom = demo_rom();
    rom.extend_from_slice(b"SRAM_V113");
    rom
}

#[test]
fn la_partida_guardada_entra_y_sale_como_bytes() {
    let mut gba = Emulator::new();
    let rom = gba
        .load_rom("demo", &demo_rom_with_sram())
        .expect("la demo carga");
    assert!(rom.battery_backed);

    let empty = gba.battery_ram().expect("el cartucho guarda partida");
    assert_eq!(empty.len(), 32 * 1024);
    assert!(empty.iter().all(|byte| *byte == 0xFF));

    let mut save = empty;
    save[..4].copy_from_slice(&[1, 2, 3, 4]);
    gba.set_battery_ram(&save);
    assert_eq!(gba.battery_ram().as_deref(), Some(save.as_slice()));

    // Ni el reinicio duro ni un save state pierden la partida.
    run_frames(&mut gba, 2);
    let state = gba.save_state().expect("el estado se guarda");
    gba.reset(ResetKind::Hard).expect("la consola se reinicia");
    assert_eq!(gba.battery_ram().as_deref(), Some(save.as_slice()));
    gba.load_state(&state).expect("el estado se restaura");
    assert_eq!(gba.battery_ram().as_deref(), Some(save.as_slice()));
}

/// Canal izquierdo de `frames` frames seguidos.
fn left_channel(gba: &mut Emulator, frames: usize) -> Vec<f32> {
    let mut samples = Vec::new();
    let mut left = Vec::new();
    for _ in 0..frames {
        gba.run_frame().expect("el frame se emula");
        gba.audio_samples(&mut samples);
        left.extend(samples.iter().step_by(AUDIO_CHANNELS));
    }
    left
}

#[test]
fn la_demo_calla_con_el_boton_suelto() {
    let mut gba = booted();
    let left = left_channel(&mut gba, 30);
    assert!(left.iter().all(|sample| sample.abs() < 0.001));
}

#[test]
fn el_boton_a_hace_sonar_un_tono_de_440_hz() {
    let mut gba = booted();
    gba.set_button(Button::A, true);
    run_frames(&mut gba, 2);

    let left = left_channel(&mut gba, 60);
    let peak = left
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    assert!((0.1..=1.0).contains(&peak), "amplitud: {peak}");

    // Frecuencia: flancos de subida por segundo.
    let edges = left
        .windows(2)
        .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
        .count();
    let seconds = left.len() as f64 / f64::from(DEFAULT_SAMPLE_RATE);
    let hertz = edges as f64 / seconds;
    assert!((430.0..=450.0).contains(&hertz), "frecuencia: {hertz} Hz");

    // Al soltar el botón vuelve el silencio.
    gba.set_button(Button::A, false);
    run_frames(&mut gba, 2);
    let left = left_channel(&mut gba, 10);
    assert!(left.iter().all(|sample| sample.abs() < 0.001));
}

// PARCHE gba-emulator-console (ver vendor/rustboyadvance-ng/PARCHES.md):
// el contenido de la memoria de guardado vive en `buffer` y se conserva al clonar y
// al serializar, de modo que funciona sin disco (web) y viaja dentro de los save
// states. El volcado a archivo sigue disponible si se da una ruta.
use std::fs::{File, OpenOptions};
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::BackupMemoryInterface;
use rustboyadvance_utils::write_bin_file;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupFile {
    size: usize,
    path: Option<PathBuf>,
    #[serde(skip)]
    file: Option<File>,
    buffer: Vec<u8>,
}

impl Clone for BackupFile {
    fn clone(&self) -> Self {
        BackupFile {
            size: self.size,
            path: self.path.clone(),
            file: self.file.as_ref().and_then(|file| file.try_clone().ok()),
            buffer: self.buffer.clone(),
        }
    }
}

impl BackupFile {
    pub fn new(size: usize, path: Option<PathBuf>) -> BackupFile {
        // TODO handle errors without unwrap
        let mut file: Option<File> = None;
        let buffer = if let Some(path) = &path {
            if !path.is_file() {
                write_bin_file(path, &vec![0xff; size]).unwrap();
            }

            let mut _file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .unwrap();

            let mut buffer = Vec::new();
            _file.read_to_end(&mut buffer).unwrap();
            buffer.resize(size, 0xff);

            file = Some(_file);

            buffer
        } else {
            vec![0xff; size]
        };

        BackupFile {
            size,
            path,
            file,
            buffer,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buffer
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Sustituye el contenido por `data`, recortando o rellenando con `0xff` hasta el
    /// tamaño de la memoria.
    pub fn load_bytes(&mut self, data: &[u8]) {
        let len = data.len().min(self.buffer.len());
        self.buffer.fill(0xff);
        self.buffer[..len].copy_from_slice(&data[..len]);
        self.flush();
    }

    pub fn flush(&mut self) {
        if let Some(file) = &mut self.file {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(&self.buffer).unwrap();
        }
    }
}

impl BackupMemoryInterface for BackupFile {
    fn write(&mut self, offset: usize, value: u8) {
        self.buffer[offset] = value;
        if let Some(file) = &mut self.file {
            file.seek(SeekFrom::Start(offset as u64)).unwrap();
            file.write_all(&[value]).unwrap();
        }
    }

    fn read(&self, offset: usize) -> u8 {
        self.buffer[offset]
    }

    fn resize(&mut self, new_size: usize) {
        self.size = new_size;
        self.buffer.resize(new_size, 0xff);
        self.flush();
    }
}

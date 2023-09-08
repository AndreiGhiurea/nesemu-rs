use std::fs;

const NES_HEADER_SIZE: usize = 0x10;
const TRAINER_SIZE: usize = 0x200;
const PRG_ROM_BLOCK_SIZE: usize = 0x4000;
const CHR_ROM_BLOCK_SIZE: usize = 0x2000;

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub has_prg_ram: bool,
    pub has_trainer: bool,
    pub has_four_screen: bool,
    pub mapper: u8,
}

#[derive(Default)]
pub enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
}

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

impl Cartridge {
    pub fn new(path: &str) -> Result<Cartridge, String> {
        let file = match fs::read(path) {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };

        let raw = &file;
        if raw[0..4] != NES_TAG {
            return Err("NES signature not found".to_string());
        }

        let mirroring = if raw[6] & 0b1 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let has_prg_ram = raw[6] & 0b10 != 0;
        let has_trainer = raw[6] & 0b100 != 0;
        let has_four_screen = raw[6] & 0b1000 != 0;

        let mapper = (raw[7] & 0xF0) | (raw[6] & 0xF0 >> 4);

        if raw[7] & 0b1100 == 0b1100 {
            return Err("Can't parse NES 2.0 roms yet".to_string());
        }

        let trainer_size = if has_trainer { 0 } else { TRAINER_SIZE };
        let prg_rom_size = raw[4] as usize * PRG_ROM_BLOCK_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_BLOCK_SIZE;

        let prg_rom_begin = NES_HEADER_SIZE + trainer_size;
        let prg_rom_end = prg_rom_begin + prg_rom_size;
        let prg_rom = raw[prg_rom_begin..prg_rom_end].to_vec();

        let chr_rom_begin = prg_rom_end;
        let chr_rom_end = chr_rom_begin + chr_rom_size;
        let chr_rom = raw[chr_rom_begin..chr_rom_end].to_vec();

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            mirroring,
            has_prg_ram,
            has_trainer,
            has_four_screen,
            mapper,
        })
    }
}

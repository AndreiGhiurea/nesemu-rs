use nom::bytes::streaming::{tag, take};
use nom::combinator::map;
use nom::error::Error;
use nom::number::streaming::be_u8;
use nom::sequence::tuple;
use nom::IResult;

use std::{cell::RefCell, fs, rc::Rc};

use super::bus::Bus;

pub struct Cartridge {
    bus: Rc<RefCell<Bus>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
enum TvSystem {
    #[default]
    NTSC,
    PAL,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct NesHeader {
    signature: [u8; 3],
    prg_rom_size: u8,
    chr_rom_size: u8,
    mirroring: Mirroring,
    has_battery_prg_ram: bool,
    has_trainer: bool,
    ignore_mirroring: bool,
    mapper: u8,
    vs_unisystem: bool,
    has_playchoice: bool,
    is_nes_2_format: bool,
    prg_ram_size: u8,
    tv_system: TvSystem,
    padding: [u8; 5],
}

impl NesHeader {
    fn parse(bytes: &[u8]) -> IResult<&[u8], NesHeader> {
        map(
            tuple((
                tag("NES"),
                be_u8,
                be_u8,
                be_u8,
                be_u8,
                be_u8,
                be_u8,
                be_u8,
                be_u8,
                take(5usize),
            )),
            |(sig, _eof, prg, chr, flg6, flg7, flg8, flg9, _flg10, pad)| NesHeader {
                signature: Vec::from(sig).try_into().unwrap(),
                prg_rom_size: prg,
                chr_rom_size: chr,
                mirroring: if flg6 & 1 != 0x0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                },
                has_battery_prg_ram: flg6 & 2 != 0x0,
                has_trainer: flg6 & 4 != 0x0,
                ignore_mirroring: flg6 & 8 != 0x0,
                mapper: (flg7 & 0xF0) | ((flg6 & 0xF0) >> 4),
                vs_unisystem: flg7 & 1 != 0x0,
                has_playchoice: flg7 & 2 != 0x0,
                is_nes_2_format: flg7 & 0xC == 0xC,
                prg_ram_size: flg8,
                tv_system: if flg9 & 1 != 0x0 {
                    TvSystem::PAL
                } else {
                    TvSystem::NTSC
                },
                padding: Vec::from(pad).try_into().unwrap(),
            },
        )(bytes)
    }
}

const TRAINER_SIZE: usize = 0x200;
const PRG_ROM_BLOCK_SIZE: usize = 0x4000;
const CHR_ROM_BLOCK_SIZE: usize = 0x2000;
const PLAYCHOICE_INST_ROM_SIZE: usize = 0x2000;
const _PLAYCHOICE_PROM_SIZE: usize = 0x20;

#[derive(Debug, Default)]
struct NesImage<'a> {
    header: NesHeader,
    trainer: Option<Box<&'a [u8]>>,
    prg_rom: Box<&'a [u8]>,
    chr_rom: Box<&'a [u8]>,
    playchoice_inst_rom: Option<Box<&'a [u8]>>,
}

impl<'a> NesImage<'a> {
    fn parse_trainer(
        header: &NesHeader,
        bytes: &'a [u8],
    ) -> IResult<&'a [u8], Option<Box<&'a [u8]>>> {
        if header.has_trainer {
            let (bytes, trainer) = take(TRAINER_SIZE)(bytes)?;
            Ok((bytes, Some(Box::new(trainer))))
        } else {
            Ok((bytes, None))
        }
    }

    fn parse_playchoice_inst_rom(
        header: &NesHeader,
        bytes: &'a [u8],
    ) -> IResult<&'a [u8], Option<Box<&'a [u8]>>> {
        if header.has_playchoice {
            let (bytes, playchoice_inst) = take(PLAYCHOICE_INST_ROM_SIZE)(bytes)?;
            Ok((bytes, Some(Box::new(playchoice_inst))))
        } else {
            Ok((bytes, None))
        }
    }

    fn parse(bytes: &'a [u8]) -> Result<NesImage<'a>, nom::Err<Error<&[u8]>>> {
        let (bytes, header) = NesHeader::parse(bytes)?;

        // Trainer is optional, so we have a function that can return None if needed.
        let (bytes, trainer) = NesImage::parse_trainer(&header, bytes)?;

        let prg_rom_size = PRG_ROM_BLOCK_SIZE * header.prg_rom_size as usize;
        let (bytes, prg_rom) = take(prg_rom_size)(bytes)?;
        let prg_rom = Box::new(prg_rom);

        let chr_rom_size = CHR_ROM_BLOCK_SIZE * header.chr_rom_size as usize;
        let (bytes, chr_rom) = take(chr_rom_size)(bytes)?;
        let chr_rom = Box::new(chr_rom);

        let (_bytes, playchoice_inst_rom) = NesImage::parse_playchoice_inst_rom(&header, bytes)?;

        //
        // Playchoice PROM parsing?
        //

        //
        // Parse further
        // Here we could handle the error and send up a more specific NesParserError
        // since this parsing bit might go into it's own file.
        // We could have like, NesHeaderParsingFailed or something.
        //

        Ok(NesImage {
            header,
            trainer,
            prg_rom,
            chr_rom,
            playchoice_inst_rom,
        })
    }
}

impl Cartridge {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Cartridge {
        Cartridge { bus }
    }

    pub fn load(&mut self, path: &str) {
        let bytes = fs::read(path).expect(format!("Cannot fine file: {}", path).as_str());

        let result = NesImage::parse(bytes.as_slice());

        let image = match result {
            Ok(img) => img,
            //
            // These nom error are kinda clunky and don't say much.
            // We should handle them here at least and send up an easy error like CartridgeLoadFailed.
            // Might be even better if we have NesParser errors here.
            //
            // We'll see how this Cartridge development goes.
            // If it gets complicated I might move the parser out and send parser errors here,
            // which will be sent out or simplified further.
            //
            Err(nom::Err::Error(err)) => {
                println!("Couldn't parse {path}, got error code: {:?}", err.code);
                return;
            }
            _ => {
                println!("Couldn't parse {path}, parsing either failed or is incomplete.");
                return;
            }
        };

        let mut load_addr: usize = 0x8000;
        for byte in *image.prg_rom {
            self.bus.borrow_mut().write_u8(load_addr as u16, *byte);
            load_addr += 0x1;
        }

        load_addr = 0xC000;
        for byte in *image.prg_rom {
            self.bus.borrow_mut().write_u8(load_addr as u16, *byte);
            load_addr += 0x1;
        }
    }
}

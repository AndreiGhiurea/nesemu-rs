use nom::bytes::streaming::{tag, take};
use nom::combinator::{flat_map, map, map_res};
use nom::error::{Error, ErrorKind};
use nom::multi::{length_data, many0, many_m_n};
use nom::number::streaming::{be_f64, be_i16, be_i24, be_u16, be_u24, be_u32, be_u8};
use nom::sequence::{pair, terminated, tuple};
use nom::{Err, IResult, Needed};

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

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct NesHeader {
    signature: [u8; 3],
    prg_size: u8,
    chr_size: u8,
    mirroring: Mirroring,
    battery_prg_ram: bool,
    trainer: bool,
    ignore_mirroring: bool,
    mapper: u8,
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
            |(sig, _eof, prg, chr, flg6, flg7, flg8, flg9, flg10, pad)| NesHeader {
                signature: Vec::from(sig).try_into().unwrap(),
                prg_size: prg,
                chr_size: chr,
                mirroring: if flg6 & 1 != 0x0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                },
                battery_prg_ram: flg6 & 2 != 0x0,
                trainer: flg6 & 4 != 0x0,
                ignore_mirroring: flg6 & 8 != 0x0,
                mapper: (flg6 & 0xF0) >> 4,
                padding: Vec::from(pad).try_into().unwrap(),
            },
        )(bytes)
    }
}

#[derive(Debug)]
struct NesImage {
    header: NesHeader,
}

impl NesImage {
    fn parse(bytes: &[u8]) -> Result<NesImage, nom::Err<Error<&[u8]>>> {
        let (bytes, header) = NesHeader::parse(bytes)?;

        //
        // Parse further
        // Here we could handle the error and send up a more specific NesParserError
        // since this parsing bit might go into it's own file.
        // We could have like, NesHeaderParsingFailed or something.
        //

        Ok(NesImage { header })
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

        println!("{:?}", image);

        /*
        let prg_rom = &bytes[16..][..0x4000];

        let mut load_addr: u16 = 0x8000;
        for byte in prg_rom {
            self.bus.borrow_mut().write_u8(load_addr, *byte);
            load_addr += 0x1;
        }

        load_addr = 0xC000 - 0x1;
        for byte in prg_rom {
            load_addr += 0x1;
            self.bus.borrow_mut().write_u8(load_addr, *byte);
        }
        */
    }
}

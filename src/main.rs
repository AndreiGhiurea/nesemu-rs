mod nes;
use nes::Nes;

fn main() {
    let rom_path = "games/donkey_kong.nes";

    let mut nes = match Nes::new(rom_path) {
        Ok(nes) => nes,
        Err(e) => {
            eprintln!("nesemu failed: {e}");
            return;
        }
    };

    nes.run();
}

mod nes;
use nes::Nes;

fn main() {
    let mut nes = Nes::new();
    nes.run();
}

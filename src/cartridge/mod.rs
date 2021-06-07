pub struct Cartridge {
    pub rom: Vec<u8>,
    pub bytes_read: usize
}

impl Cartridge {
    pub fn read(filename: &str) -> Cartridge {
        let bytes = match std::fs::read(filename) {
            Ok(b) => b,
            Err(e) => panic!("{}", e)
        };

        Cartridge {
            rom: bytes.clone(),
            bytes_read: bytes.len()
        }
    }
}
pub struct ProcessorState {
    pub vram: [[u8; 64]; 32],
    pub vram_changed: bool,
    pub beep: bool
}
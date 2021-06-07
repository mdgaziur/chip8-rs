mod processor;
mod font;
mod cartridge;
mod output;
mod audio;
mod display;
mod input;

fn main() {
    let sleep_duration = std::time::Duration::from_millis(2);

    let sdl_context = sdl2::init().unwrap();
    let args: Vec<String> = std::env::args().collect();
    let cartridge_filename = &args[1];

    let audio_driver = audio::Audio::new(&sdl_context);
    let cartridge_driver = cartridge::Cartridge::read(&cartridge_filename);
    let mut display_driver = display::DisplayDriver::new(&sdl_context);
    let mut input_driver = input::InputDriver::new(&sdl_context);
    let mut processor = processor::Processor::new();

    processor.load_program(cartridge_driver.rom);

    while let Ok(keypad) = input_driver.poll() {
        let output = processor.tick(keypad);

        if output.vram_changed {
            display_driver.draw(&output.vram);
        }

        if output.beep {
            audio_driver.start_beep();
        }
        else {
            audio_driver.stop_beep();
        }

        std::thread::sleep(sleep_duration);
    }
}
use std::time::Duration;

fn main() {
    println!("Attempting to open audio device...");
    let (_stream, handle) = match rodio::OutputStream::try_default() {
        Ok(v) => {
            println!("Audio device opened OK");
            v
        }
        Err(e) => {
            println!("FAILED to open audio: {}", e);
            return;
        }
    };

    let sink = rodio::Sink::try_new(&handle).unwrap();
    sink.set_volume(1.0);

    println!("Playing 440Hz sine wave for 3 seconds...");
    let source = rodio::source::SineWave::new(440.0);
    sink.append(source);

    std::thread::sleep(Duration::from_secs(3));
    println!("Done. Did you hear a tone?");
}

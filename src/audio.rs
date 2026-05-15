use rodio::Player;
use rodio::buffer::SamplesBuffer;
use rodio::stream::DeviceSinkBuilder;
use std::num::{NonZeroU16, NonZeroU32};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub struct AudioPlayerConfig {
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct AudioPlayer {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl AudioPlayer {
    pub fn new(
        config: AudioPlayerConfig,
        samples_rx: Receiver<Vec<f32>>,
        running: Arc<AtomicBool>,
    ) -> Option<Self> {
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let mut sink = match DeviceSinkBuilder::open_default_sink() {
                Ok(s) => s,
                Err(_) => return,
            };
            sink.log_on_drop(false);

            let channels = match NonZeroU16::new(config.channels) {
                Some(c) => c,
                None => return,
            };
            let sample_rate = match NonZeroU32::new(config.sample_rate) {
                Some(r) => r,
                None => return,
            };

            let player = Player::connect_new(sink.mixer());

            while let Ok(samples) = samples_rx.recv() {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }

                if samples.is_empty() {
                    continue;
                }

                let source = SamplesBuffer::new(channels, sample_rate, samples);
                player.append(source);
            }

            while !player.empty() && running_clone.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
        });

        Some(Self {
            running,
            handle: Some(handle),
        })
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

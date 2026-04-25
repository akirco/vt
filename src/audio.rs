use rodio::{OutputStream, Sink};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
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
    /// # 参数
    /// - config: 音频配置（采样率和声道数）
    /// - samples_rx: 接收解码后音频样本的channel
    /// - running: 共享的运行状态标志
    pub fn new(
        config: AudioPlayerConfig,
        samples_rx: Receiver<Vec<f32>>,
        running: Arc<AtomicBool>,
    ) -> Option<Self> {
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(_) => return,
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => s,
                Err(_) => return,
            };

            while let Ok(samples) = samples_rx.recv() {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }

                if samples.is_empty() {
                    continue;
                }

                let source =
                    rodio::buffer::SamplesBuffer::new(config.channels, config.sample_rate, samples);
                sink.append(source);
            }

            while !sink.empty() && running_clone.load(Ordering::SeqCst) {
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

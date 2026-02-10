use crossterm::event::{self, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Render,
    LlmResponse(String, Option<crate::llm_client::Usage>, f32),  // Added elapsed time in seconds
    LlmError(String),
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        // Spawn keyboard input thread
        let key_tx = tx.clone();
        thread::spawn(move || loop {
            if let Ok(event) = event::read() {
                if let crossterm::event::Event::Key(key) = event {
                    if key_tx.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn tick thread (250ms interval)
        let tick_tx = tx.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(250));
            if tick_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        });

        // Spawn render thread (16ms = 60 FPS)
        let render_tx = tx.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(16));
            if render_tx.send(AppEvent::Render).is_err() {
                break;
            }
        });

        EventHandler { rx, tx }
    }

    pub fn next(&self) -> Option<AppEvent> {
        self.rx.recv().ok()
    }

    pub fn sender(&self) -> mpsc::Sender<AppEvent> {
        self.tx.clone()
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

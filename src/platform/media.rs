/// Media hooks for deterministic playback control in tests

#[derive(Debug, Clone, PartialEq)]
pub enum MediaState {
    Playing,
    Paused,
    Ended,
}

pub trait MediaHooks: Send + Sync {
    fn play(&self);
    fn pause(&self);
    fn seek(&self, seconds: f64);
    fn state(&self) -> MediaState;
}

/// Noop implementation that keeps state in-memory for tests
pub struct NoopMediaHooks {
    state: std::sync::Mutex<MediaState>,
}

impl NoopMediaHooks {
    pub fn new() -> Self {
        NoopMediaHooks { state: std::sync::Mutex::new(MediaState::Paused) }
    }
}

impl MediaHooks for NoopMediaHooks {
    fn play(&self) {
        let mut s = self.state.lock().unwrap();
        *s = MediaState::Playing;
    }

    fn pause(&self) {
        let mut s = self.state.lock().unwrap();
        *s = MediaState::Paused;
    }

    fn seek(&self, _seconds: f64) {
        // For the prototype, just leave it as-is; tests can assert transitions
    }

    fn state(&self) -> MediaState {
        self.state.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_media_transitions_state() {
        let m = NoopMediaHooks::new();
        assert_eq!(m.state(), MediaState::Paused);
        m.play();
        assert_eq!(m.state(), MediaState::Playing);
        m.pause();
        assert_eq!(m.state(), MediaState::Paused);
    }
}

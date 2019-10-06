use std::time::*;

pub struct Timer {
    last: Option<Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            last: None,
            duration: None
        }
    }

    pub fn start(&mut self) {
        self.last = Some(Instant::now())
    }

    pub fn stop(&mut self) -> Duration {
        if let Some(last) = self.last {
            self.last = None;
            last.elapsed();
        }
        else {
            panic!("Timer.stop was called before Timer.start");
        }
    }

    pub fn duration(&self) -> Duration {
        if let Some(last) = self.last {
            last.elapsed();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn expected_duration() {
        let mut timer = Timer::new();

        timer.start();
        
        let ten_millis = Duration::from_millis(10);
        std::thread::sleep(ten_millis);

        let duration = timer.stop();
        
        assert!(duration >= ten_millis);
    }
}

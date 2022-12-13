use std::sync::{Mutex, Arc};
use std::time::{Instant, Duration};

//use chrono::{DateTime, Utc};

/// A single bit of profiler information. You spawn them as needed, which will
/// immediately start the timer. Then you can call "complete" on it to finalize
/// it. Alternatively, you can simply craft one yourself with a pre-existing time,
/// but your start time will be empty
#[derive(Debug, Clone, PartialEq)]
pub struct TimerProfile
{
    pub name: String,
    pub timer: Option<Instant>, //DateTime<Utc>>,
    pub duration: Duration //milliseconds are mostly what I care about, YMMV
}

impl TimerProfile {
    /// Create a timer profile from an existing 
    pub fn from_existing(name: String, duration: Duration) -> Self {
        Self {
            name,
            duration,
            timer: None
        }
    }

    /// Create AND START a timer profile
    pub fn new(name: String) -> Self {
        Self {
            name,
            duration: Duration::from_secs(0),
            timer: Some(Instant::now())
        }
    }

    /// Complete the given timer. Returns if the duration was updated or not (if you
    /// have no timer set, this won't update anything)
    pub fn complete(&mut self) -> bool {
        if let Some(timer) = self.timer {
            self.duration = timer.elapsed();
            true
        }
        else {
            false
        }
    }
}

/// A profiler which tracks a list of timed sections of code. Threadsafe
#[derive(Debug, Clone)]
pub struct Profiler {
    //Can do auto-clone because it's just arc?
    pub profiles: Arc<Mutex<Vec<TimerProfile>>>
}

impl Profiler {
    /// Create a brand new profiler, with its own profile list
    pub fn new() -> Self {
        Self { 
            profiles: Arc::new(Mutex::new(Vec::new()))
        }
    }

    /// Add a timer profile in a thread-safe manner. Should be a completed timer!
    pub fn add(&mut self, timer: TimerProfile) {
        let mut profiles = self.profiles.lock().unwrap();
        profiles.push(timer)
    }

    /// Get a FULL COPY of all existing timer profiles saved in this instance. 
    pub fn list_copy(&self) -> Vec<TimerProfile> {
        let profiles = self.profiles.lock().unwrap();
        profiles.iter().map(|p| p.clone()).collect()
    }
}

//impl Clone for Profiler {
//    fn clone(&self) -> Self {
//        Self {
//            profiles: self.profiles.clone()
//        }
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_updates() {
        let mut profiler = Profiler::new();
        let mut other = profiler.clone();
        profiler.add(TimerProfile::from_existing(String::from("hello"), Duration::from_micros(10)));
        other.add(TimerProfile::from_existing(String::from("wow"), Duration::from_nanos(99)));

        let vec1 = profiler.list_copy();
        let vec2 = other.list_copy();

        assert_eq!(vec1.len(), vec2.len());
        assert_eq!(vec1.len(), 2);
        assert_eq!(vec1.get(0), vec2.get(0));
        assert_eq!(vec1.get(1), vec2.get(1));
    }
}

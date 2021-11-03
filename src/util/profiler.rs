use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct NamedSample {
    completed: Vec<Duration>,
    in_progress: Option<Instant>,
}

impl NamedSample {
    pub fn get_samples(&self) -> &Vec<Duration> {
        if cfg!(debug_assertions) && self.in_progress.is_some() {
            println!("Perf Warn: getting completed samples while perf sample is in progress");
        }
        &self.completed
    }
}

struct Samples(Mutex<HashMap<&'static str, NamedSample>>);

thread_local! {
    static SAMPLES: Samples = Samples(Mutex::new(HashMap::new()));
}

pub fn profile_scope(name: &'static str) -> ScopeSampler {
    SAMPLES.with(|samples| {
        samples.start(name);
    });
    ScopeSampler { name }
}

pub fn take_profile_data() -> HashMap<&'static str, NamedSample> {
    SAMPLES.with(|samples| {
        let mut guard = samples.0.lock().unwrap();
        let mut result = HashMap::new();
        std::mem::swap(&mut result, &mut guard);

        result
    })
}

pub struct ScopeSampler {
    name: &'static str,
}

impl ScopeSampler {
    pub fn end(self) {
        //Our implementation of drop takes care of ending the profile
        let _ = self;
    }
}

impl Drop for ScopeSampler {
    fn drop(&mut self) {
        SAMPLES.with(|samples| {
            samples.end(self.name);
        });
    }
}

impl Samples {
    fn end(&self, name: &'static str) {
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        let end = Instant::now();
        let mut guard = self.0.lock().unwrap();
        let sample = match guard.get_mut(name) {
            Some(vec) => vec,
            None => panic!("No matching scope name to end for: {}", name),
        };
        let start = sample
            .in_progress
            .take()
            .unwrap_or_else(|| panic!("No sample started!"));

        sample.completed.push(end - start);
    }

    fn start(&self, name: &'static str) {
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        let start = Instant::now();

        let mut guard = self.0.lock().unwrap();
        let sample = guard.entry(name).or_insert(NamedSample {
            completed: Vec::new(),
            in_progress: None,
        });

        if sample.in_progress.is_some() {
            panic!("Sample already in progress! End must be called first");
        }
        sample.in_progress = Some(start);
    }
}

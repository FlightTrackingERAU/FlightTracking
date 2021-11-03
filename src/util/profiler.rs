use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// A group of associated samples that correspond with the length of an operation
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

/// Begins a sampling scope that associates the length of time `ScopeSampler` is alive to name
/// `name`.
///
/// Once `ScopeSampler` is dropped, there will be an entry name in [`take_profile_data`] with the
/// correspond duration `ScopeSampler` was alive for. Can be repeated to add more samples to `name`.
pub fn profile_scope(name: &'static str) -> ScopeSampler {
    SAMPLES.with(|samples| {
        samples.start(name);
    });
    ScopeSampler { name }
}

/// Takes the map associating named scopes to their duration, leaving an empty map.
///
/// To gather profiling data each frame, this function should be called once per frame, with
/// [`profile_scope`] being called many times for each bit of code that should be sampled.
pub fn take_profile_data() -> HashMap<&'static str, NamedSample> {
    SAMPLES.with(|samples| {
        let mut result = HashMap::new();
        {
            let mut guard = samples.0.lock().unwrap();
            std::mem::swap(&mut result, &mut guard);
        }

        result
    })
}

/// A kind of profiling guard that captures the length `self` is alive for
pub struct ScopeSampler {
    name: &'static str,
}

impl ScopeSampler {
    /// Ends this sample.
    ///
    /// Calling this function is equivalent to dropping `self`
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

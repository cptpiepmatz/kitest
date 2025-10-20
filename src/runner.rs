use rayon::{
    ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder,
    iter::{ParallelBridge, ParallelIterator},
};

pub trait TestRunner<Extra> {
    fn run<I, F>(&self, tests: I)
    where
        I: ExactSizeIterator<Item = F> + Send,
        F: (Fn()) + Send;
}

#[derive(Default)]
pub struct SimpleRunner;

impl<Extra> TestRunner<Extra> for SimpleRunner {
    fn run<I, F>(&self, tests: I) where I: ExactSizeIterator<Item = F> + Send, F: Fn() + Send {
        for test in tests {
            test();
        }
    }
}

pub struct DefaultRunner {
    thread_pool: Option<ThreadPool>,
}

impl DefaultRunner {
    pub fn new() -> Self {
        Self { thread_pool: None }
    }

    pub fn with_threads(self, threads: usize) -> Result<Self, ThreadPoolBuildError> {
        let thread_pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
        Ok(Self { thread_pool: Some(thread_pool)})
    }

    pub fn with_thread_pool(self, thread_pool: ThreadPool) -> Self {
        Self {
            thread_pool: Some(thread_pool),
        }
    }
}

impl Default for DefaultRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl<Extra> TestRunner<Extra> for DefaultRunner {
    fn run<I, F>(&self, tests: I)
    where
        I: ExactSizeIterator<Item = F> + Send,
        F: Fn() + Send,
    {
        match &self.thread_pool {
            Some(thread_pool) => thread_pool.install(|| {
                tests.par_bridge().for_each(|f| f())
            }),
            None => tests.par_bridge().for_each(|f| f()),
        }
    }
}

pub struct SmartRunner {
    threshold: usize,
    simple: SimpleRunner,
    default: DefaultRunner,
}

impl SmartRunner {
    pub fn new() -> Self {
        Self {
            threshold: 4,
            simple: SimpleRunner,
            default: DefaultRunner { thread_pool: None }
        }
    }

    pub fn with_threshold(self, threshold: usize) -> Self {
        Self {
            threshold,
            ..self
        }
    }

    pub fn with_threads(self, threads: usize) -> Result<Self, ThreadPoolBuildError> {
        let thread_pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
        Ok(Self {
            default: DefaultRunner { thread_pool: Some(thread_pool) },
            ..self
        })
    }

    pub fn with_thread_pool(self, thread_pool: ThreadPool) -> Self {
        Self {
            default: DefaultRunner { thread_pool: Some(thread_pool) },
            ..self
        }
    }
}

impl Default for SmartRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl<Extra> TestRunner<Extra> for SmartRunner {
    fn run<I, F>(&self, tests: I)
    where
        I: ExactSizeIterator<Item = F> + Send,
        F: Fn() + Send {
        match tests.len() <= self.threshold {
            true => <SimpleRunner as TestRunner<Extra>>::run(&self.simple, tests),
            false => <DefaultRunner as TestRunner<Extra>>::run(&self.default, tests),
        }
    }
}

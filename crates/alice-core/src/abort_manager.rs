pub type AbortHandler = Box<dyn Fn(String) + Send + Sync>;

pub struct AbortManager {
    handlers: Vec<AbortHandler>,
    abort_reason: Option<String>,
}

impl Default for AbortManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AbortManager {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            abort_reason: None,
        }
    }

    pub fn is_aborted(&self) -> bool {
        self.abort_reason.is_some()
    }

    pub fn abort_reason(&self) -> Option<&str> {
        self.abort_reason.as_deref()
    }

    pub fn abort(&mut self, reason: String) {
        self.abort_reason = Some(reason.clone());
        for handler in self.handlers.iter() {
            handler(reason.clone());
        }
    }

    pub fn on_abort<F>(&mut self, handler: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
}

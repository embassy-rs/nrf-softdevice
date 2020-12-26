pub struct OnDrop<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self { f: Some(f) }
    }

    pub fn defuse(mut self) {
        self.f = None
        // drop
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f()
        }
    }
}

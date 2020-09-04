pub struct DropBomb {
    defused: bool,
}

impl DropBomb {
    pub fn new() -> Self {
        Self { defused: false }
    }

    pub fn defuse(&mut self) {
        self.defused = true;
    }
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        if !self.defused {
            depanic!("boom")
        }
    }
}

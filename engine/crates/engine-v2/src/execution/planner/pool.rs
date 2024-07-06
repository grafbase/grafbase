pub(super) struct BufferPool<T> {
    buffers: Vec<Vec<T>>,
}

impl<T> Default for BufferPool<T> {
    fn default() -> Self {
        Self { buffers: Vec::new() }
    }
}

impl<T> BufferPool<T> {
    pub fn pop(&mut self) -> Vec<T> {
        self.buffers.pop().unwrap_or_default()
    }

    pub fn push(&mut self, mut buffer: Vec<T>) {
        buffer.clear();
        self.buffers.push(buffer);
    }
}

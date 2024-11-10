pub trait Writer {
    fn write(&mut self, byte: u8);
}

impl Writer for Vec<u8> {
    fn write(&mut self, byte: u8) {
        self.push(byte);
    }
}

pub struct CountingWriter {
    num_bytes: usize,
}

impl CountingWriter {
    pub fn new() -> Self {
        return CountingWriter { num_bytes: 0 };
    }

    pub fn get_num_bytes_written(&self) -> usize {
        return self.num_bytes;
    }
}

impl Writer for CountingWriter {
    #[allow(unused_variables)]
    fn write(&mut self, byte: u8) {
        self.num_bytes += 1;
    }
}

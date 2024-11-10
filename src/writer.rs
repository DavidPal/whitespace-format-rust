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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counting_writer() {
        let mut writer = CountingWriter::new();
        assert_eq!(writer.get_num_bytes_written(), 0);
        writer.write(42);
        assert_eq!(writer.get_num_bytes_written(), 1);
        writer.write(7);
        assert_eq!(writer.get_num_bytes_written(), 2);
        writer.write(13);
        assert_eq!(writer.get_num_bytes_written(), 3);
    }

    #[test]
    fn test_vector_writer() {
        let mut writer: Vec<u8> = Vec::new();
        writer.write(42);
        writer.write(7);
        writer.write(13);
        assert_eq!(writer, vec![42, 7, 13]);
    }
}

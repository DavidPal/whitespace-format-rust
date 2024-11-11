/// Writer is an abstraction of an output buffer
/// that can rewind back to a previous position.
pub trait Writer {
    /// Writes a single byte to the output
    /// and implicitly advances the position by 1.
    fn write(&mut self, byte: u8);

    // Writes multiple bytes to the output
    // and implicitly advances the position by the number of bytes written.
    fn write_bytes(&mut self, bytes: &[u8]);

    // Rewinds to a previous position.
    fn rewind(&mut self, previous_position: usize);

    /// Gets the current position.
    fn position(&self) -> usize;
}

impl Writer for Vec<u8> {
    fn write(&mut self, byte: u8) {
        self.push(byte);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn rewind(&mut self, previous_position: usize) {
        self.truncate(previous_position);
    }

    fn position(&self) -> usize {
        return self.len();
    }
}

/// Writer that only counts the number of bytes written.
/// The bytes are written to /dev/null.
#[derive(Debug)]
pub struct CountingWriter {
    position: usize,
}

impl CountingWriter {
    pub fn new() -> Self {
        return CountingWriter { position: 0 };
    }
}

impl Writer for CountingWriter {
    #[allow(unused_variables)]
    fn write(&mut self, byte: u8) {
        self.position += 1;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.position += bytes.len();
    }

    fn rewind(&mut self, previous_position: usize) {
        self.position = previous_position;
    }

    fn position(&self) -> usize {
        return self.position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counting_writer() {
        let mut writer = CountingWriter::new();
        assert_eq!(writer.position(), 0);
        writer.write(42);
        assert_eq!(writer.position(), 1);
        writer.write(7);
        assert_eq!(writer.position(), 2);
        writer.write(13);
        assert_eq!(writer.position(), 3);
        writer.rewind(2);
        assert_eq!(writer.position(), 2);
    }

    #[test]
    fn test_vector_writer() {
        let mut writer: Vec<u8> = Vec::new();
        assert_eq!(writer.position(), 0);
        writer.write(42);
        assert_eq!(writer.position(), 1);
        writer.write(7);
        assert_eq!(writer.position(), 2);
        writer.write(13);
        assert_eq!(writer, vec![42, 7, 13]);
        writer.rewind(2);
        assert_eq!(writer.position(), 2);
        assert_eq!(writer, vec![42, 7]);
    }
}

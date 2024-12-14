use std::cmp::max;

/// Writer is an abstraction of an output buffer
/// that can rewind back to a previous position.
pub trait Writer {
    /// Writes a single byte to the output
    /// and implicitly advances the position by 1.
    fn write(&mut self, byte: u8);

    /// Writes multiple bytes to the output
    /// and implicitly advances the position by the number of bytes written.
    fn write_bytes(&mut self, bytes: &[u8]);

    /// Rewinds to a previous position.
    fn rewind(&mut self, previous_position: usize);

    /// Gets the current position.
    fn position(&self) -> usize;
}

/// Implementation of the Writer trait for a standard vector.
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
        self.len()
    }
}

/// Writer that only counts the number of bytes written.
/// The bytes are written to /dev/null.
#[derive(Debug)]
pub struct CountingWriter {
    /// Number of bytes currently stored in the "buffer".
    position: usize,

    /// Maximum number of bytes that "buffer" ever contained.
    maximum_position: usize,
}

impl CountingWriter {
    /// Factory method.
    pub fn new() -> Self {
        CountingWriter {
            position: 0,
            maximum_position: 0,
        }
    }

    /// Getter.
    pub fn maximum_position(&self) -> usize {
        self.maximum_position
    }
}

impl Writer for CountingWriter {
    #[allow(unused_variables)]
    fn write(&mut self, byte: u8) {
        self.position += 1;
        self.maximum_position = max(self.maximum_position, self.position);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.position += bytes.len();
        self.maximum_position = max(self.maximum_position, self.position);
    }

    fn rewind(&mut self, previous_position: usize) {
        self.position = previous_position;
        self.maximum_position = max(self.maximum_position, self.position);
    }

    fn position(&self) -> usize {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counting_writer() {
        let mut writer = CountingWriter::new();
        assert_eq!(writer.position(), 0);
        assert_eq!(writer.maximum_position(), 0);
        writer.write(42);
        assert_eq!(writer.position(), 1);
        assert_eq!(writer.maximum_position(), 1);
        writer.write(7);
        assert_eq!(writer.position(), 2);
        assert_eq!(writer.maximum_position(), 2);
        writer.write(13);
        assert_eq!(writer.position(), 3);
        assert_eq!(writer.maximum_position(), 3);
        writer.rewind(2);
        assert_eq!(writer.position(), 2);
        assert_eq!(writer.maximum_position(), 3);
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

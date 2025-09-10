use std::io::Read;

pub struct ChunkReader {
    data: Vec<u8>,
    num_bytes_per_read: usize,
    pos: usize,
}

#[allow(dead_code)]
impl ChunkReader {
    pub fn new(data: &str, num_bytes_per_read: usize) -> Self {
        Self {
            data: data.as_bytes().to_vec(),
            num_bytes_per_read,
            pos: 0,
        }
    }
}

impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }

        let end = (self.pos + self.num_bytes_per_read).min(self.data.len());
        let chunk = &self.data[self.pos..end];

        let n = chunk.len().min(buf.len());
        buf[..n].copy_from_slice(&chunk[..n]);

        self.pos += n;
        Ok(n)
    }
}

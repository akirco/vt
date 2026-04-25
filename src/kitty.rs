use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::io::{self, Write};

pub struct KittyEncoder {
    frame_id: u32,
    b64_buffer: String,
}

impl KittyEncoder {
    pub fn new() -> Self {
        Self {
            frame_id: 0,
            b64_buffer: String::new(),
        }
    }

    pub fn encode_frame<W: Write>(
        &mut self,
        writer: &mut W,
        width: usize,
        height: usize,
        rgb_data: &[u8],
    ) -> io::Result<()> {
        if width == 0 || height == 0 || rgb_data.is_empty() {
            return Ok(());
        }

        self.frame_id += 1;
        let frame_id = self.frame_id;

        write!(writer, "\x1b[H")?;

        self.b64_buffer.clear();
        BASE64.encode_string(rgb_data, &mut self.b64_buffer);

        let chunks = self.b64_buffer.as_bytes().chunks(4096);
        let num_chunks = chunks.len();

        for (i, chunk) in chunks.enumerate() {
            let m = if i == num_chunks - 1 { 0 } else { 1 };
            if i == 0 {
                write!(
                    writer,
                    "\x1b_Ga=T,f=24,s={},v={},i={},p=1,q=1,m={};",
                    width, height, frame_id, m
                )?;
            } else {
                write!(writer, "\x1b_Gm={};", m)?;
            }
            writer.write_all(chunk)?;
            write!(writer, "\x1b\\")?;
        }

        Ok(())
    }
}

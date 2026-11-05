use anyhow::{bail, Context, Result};
use std::io::{Read, Write};

/// Read one native-messaging frame. Returns `Ok(None)` on clean EOF before a frame.
pub fn read_message(reader: &mut impl Read) -> Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e).context("read length"),
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    // Chrome caps native messages at 1 MiB.
    if len > 1024 * 1024 {
        bail!("native message too large: {len} bytes");
    }
    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .context("read native message body")?;
    Ok(Some(buf))
}

pub fn write_message(writer: &mut impl Write, body: &[u8]) -> Result<()> {
    if body.len() > 1024 * 1024 {
        bail!("native message too large to send: {} bytes", body.len());
    }
    let len = (body.len() as u32).to_le_bytes();
    writer.write_all(&len).context("write length")?;
    writer.write_all(body).context("write body")?;
    writer.flush().context("flush")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip() {
        let payload = br#"{"id":"1","method":"ping"}"#;
        let mut buf = Vec::new();
        write_message(&mut buf, payload).unwrap();
        let mut cur = Cursor::new(buf);
        let got = read_message(&mut cur).unwrap().unwrap();
        assert_eq!(got, payload);
    }
}

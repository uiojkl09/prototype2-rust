use anyhow::{Context, Result, ensure};

pub struct Cursor<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}
impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).context("byte range overflow")?;
        let bytes = self.data.get(self.pos..end).context("truncated data")?;
        self.pos = end;
        Ok(bytes)
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    pub fn f32(&mut self) -> Result<f32> {
        let n = f32::from_le_bytes(self.take(4)?.try_into()?);
        ensure!(n.is_finite(), "non-finite float");
        Ok(n)
    }
    pub fn vec3(&mut self) -> Result<[f32; 3]> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }
    /// The length includes any alignment/null bytes in the serialized string.
    pub fn string8(&mut self) -> Result<String> {
        let len = self.take(1)?[0] as usize;
        text(self.take(len)?)
    }
}
pub fn text(data: &[u8]) -> Result<String> {
    let end = data.iter().position(|&x| x == 0).unwrap_or(data.len());
    Ok(std::str::from_utf8(&data[..end])
        .context("invalid UTF-8 name")?
        .to_owned())
}

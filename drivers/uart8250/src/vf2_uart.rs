use basic::{io::SafeIORegion, AlienResult};

pub struct Uart8250<const W: usize> {
    region: SafeIORegion,
}

impl<const W: usize> Uart8250<W> {
    pub fn new(region: SafeIORegion) -> Self {
        Self { region }
    }
    pub fn enable_receive_interrupt(&self) -> AlienResult<()> {
        let ier = self.region.read_at::<u32>(W * 1)? as u8;
        self.region.write_at::<u32>(W * 1, (ier | 1) as u32)?;
        Ok(())
    }
    pub fn disable_receive_interrupt(&self) -> AlienResult<()> {
        let ier = self.region.read_at::<u32>(W * 1)? as u8;
        self.region.write_at::<u32>(W * 1, (ier & !1) as u32)?;
        Ok(())
    }

    fn loop_putc(&self, ch: u8) -> AlienResult<()> {
        loop {
            let lsr = self.region.read_at::<u32>(W * 5)?;
            if lsr & 0x20 != 0 {
                self.region.write_at::<u32>(0 * W, ch as _)?;
                break;
            }
        }
        Ok(())
    }

    pub fn putc(&self, ch: u8) -> AlienResult<()> {
        if ch == b'\n' {
            self.loop_putc(b'\r')?;
        }
        self.loop_putc(ch)
    }

    pub fn getc(&self) -> AlienResult<Option<u8>> {
        let lsr = self.region.read_at::<u32>(W * 5)? as u8;
        if lsr & 1 != 0 {
            return Ok(Some(self.region.read_at::<u32>(W * 0)? as _));
        }
        Ok(None)
    }

    pub fn have_data_to_get(&self) -> AlienResult<bool> {
        let lsr = self.region.read_at::<u32>(W * 5)?;
        Ok(lsr & 1 != 0)
    }
}

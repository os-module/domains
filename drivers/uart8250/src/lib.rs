#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use core::{
    fmt::{Debug, Formatter},
    ops::Range,
};

use basic::{println, AlienResult};
use interface::{Basic, DeviceBase, UartDomain};
use spin::Once;
use uart8250::MmioUart8250;

static UART: Once<MmioUart8250<u32>> = Once::new();
struct Uart8250;

impl DeviceBase for Uart8250 {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl Debug for Uart8250 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Uart8250"))
    }
}

impl Basic for Uart8250 {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl UartDomain for Uart8250 {
    fn init(&self, address_range: &Range<usize>) -> AlienResult<()> {
        let region = address_range;
        println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
        // let io_region = SafeIORegion::from(region.clone());
        let uart = MmioUart8250::new(region.start);
        UART.call_once(|| uart);
        self.enable_receive_interrupt()?;
        println!("init uart success");
        Ok(())
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        let uart = UART.get().unwrap();
        loop {
            if uart.write_byte(ch).is_ok() {
                break;
            }
        }
        Ok(())
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        Ok(UART.get().unwrap().read_byte())
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        Ok(UART.get().unwrap().is_data_ready())
    }

    fn enable_receive_interrupt(&self) -> AlienResult<()> {
        UART.get()
            .unwrap()
            .enable_received_data_available_interrupt();
        Ok(())
    }

    fn disable_receive_interrupt(&self) -> AlienResult<()> {
        UART.get()
            .unwrap()
            .disable_received_data_available_interrupt();
        Ok(())
    }
}

pub fn main() -> Box<dyn UartDomain> {
    Box::new(Uart8250)
}

#![no_std]
#![forbid(unsafe_code)]
mod vf2_uart;
extern crate alloc;
use alloc::boxed::Box;
use core::{
    fmt::{Debug, Formatter},
    ops::Range,
};

use basic::{io::SafeIORegion, println, AlienResult};
use interface::{Basic, DeviceBase, UartDomain};
use rref::RRefVec;
use spin::Once;

use crate::vf2_uart::Uart8250;

static UART: Once<Uart8250<4>> = Once::new();
struct Uart8250Domain;

impl DeviceBase for Uart8250Domain {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl Debug for Uart8250Domain {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Uart8250"))
    }
}

impl Basic for Uart8250Domain {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl UartDomain for Uart8250Domain {
    fn init(&self, address_range: &Range<usize>) -> AlienResult<()> {
        let region = address_range;
        println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
        // let io_region = SafeIORegion::from(region.clone());
        let uart = Uart8250::<4>::new(SafeIORegion::from(region.clone()));
        UART.call_once(|| uart);
        self.enable_receive_interrupt().unwrap();
        println!("init uart success");
        Ok(())
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        let uart = UART.get().unwrap();
        uart.putc(ch)
    }
    fn getc(&self) -> AlienResult<Option<u8>> {
        UART.get().unwrap().getc()
    }

    fn put_bytes(&self, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let uart = UART.get().unwrap();
        for i in 0..buf.len() {
            uart.putc(buf[i])?;
        }
        Ok(buf.len())
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        UART.get().unwrap().have_data_to_get()
    }

    fn enable_receive_interrupt(&self) -> AlienResult<()> {
        UART.get().unwrap().enable_receive_interrupt()
    }

    fn disable_receive_interrupt(&self) -> AlienResult<()> {
        UART.get().unwrap().disable_receive_interrupt()
    }
}

pub fn main() -> Box<dyn UartDomain> {
    Box::new(Uart8250Domain)
}

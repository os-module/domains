#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use core::fmt::Debug;

use basic::{println, sync::Mutex, AlienError, AlienResult};
use interface::{Basic, BufUartDomain, DeviceBase, DomainType, UartDomain};
use rref::RRefVec;
use spin::Once;

static UART: Once<Arc<dyn UartDomain>> = Once::new();
#[derive(Debug)]
pub struct Uart {
    inner: Mutex<UartInner>,
}

#[derive(Debug)]
struct UartInner {
    rx_buf: VecDeque<u8>,
    wait_queue: VecDeque<usize>,
}

impl Default for Uart {
    fn default() -> Self {
        Self::new()
    }
}

impl Uart {
    pub fn new() -> Self {
        let inner = UartInner {
            rx_buf: VecDeque::new(),
            wait_queue: VecDeque::new(),
        };
        Uart {
            inner: Mutex::new(inner),
        }
    }
}

impl Basic for Uart {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl DeviceBase for Uart {
    fn handle_irq(&self) -> AlienResult<()> {
        let mut inner = self.inner.lock();
        let uart = UART.get().unwrap();
        while let Ok(Some(c)) = uart.getc() {
            inner.rx_buf.push_back(c);
            if !inner.wait_queue.is_empty() {
                let tid = inner.wait_queue.pop_front().unwrap();
                basic::wake_up_wait_task(tid)?
            }
        }
        Ok(())
    }
}

impl BufUartDomain for Uart {
    fn init(&self, uart_domain_name: &str) -> AlienResult<()> {
        let uart_domain = basic::get_domain(uart_domain_name).unwrap();
        match uart_domain {
            DomainType::UartDomain(uart) => {
                // enable receive interrupt
                // todo!(update it)
                uart.enable_receive_interrupt()?;
                UART.call_once(|| uart);
                Ok(())
            }
            ty => {
                println!("uart_domain_name: {},ty: {:?}", uart_domain_name, ty);
                Err(AlienError::EINVAL)
            }
        }?;
        println!("init buf uart success");
        Ok(())
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        let uart = UART.get().unwrap();
        uart.putc(ch)
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        loop {
            let mut inner = self.inner.lock();
            if inner.rx_buf.is_empty() {
                let tid = basic::current_tid()?.unwrap();
                inner.wait_queue.push_back(tid);
                drop(inner);
                basic::wait_now()?;
            } else {
                return Ok(inner.rx_buf.pop_front());
            }
        }
    }

    fn put_bytes(&self, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let uart = UART.get().unwrap();
        uart.put_bytes(buf)
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        Ok(!self.inner.lock().rx_buf.is_empty())
    }

    fn have_space_to_put(&self) -> AlienResult<bool> {
        Ok(true)
    }
}

pub fn main() -> Box<dyn BufUartDomain> {
    let uart = Uart::new();
    Box::new(uart)
}

#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque};
use core::{fmt::Debug, ops::Range};

use basic::{sync::Mutex, AlienResult};
use interface::{Basic, DeviceBase, NetDeviceDomain};
use rref::RRefVec;

#[derive(Debug)]
pub struct LoopBackNetDevice {
    mac_address: [u8; 6],
    packet: Mutex<VecDeque<RRefVec<u8>>>,
}

impl LoopBackNetDevice {
    pub fn new() -> Self {
        Self {
            mac_address: [0xff; 6],
            packet: Mutex::new(VecDeque::new()),
        }
    }
}

impl DeviceBase for LoopBackNetDevice {
    fn handle_irq(&self) -> AlienResult<()> {
        Ok(())
    }
}

impl Basic for LoopBackNetDevice {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl NetDeviceDomain for LoopBackNetDevice {
    fn init(&self, _device_info: &Range<usize>) -> AlienResult<()> {
        Ok(())
    }

    fn mac_address(&self) -> AlienResult<[u8; 6]> {
        Ok(self.mac_address)
    }

    fn can_transmit(&self) -> AlienResult<bool> {
        Ok(true)
    }

    fn can_receive(&self) -> AlienResult<bool> {
        Ok(true)
    }

    fn rx_queue_size(&self) -> AlienResult<usize> {
        Ok(128)
    }

    fn tx_queue_size(&self) -> AlienResult<usize> {
        Ok(128)
    }

    fn transmit(&self, tx_buf: &RRefVec<u8>) -> AlienResult<()> {
        let packet = RRefVec::from_slice(tx_buf.as_slice());
        self.packet.lock().push_back(packet);
        Ok(())
    }

    fn receive(&self, rx_buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let mut packet = self.packet.lock();
        if let Some(p) = packet.pop_front() {
            let len = p.len();
            Ok((p, len))
        } else {
            Ok((rx_buf, 0))
        }
    }
}

pub fn main() -> Box<dyn NetDeviceDomain> {
    Box::new(LoopBackNetDevice::new())
}

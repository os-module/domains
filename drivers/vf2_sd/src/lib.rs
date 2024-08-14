#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "fs_test")]
mod fs_test;
mod ops;

extern crate alloc;
use alloc::boxed::Box;
use core::ops::Range;

use basic::{io::SafeIORegion, println, println_color, sync::Mutex, AlienResult};
use interface::{define_unwind_for_BlkDeviceDomain, Basic, BlkDeviceDomain, DeviceBase};
use rref::RRef;
use spin::Lazy;
use visionfive2_sd::Vf2SdDriver;

use crate::ops::{SdIoImpl, SleepOpsImpl};

static SD_CARD: Lazy<Mutex<Vf2SdDriver<SdIoImpl, SleepOpsImpl>>> = Lazy::new(|| {
    let io = SafeIORegion::from(0..0);
    Mutex::new(Vf2SdDriver::new(SdIoImpl::new(io)))
});

#[derive(Debug)]
pub struct Vf2SDCardDomain;

impl DeviceBase for Vf2SDCardDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        unimplemented!()
    }
}

impl Basic for Vf2SDCardDomain {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl BlkDeviceDomain for Vf2SDCardDomain {
    fn init(&self, device_info: &Range<usize>) -> AlienResult<()> {
        let region = device_info;
        println!("sd card: {:#x}-{:#x}", region.start, region.end);
        let io_region = SafeIORegion::from(device_info.clone());
        // preprint::init_print(&PrePrint);
        let mut sd = Vf2SdDriver::new(SdIoImpl::new(io_region));
        sd.init();
        println_color!(32, "sd card init success");
        let mut buf = [0; 512];
        sd.read_block(0, &mut buf);
        println!("buf: {:x?}", &buf[..16]);
        #[cfg(feature = "fs_test")]
        fs_test::init_fatfs(sd);
        #[cfg(not(feature = "fs_test"))]
        {
            *SD_CARD.lock() = sd;
        }
        Ok(())
    }

    fn read_block(&self, block: u32, mut data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        SD_CARD
            .lock()
            .read_block(block as usize, data.as_mut_slice());
        Ok(data)
    }

    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        SD_CARD.lock().write_block(block as usize, data.as_ref());
        Ok(data.len())
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        Ok(32 * 1024 * 1024 * 1024 / 512)
    }

    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }
}

define_unwind_for_BlkDeviceDomain!(Vf2SDCardDomain);

pub fn main() -> Box<dyn BlkDeviceDomain> {
    Box::new(UnwindWrap::new(Vf2SDCardDomain))
}

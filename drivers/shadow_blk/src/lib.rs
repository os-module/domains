#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
};

use basic::{println, println_color, AlienError, AlienResult};
use interface::{
    define_unwind_for_ShadowBlockDomain, Basic, BlkDeviceDomain, DeviceBase, DomainType,
    ShadowBlockDomain,
};
use log::error;
use rref::RRef;
use spin::Once;

static BLOCK: Once<Arc<dyn BlkDeviceDomain>> = Once::new();

#[derive(Debug)]
pub struct ShadowBlockDomainImpl {
    blk_domain_name: Once<String>,
}

impl Default for ShadowBlockDomainImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowBlockDomainImpl {
    pub fn new() -> Self {
        Self {
            blk_domain_name: Once::new(),
        }
    }
}

impl Basic for ShadowBlockDomainImpl {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl DeviceBase for ShadowBlockDomainImpl {
    fn handle_irq(&self) -> AlienResult<()> {
        BLOCK.get().unwrap().handle_irq()
    }
}

impl ShadowBlockDomain for ShadowBlockDomainImpl {
    fn init(&self, blk_domain: &str) -> AlienResult<()> {
        let blk = basic::get_domain(blk_domain).unwrap();
        let blk = match blk {
            DomainType::BlkDeviceDomain(blk) => blk,
            _ => panic!("not a block domain"),
        };
        BLOCK.call_once(|| blk);
        self.blk_domain_name.call_once(|| blk_domain.to_string());
        Ok(())
    }

    // todo!(fix it if more than one thread read the same block at the same time)
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        let blk = BLOCK.get().unwrap();
        let mut data = data;
        let res = blk.read_block(block, data);
        match res {
            Ok(res) => Ok(res),
            Err(AlienError::DOMAINCRASH) => {
                error!("domain crash, try restart domain");
                basic::checkout_shared_data().unwrap();
                // try reread block
                println_color!(31, "try reread block");
                data = RRef::new([0u8; 512]);
                blk.read_block(block, data)
            }
            Err(e) => Err(e),
        }
    }

    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        BLOCK.get().unwrap().write_block(block, data)
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        BLOCK.get().unwrap().get_capacity()
    }

    fn flush(&self) -> AlienResult<()> {
        BLOCK.get().unwrap().flush()
    }
}

define_unwind_for_ShadowBlockDomain!(ShadowBlockDomainImpl);

pub fn main() -> Box<dyn ShadowBlockDomain> {
    Box::new(UnwindWrap::new(ShadowBlockDomainImpl::new()))
}

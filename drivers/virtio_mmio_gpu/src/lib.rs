#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::{fmt::Debug, ops::Range};

use basic::{io::SafeIORegion, println, sync::Mutex, AlienError, AlienResult};
use interface::{Basic, DeviceBase, GpuDomain};
use rref::RRefVec;
use spin::Once;
use virtio_drivers::{device::gpu::VirtIOGpu, transport::mmio::MmioTransport};
use virtio_mmio_common::{HalImpl, SafeIORW};

static GPU: Once<Mutex<VirtIOGpu<HalImpl, MmioTransport>>> = Once::new();

#[derive(Debug)]
pub struct GPUDomain {
    buffer_range: Once<Range<usize>>,
}

impl GPUDomain {
    pub fn new() -> Self {
        Self {
            buffer_range: Once::new(),
        }
    }
}

impl Basic for GPUDomain {}

impl DeviceBase for GPUDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        unimplemented!()
    }
}

impl GpuDomain for GPUDomain {
    fn init(&self, address_range: &Range<usize>) -> AlienResult<()> {
        let virtio_gpu_addr = address_range.start;
        println!("virtio_gpu_addr: {:#x?}", virtio_gpu_addr);
        let io_region = SafeIORW(SafeIORegion::from(address_range.clone()));
        let transport = MmioTransport::new(Box::new(io_region)).unwrap();
        let mut gpu = VirtIOGpu::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create gpu driver");

        let (width, height) = gpu.resolution().expect("failed to get resolution");
        let width = width as usize;
        let height = height as usize;
        println!("GPU resolution is {}x{}", width, height);
        let fb = gpu.setup_framebuffer().expect("failed to get fb");
        let buffer_range = fb.as_ptr() as usize..(fb.as_ptr() as usize + fb.len());
        gpu.move_cursor(50, 50).unwrap();
        gpu.flush().unwrap();
        self.buffer_range.call_once(|| buffer_range);
        GPU.call_once(|| Mutex::new(gpu));
        Ok(())
    }

    fn flush(&self) -> AlienResult<()> {
        let gpu = GPU.get().unwrap();
        gpu.lock().flush().unwrap();
        Ok(())
    }

    fn fill(&self, _offset: u32, _buf: &RRefVec<u8>) -> AlienResult<usize> {
        todo!()
    }

    fn buffer_range(&self) -> AlienResult<Range<usize>> {
        self.buffer_range
            .get()
            .ok_or(AlienError::EINVAL)
            .map(|r| r.clone())
    }
}

pub fn main() -> Box<dyn GpuDomain> {
    Box::new(GPUDomain::new())
}

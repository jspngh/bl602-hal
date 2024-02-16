//! Core-Local Interrupt Controller
use embedded_time::fixed_point::FixedPoint;
use embedded_time::rate::{Hertz, Kilohertz, Megahertz};

use crate::clock::Clocks;

const CLIC_CTRL_ADDR: u32 = 0x02000000;

const CLIC_MTIMECMP: u32 = 0x4000;
const CLIC_MTIME: u32 = 0xBFF8;

pub struct Clic {
    freq: Hertz,
}

impl Clic {
    /// Returns a `Clic` instance if the RTC is enabled
    pub fn try_new(clocks: Clocks) -> Option<Self> {
        // The rtc clock depends the frequency of mtimer ticks
        clocks.rtc_clk().map(|freq| Self { freq })
    }

    pub fn get_time_ms(&self) -> u64 {
        let scale = Kilohertz::<u32>::from(self.freq).integer() as u64;
        self.get_ticks() / scale
    }

    pub fn get_time_us(&self) -> u64 {
        let scale = Megahertz::<u32>::from(self.freq).integer() as u64;
        self.get_ticks() / scale
    }

    /// Read the current tick from the mtime register
    pub fn get_ticks(&self) -> u64 {
        // The bl_iot_sdk has multiple implemenations for reading the mtime register.
        // One does 2 separate 32-bit reads as follows:
        // ```rust
        // let ticks = unsafe {
        //     let mtime_low_addr = (CLIC_CTRL_ADDR + CLIC_MTIME) as *const u32;
        //     let mtime_high_addr = (CLIC_CTRL_ADDR + CLIC_MTIME + 4) as *const u32;
        //     let mut mtime_low = mtime_low_addr.read_volatile();
        //     let mut mtime_high = mtime_high_addr.read_volatile();
        //     while mtime_high_addr.read_volatile() != mtime_high {
        //         mtime_low = mtime_low_addr.read_volatile();
        //         mtime_high = mtime_high_addr.read_volatile();
        //     }
        //     (mtime_high as u64) << 32 | mtime_low as u64
        // };
        // ```
        // However there seems to be no need for this, as a single 64-bit read also works.

        let mtime_addr = (CLIC_CTRL_ADDR + CLIC_MTIME) as *const u64;
        unsafe { mtime_addr.read_volatile() }
    }

    /// Set the mtimecmp register with the current mtime value + the specified delay (in ticks)
    pub fn set_timecmp(&mut self, delay: u64) {
        let mtimecmp_addr = (CLIC_CTRL_ADDR + CLIC_MTIMECMP) as *mut u64;
        unsafe {
            mtimecmp_addr.write_volatile(self.get_ticks() + delay);
        }
    }
}

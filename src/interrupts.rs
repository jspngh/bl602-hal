/*!
  # Interrupt Management
  Interrupts can be enabled, disabled and cleared.

  ## Example
  ```rust
    enable_interrupt(TimerCh0);

    // ...

    #[no_mangle]
    fn TimerCh0() {
        // ..
        clear_interrupt(TimerCh0);
    }
  ```

  ## The following functions can be implemented as interrupt handlers
  ```rust
    fn Gpio();
    fn TimerCh0();
    fn TimerCh1();
    fn Watchdog();
  ```
*/

use riscv::register::mcause;

extern "C" {
    fn MachineSoft(trap_frame: &mut TrapFrame);
    fn MachineTimer(trap_frame: &mut TrapFrame);
    fn MachineExternal(trap_frame: &mut TrapFrame);

    fn Gpio(trap_frame: &mut TrapFrame);
    fn TimerCh0(trap_frame: &mut TrapFrame);
    fn TimerCh1(trap_frame: &mut TrapFrame);
    fn Watchdog(trap_frame: &mut TrapFrame);
    fn Dma(trap_frame: &mut TrapFrame);
    fn Spi(trap_frame: &mut TrapFrame);
    fn Uart0(trap_frame: &mut TrapFrame);
    fn Uart1(trap_frame: &mut TrapFrame);
    fn I2c(trap_frame: &mut TrapFrame);
    fn Pwm(trap_frame: &mut TrapFrame);
}

// see components\bl602\bl602_std\bl602_std\RISCV\Core\Include\clic.h
// see components\hal_drv\bl602_hal\bl_irq.c
const IRQ_NUM_BASE: u32 = 16;
const CLIC_HART0_ADDR: u32 = 0x02800000;
const CLIC_INTIE: u32 = 0x400;
const CLIC_INTIP: u32 = 0x000;

const MSIP_IRQ: u32 = 3;
const MTIP_IRQ: u32 = 7;
const MEIP_IRQ: u32 = 11;

const DMA0_IRQ: u32 = IRQ_NUM_BASE + 15;
const SPI0_IRQ: u32 = IRQ_NUM_BASE + 27;
const UART0_IRQ: u32 = IRQ_NUM_BASE + 29;
const UART1_IRQ: u32 = IRQ_NUM_BASE + 30;
const I2C0_IRQ: u32 = IRQ_NUM_BASE + 32;
const PWM_IRQ: u32 = IRQ_NUM_BASE + 34;
const TIMER_CH0_IRQ: u32 = IRQ_NUM_BASE + 36;
const TIMER_CH1_IRQ: u32 = IRQ_NUM_BASE + 37;
const WATCHDOG_IRQ: u32 = IRQ_NUM_BASE + 38;
const GPIO_IRQ: u32 = IRQ_NUM_BASE + 44;

#[doc(hidden)]
#[no_mangle]
pub fn _setup_interrupts() {
    extern "C" {
        pub fn _start_trap_hal();
    }

    let new_mtvec = _start_trap_hal as usize;
    unsafe {
        riscv::interrupt::disable();
        riscv::register::mtvec::write(new_mtvec | 2, riscv::register::mtvec::TrapMode::Direct);
    }

    // disable all interrupts
    let e = unsafe {
        core::slice::from_raw_parts_mut((CLIC_HART0_ADDR + CLIC_INTIE) as *mut u32, 16 + 8)
    };
    let p = unsafe {
        core::slice::from_raw_parts_mut((CLIC_HART0_ADDR + CLIC_INTIP) as *mut u32, 16 + 8)
    };

    e.iter_mut().for_each(|v| *v = 0);
    p.iter_mut().for_each(|v| *v = 0);

    unsafe {
        riscv::interrupt::enable();
    }
}

/// Registers saved in trap handler
#[doc(hidden)]
#[allow(missing_docs)]
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    pub ra: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub gp: usize,
    pub tp: usize,
    pub sp: usize,
}

/// # Safety
///
/// This function is called from an assembly trap handler.
#[doc(hidden)]
#[link_section = ".trap.rust"]
#[export_name = "_start_trap_rust_hal"]
pub unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
    extern "C" {
        pub fn _start_trap_rust(trap_frame: *const TrapFrame);
    }

    let cause = mcause::read();
    if cause.is_exception() {
        _start_trap_rust(trap_frame);
    } else {
        let code = cause.code();
        if code < IRQ_NUM_BASE as usize {
            _start_trap_rust(trap_frame);
        } else {
            let interrupt_number = (code & 0xff) as u32;
            let interrupt = Interrupt::from(interrupt_number);

            match interrupt {
                Interrupt::Unknown => _start_trap_rust(trap_frame),
                Interrupt::MachineSoft => MachineSoft(trap_frame.as_mut().unwrap()),
                Interrupt::MachineTimer => MachineTimer(trap_frame.as_mut().unwrap()),
                Interrupt::MachineExternal => MachineExternal(trap_frame.as_mut().unwrap()),
                Interrupt::Gpio => Gpio(trap_frame.as_mut().unwrap()),
                Interrupt::TimerCh0 => TimerCh0(trap_frame.as_mut().unwrap()),
                Interrupt::TimerCh1 => TimerCh1(trap_frame.as_mut().unwrap()),
                Interrupt::Watchdog => Watchdog(trap_frame.as_mut().unwrap()),
                Interrupt::Dma => Dma(trap_frame.as_mut().unwrap()),
                Interrupt::Spi => Spi(trap_frame.as_mut().unwrap()),
                Interrupt::Uart0 => Uart0(trap_frame.as_mut().unwrap()),
                Interrupt::Uart1 => Uart1(trap_frame.as_mut().unwrap()),
                Interrupt::I2c => I2c(trap_frame.as_mut().unwrap()),
                Interrupt::Pwm => Pwm(trap_frame.as_mut().unwrap()),
            };
        }
    }
}

/// Available interrupts
pub enum Interrupt {
    #[doc(hidden)]
    Unknown,
    /// Machine Software Interrupt
    MachineSoft,
    /// Machine Timer Interrupt
    MachineTimer,
    /// Machine External Interrupt
    MachineExternal,
    /// GPIO Interrupt
    Gpio,
    /// Timer Channel 0 Interrupt
    TimerCh0,
    /// Timer Channel 1 Interrupt
    TimerCh1,
    /// Watchdog Timer Interrupt
    /// Used when WDT is configured in Interrupt mode using ConfiguredWatchdog0::set_mode()
    Watchdog,
    /// DMA Interrupt
    Dma,
    /// SPI Interrupt
    Spi,
    /// UART Port 0 Interrupt
    Uart0,
    /// UART Port 1 Interrupt
    Uart1,
    /// I2C Interrupt
    I2c,
    /// PWM Interrupt
    Pwm,
}

impl Interrupt {
    fn to_irq(&self) -> u32 {
        match &self {
            Interrupt::Unknown => panic!("Unknown interrupt has no irq number"),
            Interrupt::MachineSoft => MSIP_IRQ,
            Interrupt::MachineTimer => MTIP_IRQ,
            Interrupt::MachineExternal => MEIP_IRQ,
            Interrupt::Gpio => GPIO_IRQ,
            Interrupt::TimerCh0 => TIMER_CH0_IRQ,
            Interrupt::TimerCh1 => TIMER_CH1_IRQ,
            Interrupt::Watchdog => WATCHDOG_IRQ,
            Interrupt::Dma => DMA0_IRQ,
            Interrupt::Spi => SPI0_IRQ,
            Interrupt::Uart0 => UART0_IRQ,
            Interrupt::Uart1 => UART1_IRQ,
            Interrupt::I2c => I2C0_IRQ,
            Interrupt::Pwm => PWM_IRQ,
        }
    }

    fn from(irq: u32) -> Interrupt {
        match irq {
            MSIP_IRQ => Interrupt::MachineSoft,
            MTIP_IRQ => Interrupt::MachineTimer,
            MEIP_IRQ => Interrupt::MachineExternal,
            GPIO_IRQ => Interrupt::Gpio,
            TIMER_CH0_IRQ => Interrupt::TimerCh0,
            TIMER_CH1_IRQ => Interrupt::TimerCh1,
            WATCHDOG_IRQ => Interrupt::Watchdog,
            DMA0_IRQ => Interrupt::Dma,
            SPI0_IRQ => Interrupt::Spi,
            UART0_IRQ => Interrupt::Uart0,
            UART1_IRQ => Interrupt::Uart1,
            I2C0_IRQ => Interrupt::I2c,
            PWM_IRQ => Interrupt::Pwm,
            _ => Interrupt::Unknown,
        }
    }
}

/// Enable the given interrupt
pub fn enable_interrupt(interrupt: Interrupt) {
    let irq = interrupt.to_irq();
    let ptr = (CLIC_HART0_ADDR + CLIC_INTIE + irq) as *mut u8;
    unsafe {
        ptr.write_volatile(1);
    }
}

/// Disable the given interrupt
pub fn disable_interrupt(interrupt: Interrupt) {
    let irq = interrupt.to_irq();
    let ptr = (CLIC_HART0_ADDR + CLIC_INTIE + irq) as *mut u8;
    unsafe {
        ptr.write_volatile(0);
    }
}

/// Clear the given interrupt.
/// Usually the interrupt needs to be cleared also on the peripheral level.
pub fn clear_interrupt(interrupt: Interrupt) {
    let irq = interrupt.to_irq();
    let ptr = (CLIC_HART0_ADDR + CLIC_INTIP + irq) as *mut u8;
    unsafe {
        ptr.write_volatile(0);
    }
}

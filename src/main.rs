#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;


use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac,
    prelude::*,
    gpio::*,
};

mod test_points;
use test_points::{*};


#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let rcc = dp.RCC.constrain();
    let gpioc: stm32f4xx_hal::gpio::gpioc::Parts = dp.GPIOC.split();

    // Configure PA5 as a digital output
    let mut test_point = TestPoints::new(gpioc);


    // Toggle the LED
    let mut shift: u8 = 1;
    loop {
        test_point.reset_all();

        match shift {
            0x01 => { set!(test_point, 1)},
            0x02 => { set!(test_point, 2)},
            0x04 => { set!(test_point, 3)},
            0x08 => { set!(test_point, 4)},
            0x10 => { set!(test_point, 5)},
            0x20 => { set!(test_point, 6)},
            0x40 => { set!(test_point, 7)},
            0x80 => { set!(test_point, 8)},
            _ => {shift = 0x01}
        }
        shift <<= 1;
        if shift == 0 {shift = 0x01}

        cortex_m::asm::delay(4_000_000);
    }}

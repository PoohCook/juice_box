#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    gpio::*, pac, prelude::*
};

mod test_points;
use test_points::{*};

mod display;
use display::*;


#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let _ = dp.RCC.constrain();

    // let gpioa = dp.GPIOA.split();
    // let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpioe = dp.GPIOE.split();

    // Configure PA5 as a digital output
    let mut test_point = TestPoints::new(
        gpioc.pc0,
        gpioc.pc1,
        gpioc.pc2,
        gpioc.pc3,
        gpioc.pc4,
        gpioc.pc5,
        gpioc.pc6,
        gpioc.pc7,
    );
    test_point.reset_all();

    // Configure SPI peripheral
    let mut display = TM1638::new(
        gpioc.pc8,
        gpioc.pc9,
        gpioc.pc10,
    );

    display.initialize();

    let mut cur_count = 0;

    loop {

        display.display_num(1, cur_count);
        display.display_num(2, cur_count+20);
        display.display_num(3, cur_count+40);
        display.display_num(4, cur_count+80);


        cur_count += 1;
        if cur_count > 99 {
            cur_count = 0;
        }
        cortex_m::asm::delay(8_000_000);

    }}

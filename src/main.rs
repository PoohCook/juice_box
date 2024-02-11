#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    gpio::*, pac, prelude::*, rtc::Event
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

    display.initialize(7);
    display.set_brightness(7);
    display.display_num(1, 1);
    display.display_num(2, 2);
    display.display_num(3, 3);
    display.display_num(4, 4);


    let mut cur_count = 0;

    let mut butts: u8 = 0;

    loop {
        cur_count += 1;
        if cur_count > 99 {
            cur_count = 0;
        }

        // let butts = display.read_buttons();
        let key_event = display.get_key_events();

        match key_event {
            Some(ev) => {
                match ev {
                    KeyEvent::KeyDown { key } => {
                        butts ^= 0x01 << key;
                    },
                    _ => {}
                }
            }
            None => {}
        }

        test_point.write_value(butts);

        cortex_m::asm::delay(800);

    }}

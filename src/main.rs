#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use ws2812_spi as ws2812;

// use crate::hal::delay::Delay;
use crate::hal::pac;
use crate::hal::gpio::NoPin;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::ws2812::Ws2812;

use smart_leds::{gamma, SmartLedsWrite, RGB8, hsv::Hsv, hsv::hsv2rgb};

mod test_points;
use test_points::{*};

mod display;
use display::*;


#[entry]
fn main() -> ! {
    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(8.MHz()).freeze();

    let gpioa = dp.GPIOA.split();
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
    display.display_num(1, 0);
    display.display_num(2, 0);
    display.display_num(3, 0);
    display.display_num(4, 0);

    set!(test_point, 1);

    // Configure pins for SPI
    let sck1 = gpioa.pa5.into_alternate();
    let miso1 = NoPin::new();                          // miso not needed
    let mosi1 = gpioa.pa7.into_alternate();     // PA7 is output going to data line of leds

    set!(test_point, 2);
    // SPI1 with 3Mhz
    let spi = Spi::new(dp.SPI1, (sck1, miso1, mosi1), ws2812::MODE, 3_000_000.Hz(), &clocks);

    cortex_m::asm::delay(8000);
    let mut ws = Ws2812::new(spi);
    cortex_m::asm::delay(8000);

    set!(test_point, 3);
    const LED_NUM: usize = 8;
    let mut data = [RGB8::new(0x7f, 0x00, 0x00); LED_NUM];
    // before writing, apply gamma correction for nicer rainbow

    set!(test_point, 4);
    let mut cur_count = 0;

    let mut butts = [0,0,0,0];

    loop {
        cur_count += 1;
        if cur_count > 99 {
            cur_count = 0;
        }

        set!(test_point, 5);
        ws.write(data.iter().cloned());

        test_point.reset_all();
        set!(test_point, 6);
        // let butts = display.read_buttons();
        let key_event = display.get_key_events();

        match key_event {
            Some(ev) => {
                match ev {
                    KeyEvent::KeyDown { key: 1 } => {
                        butts[0] += 1;
                        display.display_num(1, butts[0]);
                    },
                    KeyEvent::KeyDown { key: 3 } => {
                        butts[1] += 1;
                        display.display_num(2, butts[1]);
                    },
                    KeyEvent::KeyDown { key: 5 } => {
                        butts[2] += 1;
                        display.display_num(3, butts[2]);
                    },
                    KeyEvent::KeyDown { key: 7 } => {
                        butts[3] += 1;
                        display.display_num(4, butts[3]);
                    },
                    _ => {}
                }
            }
            None => {}
        }

        cortex_m::asm::delay(800);

    }}

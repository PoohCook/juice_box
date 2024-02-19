




#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::pac;
use crate::hal::prelude::*;
use ws2812_spi as ws2812;


use smart_leds::RGB8;
use rtt_target::{rprintln, rtt_init_print};

mod test_points;
use test_points::{*};

mod ev_charger;
use ev_charger::*;

mod display;
use display::*;

mod light_ports;
use light_ports::*;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let rcc = dp.RCC.constrain();
    let clocks: hal::rcc::Clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let gpioa = dp.GPIOA.split();
    // let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpioe = dp.GPIOE.split();

    // Configure PA5 as a digital output
    let mut test_point = TestPoints::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7,
    );
    test_point.reset_all();

    // Configure SPI peripheral
    let mut display = TM1638::new(gpioc.pc8, gpioc.pc9, gpioc.pc10,);

    display.initialize(7);
    display.set_brightness(7);

    let mut buffer = [0u8; (20 * 12) + 30];
    let mut lights = LightPorts::new(gpioa.pa5, gpioa.pa7, dp.SPI1, &mut buffer, &clocks);

    let mut chargers = [
        EVCharger::new(1, 0),
        EVCharger::new(2, 1),
        EVCharger::new(3, 2),
        EVCharger::new(4, 3),
    ];

    loop {
        for chrg in &mut chargers {
            chrg.refresh_ui(&mut display, &mut lights);
        }

        let res = lights.refresh();
        rprintln!("{:?}", res);

        test_point.reset_all();
        set!(test_point, 6);

        let key_event = display.get_key_events();
        match key_event {
            Some(ev) => {
                for chrg in &mut chargers {
                    chrg.on_key_event(&ev);
                }
            },
            _ => {}
        };

        cortex_m::asm::delay(1_000_000);

    }}

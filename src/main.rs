#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::pac;
use crate::hal::pac::TIM2;
use crate::hal::prelude::*;
use crate::hal::timer::Counter;

use ws2812_spi as ws2812;

use rtt_target::rprintln;
use rtt_target::rtt_init_print;

mod test_points;
use test_points::{*};

mod ev_charger;
use ev_charger::*;

mod display;
use display::*;

mod pallet;
use pallet::Colors;

mod light_ports;
use light_ports::*;

mod serial;
use serial::*;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    19200.bps();

    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIO
    let rcc = dp.RCC.constrain();
    let clocks: hal::rcc::Clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let mut sys_timer:Counter<TIM2, 1000> = dp.TIM2.counter_ms(&clocks);
    sys_timer.start(u32::MAX.millis()).unwrap();

    let gpioa = dp.GPIOA.split();
    // let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpioe = dp.GPIOE.split();

    let mut modbus = ModbusTransceiver::new(gpioa.pa2, gpioa.pa3, gpioa.pa4, dp.USART2, dp.DMA1, &clocks, &sys_timer);

    // Configure PA5 as a digital output
    let mut test_point = TestPoints::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7,
    );
    test_point.reset_all();

    // Configure SPI peripheral
    let mut display = TM1638::new(gpioc.pc8, gpioc.pc9, gpioc.pc10,);

    display.initialize(7);
    display.set_brightness(7);

    let mut buffer = [0u8; (LED_NUM * 12) + 30];
    let mut lights = LightPorts::new(gpioa.pa5, gpioa.pa7, dp.SPI1, &mut buffer, &clocks, &sys_timer);

    let mut chargers = [
        EVCharger::new(1, 0),
        EVCharger::new(2, 1),
        EVCharger::new(3, 2),
        EVCharger::new(4, 3),
    ];

    loop {
        let mut updated = false;
        for chrg in &mut chargers {
            if chrg.refresh_ui(&mut display, &mut lights) {
                updated = true;
            }
        }

        lights.refresh( updated);

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

        modbus.scan_rx_msg(|msg: &ModbusFrame, modbus | {
            rprintln!("--> on_receive: {:?}", msg);
            if msg.unit_id == 1 {
                let resp = ModbusFrame::new(
                    msg.unit_id,
                    msg.command,
                    Reference::Size(2),
                    0
                );
                modbus.send_tx_msg(resp);
            }
        });

        // cortex_m::asm::delay(1_000_000);
        // this si a bit mickey mouse but it hunts for now
        let timeout: fugit::Instant<u32, 1, 1000> = sys_timer.now() + 1.millis();
        while sys_timer.now() < timeout { }


    }}

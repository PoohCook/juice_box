use stm32f4xx_hal as hal;

use crate::hal::rcc::*;
use crate::hal::pac::TIM2;
use crate::hal::prelude::*;
use crate::hal::timer::Counter;

use crate::hal::gpio::{Pin, Output};
use crate::hal::uart::{Rx, Tx};
use crate::hal::serial::{config::Config, Serial};
use crate::hal::dma::{Transfer, StreamsTuple, StreamX, PeripheralToMemory};
use crate::hal::dma::config::DmaConfig;
use crate::hal::pac::{USART2, DMA1};
use crate::hal::time::Bps;

use rtt_target::rprintln;

use crate::modbus::*;


// Create buffers for sending and receiving data
const BUF_LEN: usize = 20;
static mut RX_BUFFER: [u8; BUF_LEN] = [0; BUF_LEN];


pub struct ModbusTransceiver<'a> {
    sys_timer: &'a Counter<TIM2, 1000>,
    rx_transfer: Transfer<StreamX<DMA1, 5>, 4, Rx<USART2>, PeripheralToMemory, &'static mut [u8; BUF_LEN]>,
    last_xfs: u16,
    last_rcv_to: Option<fugit::Instant<u32, 1, 1000>>,
    den: Pin<'A', 4, Output>,
    uart_tx: Tx<USART2>

}

impl <'a>ModbusTransceiver<'a> {
    pub fn new(
        pa2: Pin<'A', 2>,
        pa3: Pin<'A', 3>,
        pa4: Pin<'A', 4>,
        usart2: USART2,
        dma1: DMA1,
        clocks: &Clocks,
        sys_timer: &'a Counter<TIM2, 1000>,
    ) -> Self {

        let tx = pa2.into_alternate();
        let rx = pa3.into_alternate();
        let mut den: Pin<'A', 4, hal::gpio::Output> = pa4.into_push_pull_output();

        den.set_low();

        let ser_config = Config::default()
        .baudrate(Bps(19200))
        .parity_none()
        .wordlength_8()
        .dma(hal::serial::config::DmaConfig::TxRx);

        rprintln!("ser config: {:?}", ser_config);

        let usart2: Serial<USART2> = Serial::new(
            usart2,
            (tx, rx),
            ser_config,
            &clocks,
        )
        .unwrap();

        // Split UART peripheral into transmitter and receiver
        let (uart2_tx, uart2_rx) = usart2.split();


        // Initialize DMA
        let dma_channels = StreamsTuple::new(dma1);
        let rx_channel = dma_channels.5;


        let dma_config = DmaConfig::default()
        .transfer_complete_interrupt(true)
        .memory_increment(true);

        rprintln!("dma config: {:?}", dma_config);

        let mut rx_transfer = Transfer::init_peripheral_to_memory(
            rx_channel,
            uart2_rx,
            unsafe { &mut RX_BUFFER },
            None,
            dma_config,
        );

        rx_transfer.start(|p: &mut hal::uart::Rx<USART2>| rprintln!("data: {:?}", p.is_rx_not_empty()));

        // let tx_transfer = Transfer::init_memory_to_peripheral(
        //     tx_channel,
        //     uart2_tx,
        //     unsafe { &mut TX_BUFFER },
        //     None,
        //     dma_config
        // );


        Self {
            sys_timer,
            rx_transfer,
            last_xfs: BUF_LEN as u16,
            last_rcv_to: None,
            den,
            uart_tx: uart2_tx
        }

    }

    pub fn scan_rx_msg<F>(&mut self, on_receive: F)
    where
        F: Fn(&ModbusFrame, &mut Self),
    {
        let xfrs = self.rx_transfer.number_of_transfers();
        if self.last_xfs != xfrs {
            self.last_rcv_to = Some(self.sys_timer.now() + 3.millis());
            self.last_xfs = xfrs;
        }

        match self.last_rcv_to {
            Some(tout) if self.sys_timer.now() >= tout  => {
                let rx_size = BUF_LEN - xfrs  as usize;
                let msg = unsafe{&RX_BUFFER[0..rx_size]};

                match ModbusFrame::decode(msg) {
                    Ok(msg) => {
                        on_receive(&msg, self);
                    }
                    _ => {}
                }

                self.last_rcv_to = None;
                self.last_xfs = BUF_LEN as u16;
                unsafe{RX_BUFFER = [0; BUF_LEN]};
                self.rx_transfer.next_transfer(unsafe{&mut RX_BUFFER})
                .unwrap();
            },
            _ => {}
        }

    }

    pub fn send_tx_msg(&mut self, msg: ModbusFrame) -> Result<(), &str> {

        rprintln!("<-- send: {:?}", msg);

        self.den.set_high();

        let mut tx_data: [u8; BUF_LEN] = [0; BUF_LEN];
        let len = msg.encode(&mut tx_data);

        let tx_data = &tx_data[..len];

        for d in tx_data {
            while !self.uart_tx.is_tx_empty(){};

            stm32f4xx_hal::prelude::_embedded_hal_serial_nb_Write::write(&mut self.uart_tx, *d)
            .unwrap();

        }

        let timeout: fugit::Instant<u32, 1, 1000> = self.sys_timer.now() + 2.millis();
        while self.sys_timer.now() < timeout { }

        self.den.set_low();
        Ok(())

    }

}

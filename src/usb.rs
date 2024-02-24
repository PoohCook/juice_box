use crate::hal::otg_fs::{UsbBus, USB};
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use rtt_target::rprintln;

const COM_MAX_LEN: usize = 128;

use crate::ev_charger::*;

pub struct UsbCommandProcessor<'a> {
    device: UsbDevice<'a, UsbBus<USB>>,
    serial: SerialPort<'a, UsbBus<USB>>,
    com_buf: [u8; COM_MAX_LEN],
    com_indx: usize,
}


impl <'a>UsbCommandProcessor<'a> {

    pub fn new(
        device: UsbDevice<'a, UsbBus<USB>>,
        serial: SerialPort<'a, UsbBus<USB>>,
     ) -> Self {

      Self {
            com_buf: [0u8; COM_MAX_LEN],
            com_indx: 0,
            serial,
            device
        }

    }

    pub fn poll(&mut self, chargers: &mut [EVCharger; 4]) {

        let mut buf = [0u8; COM_MAX_LEN];

        if self.device.poll(&mut [&mut self.serial]) {
            match self.serial.read(&mut buf) {
                Ok(count) if count > 0 => {

                    let mut count = count;
                    for chr in &buf[0..count] {
                        self.com_buf[self.com_indx] = *chr;
                        self.com_indx += 1;
                    }

                    if buf[count-1] == '\r' as u8 {
                        buf[count] = '\n' as u8;
                        count += 1;
                    }
                    let mut write_offset = 0;
                    while write_offset < count {
                        match self.serial.write(&buf[write_offset..count]) {
                            Ok(len) if len > 0 => {
                                write_offset += len;
                            }
                            _ => {}
                        }
                    }

                    let command = core::str::from_utf8(&self.com_buf[0..self.com_indx]).unwrap_or_default();
                    if command.contains("\r"){
                        let end = command.chars().position(|c| c == '\r').unwrap_or_default();
                        let command = &command[..end];

                        Self::process_command(command, chargers);
                        self.com_indx = 0;
                    }

                }
                _ => {}
            }
        }

    }

    fn process_command(command: &str, _chargers: &mut [EVCharger; 4]){
        rprintln!("command is: {}",  command);

        if command == "get_units" {
            let _reply = "units[]";
        }
    }

}

use core::fmt::Write;
use core::str::FromStr;
use heapless::String;
use heapless::Vec;

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

    fn write(&mut self, data: &[u8]){
        let mut write_offset = 0;
        let count = data.len();
        while write_offset < count {
            match self.serial.write(&data[write_offset..count]) {
                Ok(len) if len > 0 => {
                    write_offset += len;
                }
                _ => {}
            }
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
                    self.write(&buf[..count]);

                    let command = core::str::from_utf8(&self.com_buf[0..self.com_indx]).unwrap_or_default();
                    if command.contains("\r"){
                        let end = command.chars().position(|c| c == '\r').unwrap_or_default();
                        let command = &command[..end];

                        match Self::process_command(command, chargers){
                            Some(reply) => {
                                self.write(reply.as_bytes());

                            },
                            _ => {}
                        }
                        self.com_indx = 0;
                    }

                }
                _ => {}
            }
        }

    }

    fn process_command(command: &str, chargers: &mut [EVCharger; 4]) -> Option<String<COM_MAX_LEN>>{
        rprintln!("command is: {}",  command);

        if command == "get_units" {
            return units_reply(chargers);
        };

        if command.starts_with("set_units["){
            let command = command.trim_start_matches("set_units[").trim_end_matches("]");
            let ids: Vec<&str,10> = command.split(",").collect();

            if ids.len() == 4 {
                let ids: Vec<u8,4> = ids
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect();

                if is_unique(&ids){
                    for i in 0..4{
                        chargers[i].set_id(ids[i]);
                    }

                    return units_reply(chargers);
                }
            }

            let reply = String::from_str("Invalid!\r\nSyntax: set_units[1,2,3,4]").unwrap();
            return Some(reply);
        }

        None
    }

}

fn units_reply(chargers: &mut [EVCharger; 4]) -> Option<String<COM_MAX_LEN>>{
    let mut reply: String<COM_MAX_LEN> = String::new();
    let _ = write!(reply,
        "units[{}, {}, {}, {}]\r\n",
        chargers[0].get_id(),
        chargers[1].get_id(),
        chargers[2].get_id(),
        chargers[3].get_id(), );

    Some(reply)
}

fn is_unique(ids: &Vec<u8, 4>) -> bool{
    for i in 0..3 {
        let vi = ids[i];
        for j in i+1..4 {
            if vi == ids[j]{
                return false;
            }
        }
    };
    return true;
}

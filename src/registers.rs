
use crate::modbus::*;

#[derive(PartialEq, Debug)]
enum RegType {
    Read,
    ReadWrite
}

#[derive(PartialEq, Debug)]
struct Register {
    addr: u16,
    value: u16,
    reg_type: RegType
}

impl Register {

    fn read(&self, request: &ModbusFrame) -> ModbusFrame{
        ModbusFrame::new(
            request.unit_id,
            request.command,
            Reference::Size(2),
            self.value
        )
    }

    fn write(&mut self, request: &ModbusFrame) -> ModbusFrame{
        self.value = request.value;
        ModbusFrame::new(
            request.unit_id,
            request.command,
            Reference::Address(self.addr),
            self.value
        )
    }

}

#[derive(PartialEq, Debug)]
pub struct RegisterSet {
    pub unit_id: u8,
    registers: [Register; 3]
}

impl RegisterSet {

    pub fn new(unit_id: u8) -> Self {
        Self {
            unit_id,
            registers: [
                Register{addr: 0x3040, value: 0x0001, reg_type: RegType::Read},
                Register{addr: 0x4010, value: 0x0000, reg_type: RegType::ReadWrite},
                Register{addr: 0x4012, value: 0x0000, reg_type: RegType::ReadWrite}
            ]
        }
    }

    pub fn query(&mut self, request: &ModbusFrame ) -> Result<ModbusFrame, &str> {

        if request.unit_id != self.unit_id {return Err("not for this unit")};

        let addr = match request.refers {
            Reference::Address(addr) => {addr},
            _ => {return Err("not for this unit")}
        };

        for reg in &mut self.registers {
            if reg.addr == addr {
                match request.command {
                    3 | 4  if reg.reg_type == RegType::Read => {
                        return Ok(reg.read(request));
                    },
                    6  if reg.reg_type == RegType::ReadWrite => {
                        return Ok(reg.write(request));
                    },
                    _ => {return Err("unsuported operation")}
                };
            }
        };

        Err("unknown register")

    }

}

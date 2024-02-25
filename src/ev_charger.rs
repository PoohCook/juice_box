use crate::{KeyEvent, LightPorts, TM1638};
use smart_leds::RGB8;
use crate::Colors;

use crate::modbus::*;

const CURRENT_STATE: u16 = 0x3040;
const CHARGE_CONTROL: u16 = 0x4010;
const SERVICE_CONTROL: u16 = 0x4012;

struct Registers{
    current_state: u16,
    charge_control: u16,
    service_control: u16,
}

pub struct EVCharger {
    unit_id: u8,
    ui_bank: u8,
    update: bool,
    service_key: u8,
    aux_key: u8,
    key_colors: [RGB8; 2],
    registers: Registers
}

impl EVCharger {
    pub fn new(unit_id: u8, ui_bank: u8) -> Self {
        Self {
            unit_id,
            ui_bank,
            // status: ChgState::Wait,
            update: true,
            service_key: (ui_bank * 2) + 1,
            aux_key: (ui_bank * 2) + 2,
            key_colors: [RGB8::default(), RGB8::default()],
            registers: Registers {
                current_state: 0x0001,
                charge_control: 0x0000,
                service_control: 0x0000
            }
        }
    }

    pub fn on_key_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::KeyDown { key } if *key == self.service_key => {
                self.advance_state();
            },
            KeyEvent::KeyUp { key } if *key == self.service_key => {
                self.update = true;
            },
            KeyEvent::KeyDown { key } if *key == self.aux_key => {
                self.key_colors[1] = Colors::Blue.as_rgb();
                self.update = true;
            },
            KeyEvent::KeyUp { key } if *key == self.aux_key => {
                self.key_colors[1] = Colors::Black.as_rgb();
                self.update = true;
            },
            _ => {}
        }
    }

    fn get_state(&self) -> UnitState{
        UnitState::from(self.registers.current_state)
    }

    pub fn advance_state(&mut self) {
        match self.get_state() {
            unit  if unit.charger == ChgState::Standby => {
                self.registers.current_state = 0x8103;
                self.update = true;
            },
            unit  if unit.charger == ChgState::Connect => {
                self.registers.current_state = 0x8104;
                self.update = true;
            },
            unit  if unit.charger == ChgState::Charge => {
                self.registers.current_state = 0x8002;
                self.update = true;
            },
            _ => {}
        }
    }

    fn update_led_status(&self, light_ports: &mut LightPorts){
        match self.get_state() {
            unit  if unit.charger == ChgState::Wait => {
                light_ports.set_bar(self.ui_bank, Colors::Green.as_rgb(), false).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Black.as_rgb(), true).unwrap();
            },
            unit  if unit.charger == ChgState::Standby => {
                light_ports.set_bar(self.ui_bank, Colors::Yellow.as_rgb(), true).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Yellow.as_rgb(), true).unwrap();
            },
            unit  if unit.charger == ChgState::Connect => {
                light_ports.set_bar(self.ui_bank, Colors::Orange.as_rgb(), true).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Orange.as_rgb(), true).unwrap();
           },
           unit  if unit.charger == ChgState::Charge => {
                light_ports.set_bar(self.ui_bank, Colors::Orange.as_rgb(), false).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Orange.as_rgb(), false).unwrap();
            },
            _ => {}
        }
    }

    pub fn refresh_ui(&mut self, display: &mut TM1638, light_ports: &mut LightPorts) -> bool {
        if self.update{
            display.display_num(self.ui_bank, self.unit_id);

            self.update_led_status(light_ports);

            light_ports.set_button(self.ui_bank, 1, self.key_colors[1], false).unwrap();

            self.update = false;

            return true;
        }
        false
    }

    pub fn query(&mut self, request: &ModbusFrame ) -> Result<ModbusFrame, &str> {

        if request.unit_id != self.unit_id {return Err("not for this unit")};

        let addr = match request.refers {
            Reference::Address(addr) => {addr},
            _ => {return Err("invalid query")}
        };

        match (request.command, addr) {
            (4, CURRENT_STATE)  => { Ok(request.read_reply(self.registers.current_state)) },
            (3, CHARGE_CONTROL)  => { Ok(request.read_reply(self.registers.charge_control)) },
            (3, SERVICE_CONTROL)  => { Ok(request.read_reply(self.registers.service_control)) },
            (6, CHARGE_CONTROL)  => {
                if self.registers.charge_control != request.value {
                    let mut state = UnitState::from(self.registers.current_state);
                    state.set_charge_control(request.value);
                    state.changed = true;
                    self.registers.current_state = state.to();
                }
                self.registers.charge_control = request.value;
                self.update = true;
                Ok(request.write_reply(self.registers.charge_control))
            },
            (6, SERVICE_CONTROL)  => {
                if self.registers.service_control != request.value {
                    let mut state = UnitState::from(self.registers.current_state);
                    state.set_service_control(request.value);
                    state.changed = true;
                    self.registers.current_state = state.to();
                }
                self.registers.service_control = request.value;
                self.update = true;
                Ok(request.write_reply(self.registers.service_control))
            },
            _ => {return Err("invalid operation")}
        }

    }

    pub fn get_id(&self) -> u8{
        self.unit_id
    }

    pub fn set_id(&mut self, new_id: u8) -> u8{
        self.unit_id = new_id;
        self.update = true;
        self.registers.current_state = 0x0001;
        self.registers.charge_control = 0x0000;
        self.registers.service_control = 0x0000;
        self.unit_id
    }


}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum ChgState{
    Boot,
    Wait,
    Standby,
    Connect,
    Charge,
    Outage,
    Abnormal,
    Stop,
    Reboot,
    Unknown,
}

impl ChgState{
    fn from(value: u16) -> Self{
        match value & 0x000f {
            0x00 => { Self::Boot },
            0x01 => { Self::Wait },
            0x02 => { Self::Standby },
            0x03 => { Self::Connect },
            0x04 => { Self::Charge },
            0x05 => { Self::Outage },
            0x06 => { Self::Abnormal },
            0x07 => { Self::Stop },
            0x08 => { Self::Reboot },
            _ => { Self::Unknown },
        }
    }

    fn to(&self) -> u16{
        match self {
            Self::Boot => { 0x00 },
            Self::Wait => { 0x01 },
            Self::Standby => { 0x02 },
            Self::Connect => { 0x03 },
            Self::Charge => { 0x04 },
            Self::Outage => { 0x05 },
            Self::Abnormal => { 0x06 },
            Self::Stop => { 0x07 },
            Self::Reboot => { 0x08 },
            _ => { 0x0f },
        }
    }

}

#[derive(PartialEq, Debug)]
pub enum ErrState{
    Norminal,
    ErrVoltage,
    ErrCurrent,
    ErrTemperature,
    ErrRelay,
    ErrCplt,
    ErrUnknown,
}

impl ErrState{
    fn from(value: u16) -> Self{
        match value & 0x00f0 {
            0x000 => { Self::Norminal },
            0x010 => { Self::ErrVoltage },
            0x020 => { Self::ErrCurrent },
            0x030 => { Self::ErrTemperature },
            0x040 => { Self::ErrRelay },
            0x050 => { Self::ErrCplt },
            _ => { Self::ErrUnknown },
        }
    }
    fn to(&self) -> u16{
        match self {
            Self::Norminal => { 0x000 },
            Self::ErrVoltage => { 0x010 },
            Self::ErrCurrent => { 0x020 },
            Self::ErrTemperature => { 0x030 },
            Self::ErrRelay => { 0x040 },
            Self::ErrCplt => { 0x050 },
            _ => { 0x0f0 },
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct UnitState{
    charger: ChgState,
    error: ErrState,
    connected: bool,
    changed: bool,
}

impl UnitState{
    fn from(value: u16) -> Self{
        Self {
            charger: ChgState::from(value),
            error: ErrState::from(value),
            connected: (value & 0x100) != 0,
            changed: (value & 0x8000) != 0
        }
    }

    fn to(&self) -> u16 {
        let mut value = self.charger.to() + self.error.to();
        if self.connected {value += 0x100;}
        if self.changed {value += 0x8000;}
        value
    }

    fn set_charge_control(&mut self, value: u16){
        match value{
            0x0001 => {self.charger = ChgState::Standby},
            _ => {self.charger = ChgState::Wait}
        }
    }

    fn set_service_control(&mut self, value: u16){
        match value{
            0x0001 => {self.charger = ChgState::Outage},
            _ => {self.charger = ChgState::Wait}
        }
    }

}

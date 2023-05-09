use rppal::gpio::{Gpio, OutputPin};
use once_cell::sync::Lazy;

const GPIO:Lazy<rppal::gpio::Gpio> = Lazy::new(|| Gpio::new().unwrap());
const RELAY_ADDRESSES: [u8;10] = [4,5,6,12,13,17,18,19,20,26];

pub struct GpioFacade{
    unassigned_addresses:Vec<u8>
}

pub struct Relay{
    relay:Box<OutputPin>
}

impl Relay{
    pub fn new(pin:OutputPin) -> Self{
        Self{ relay:Box::new(pin) }
    }
    pub fn low(&mut self) -> &mut Self{
        self.relay.set_low();
        return self;
    }
    pub fn high(&mut self) -> &mut Self{
        self.relay.set_high();
        return self;
    }
    pub fn address(&mut self) -> u8 {
        return self.relay.pin();
    }
}

impl GpioFacade{
    pub fn new() -> Self {
        let mut output = Self { unassigned_addresses:Vec::new() };
        for pin in RELAY_ADDRESSES.iter(){
            output.unassigned_addresses.push(*pin);
            //output.unassigned_addresses.push(Box::new(Relay::new(GPIO.get(*pin).unwrap().into_output())));
        }
        return output;
    }
    
    pub fn remove_pin(&mut self, address:u8) -> Option<Relay>{
        let mut removed_address:u8 = 0;
        for unassigned_address in self.unassigned_addresses.iter_mut() {
            if address == *unassigned_address{
                removed_address = address;
            }
        }
        if removed_address > 0{
            self.unassigned_addresses.retain(|&assigned_address| assigned_address != removed_address );
            return Some(Relay::new(GPIO.get(removed_address).unwrap().into_output()));
        }
        return None;
    }

    pub fn get_unassigned_addresses(&mut self) -> &mut Vec<u8>{
        return &mut self.unassigned_addresses;
    }
}

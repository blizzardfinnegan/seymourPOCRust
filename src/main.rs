use seymour_poc_rust::{device::Device, tty::{self,TTY,Response},gpio_facade::GpioPins};
use std::{io::{stdin,stdout,Write},thread::{self, JoinHandle},path::Path,fs};
use chrono::{DateTime,Local};

const VERSION:&str="2.0.1";

fn int_input_filtering(prompt:Option<&str>) -> u64{
    let internal_prompt = prompt.unwrap_or(">>>");
    let mut user_input:String = String::new();
    print!("{}",internal_prompt);
    _ = stdout().flush();
    stdin().read_line(&mut user_input).expect("Did not input a valid number.");
    if let Some('\n')=user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r')=user_input.chars().next_back() {
        user_input.pop();
    }
    return user_input.parse().unwrap_or(0);
}

fn input_filtering(prompt:Option<&str>) -> String{
    let internal_prompt = prompt.unwrap_or(">>>");
    let mut user_input:String = String::new();
    print!("{}",internal_prompt);
    _ = stdout().flush();
    stdin().read_line(&mut user_input).ok().expect("Did not enter a correct string");
    if let Some('\n')=user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r')=user_input.chars().next_back() {
        user_input.pop();
    }
    log::debug!("{}:{}",internal_prompt,user_input);
    return user_input;
}

fn main(){
    setup_logs();
    log::info!("Seymour Life Testing version: {}",VERSION);
    let gpio = &mut GpioPins::new();
    match std::fs::read_dir("/dev/serial/by-path"){
        Ok(available_ttys)=>{
            let mut possible_devices:Vec<Option<Device>> = Vec::new();
            let mut tty_test_threads:Vec<JoinHandle<Option<Device>>> = Vec::new();
            for possible_tty in available_ttys.into_iter(){
                tty_test_threads.push(
                    thread::spawn(move ||{
                        let tty_ref = possible_tty.as_ref();
                        match tty_ref{
                            Ok(tty_real_ref)=>{
                                let tty_path =  tty_real_ref.path();
                                let tty_name = tty_path.to_string_lossy();
                                log::info!("Testing port {}. This may take a moment...",&tty_name);
                                let possible_port = TTY::new(&tty_name);
                                match possible_port{
                                    Some(mut port) =>{
                                        port.write_to_device(tty::Command::Newline);
                                        let response = port.read_from_device(Some(":"));
                                        if response != Response::Empty{
                                            log::debug!("{} is valid port!",tty_name);
                                            let new_device = Device::new(port,Some(response));
                                            match new_device{
                                                Ok(device) => Some(device),
                                                Err(_) => None
                                            }
                                        }
                                        else { None }
                                    },
                                    None=>{None}
                                }
                            },
                            Err(error)=>{
                                //log::warn!("Invalid TTY location");
                                log::debug!("{}",error);
                                None
                            }
                        }
                }));
            }
            for thread in tty_test_threads{
                let output = thread.join().unwrap_or_else(|x|{log::trace!("{:?}",x); None});
                possible_devices.push(output);
            }

            let mut devices:Vec<Device> = Vec::new();
            for possible_device in possible_devices.into_iter(){
                if let Some(device) = possible_device{
                    devices.push(device);
                }
            }

            log::info!("Number of devices detected: {}",devices.len());

            log::info!("Dimming all screens...");
            for device in devices.iter_mut(){
                device.darken_screen();
            }

            for device in devices.iter_mut(){
                device.brighten_screen()
                    .set_serial(&input_filtering(Some("Enter the serial of the device with the bright screen: ")).to_string())
                .darken_screen();
                log::debug!("Number of unassigned addresses: {}",gpio.get_unassigned_addresses().len());
                for &address in gpio.get_unassigned_addresses(){
                    device.set_pin_address(address).start_temp();
                    if device.is_temp_running(){
                        device.stop_temp();
                        gpio.remove_address(address);
                        break;
                    }
                    else{
                        device.stop_temp();
                    }
                }
            }

            let mut iteration_count:u64 = 0;
            while iteration_count < 1{
                iteration_count = int_input_filtering(Some("Enter the number of iterations to complete: "));
            }

            let mut iteration_threads = Vec::new();
            while let Some(mut device) = devices.pop(){
                iteration_threads.push(thread::spawn(move||{
                    for i in 1..=iteration_count{
                        log::info!("Starting iteration {} of {} for device {}...",
                                       i,iteration_count,device.get_serial());
                        device.test_cycle(None, None);
                    }
                }));
            }
            for thread in iteration_threads{
                thread.join().unwrap();
            }
        }
        Err(_)=>{
            log::error!("Invalid serial location! Please make sure that /dev/serial/by-path exists.");
        }
    }
}

pub fn setup_logs(){
    let chrono_now: DateTime<Local> = Local::now();
    if ! Path::new("logs").is_dir(){
        _ = fs::create_dir("logs");
    };
    _ = fern::Dispatch::new()
        .format(|out,message,record|{
            out.finish(format_args!(
                "{} - [{}, {}] - {}",
                Local::now().to_rfc3339(),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Trace)
                .chain(fern::log_file(
                    format!("logs/{0}.log",
                    chrono_now.format("%Y-%m-%d_%H.%M").to_string()
                    )).unwrap()),
        )
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .chain(std::io::stdout())
        )
        .apply();
}

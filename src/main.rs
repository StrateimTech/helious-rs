use std::path::Path;
use std::thread;
use figlet_rs::FIGfont;

use hid_api_rs::{hid, HidMouse, HidSpecification};
use structopt::StructOpt;
use terminal_size::terminal_size;
use thread_priority::{set_current_thread_priority, ThreadPriority};

use crate::modules::{recoil, server};
use crate::modules::recoil::RecoilSettings;

mod modules;

#[derive(StructOpt, Debug)]
#[structopt(name = "helious-rs", no_version, author = "Etho")]
#[structopt(setting = structopt::clap::AppSettings::DeriveDisplayOrder)]
#[structopt(setting = structopt::clap::AppSettings::DisableVersion)]
struct Cmds {
    #[structopt(flatten)]
    recoil_settings: RecoilSettings,

    #[structopt(short, long, short = "ut")]
    /// Start uart port receiver
    pub uart_receiver: bool,

    #[structopt(long, default_value = "/dev/ttyAMA0", required_if("uart-receiver", "true"))]
    /// UART data receiver file
    pub uart_port: String,

    #[structopt(short, long, short = "ss")]
    /// Start remote state sending
    pub state_sender: bool,

    #[structopt(long, default_value = "192.168.68.62", required_if("state-sender", "true"))]
    /// Remote address for state receiver client
    pub state_port: String
}

fn main() {
    let terminal_size = terminal_size().unwrap();

    let alligator = include_str!("Alligator.flf");
    let alligator_fig_font = FIGfont::from_content(alligator).unwrap();
    let alligator_text = alligator_fig_font.convert("HELIOUS").unwrap();

    let mut rs: Vec<String> = vec![];
    for i in 0..alligator_text.height {
        let mut first_letter: bool = true;
        let mut width = 0;
        let mut rs_lines: Vec<String> = vec![];

        for character in &alligator_text.characters {
            width += character.width;

            if let Some(line) = character.characters.get(i as usize) {
                let red_index = {
                    if i == 0 {
                        25
                    } else {
                        25 + (6 * i)
                    }
                };

                let color = format!("\x1b[38;2;255;{};64m", red_index);

                let formated = match first_letter {
                    true => format!("{}{}", color, line),
                    false => line.to_string()
                };

                rs_lines.push(formated);
            }
            first_letter = false;
        }

        let offset = (terminal_size.0.0 - (width as u16)) / 2 + 1;

        for (index, rs_line) in rs_lines.iter().enumerate() {
            let mut line = rs_line.clone();

            if index == 0 {
                line = " ".repeat(offset as usize) + &*line;
            }

            rs.push(line);
        }

        rs.push(String::from("\n"));
    }
    println!("\n{}\x1b[0m", rs.join(""));

    let mut sub_text = String::from("> Build: 0.1.0u | Stable | Author: Etho <");
    let offset = (terminal_size.0.0 - (sub_text.chars().count() as u16)) / 2 + 1;
    sub_text = " ".repeat(offset as usize) + &*sub_text;

    println!("{}\n", sub_text);

    let specification: HidSpecification = HidSpecification {
        mouse_inputs: Some(
            vec![
                HidMouse {
                    mouse_path: String::from("/dev/input/mice"),
                    mouse_poll_rate: Some(1000),
                    mouse_side_buttons: true
                }
            ]
        ),
        keyboard_inputs: None,
        gadget_output: String::from("/dev/hidg0"),
    };

    if let Err(err) = hid_api_rs::start_pass_through(specification) {
        println!(">> Failed to start device pass-through! \n   --> {}, \"/dev/hidg0\"", err);
        
        if !cfg!(target_os = "linux") {
            println!("   --> Not running linux based OS.")
        }
        
        return;
    };

    let arguments = Cmds::from_args();
    println!("{:#?}", arguments);

    if arguments.recoil_settings.vertical != 0.0 {
        println!("{:#?}", arguments.recoil_settings);
        println!(">> Running Recoil Handler");
        thread::spawn(|| recoil::start_recoil_handler(arguments.recoil_settings, hid::open_gadget_device(String::from("/dev/hidg1")).unwrap()));
    }
    
    if arguments.state_sender {
        println!(">> Running State Sender | {}", &arguments.state_port);
        thread::spawn(move || server::start_state_sender(&arguments.state_port, 7484));
    }

    let local_ip = "192.168.68.54";
    println!(">> Running Local Receiver | {}", &local_ip);
    thread::spawn(|| {
        if let Err(err) = set_current_thread_priority(ThreadPriority::Max) {
            println!("Failed to set max thread priority, {}", err);
        }

        server::start_local_server(local_ip, 7483, hid::open_gadget_device(String::from("/dev/hidg1")).unwrap())
    });

    if arguments.uart_receiver {
        println!(">> Running Local UART Receiver | {}", &arguments.uart_port);
        thread::spawn(move || {
            if let Err(err) = set_current_thread_priority(ThreadPriority::Max) {
                println!("Failed to set max thread priority, {}", err);
            }

            server::start_local_uart_server(Path::new(&arguments.uart_port), hid::open_gadget_device(String::from("/dev/hidg1")).unwrap());
        });
    }

    loop {}
}

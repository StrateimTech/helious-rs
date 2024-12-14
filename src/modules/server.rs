use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read};
use std::net::UdpSocket;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use hid_api_rs::gadgets::mouse;
use hid_api_rs::gadgets::mouse::{MouseRaw, MouseState};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub fn start_local_uart_server(uart_path: &Path, gadget_file: File) {
    if !uart_path.exists() {
        println!("UART Path does not exist.");
        return;
    }

    if let Ok(mut file) = OpenOptions::new()
        .read(true)
        .write(true)
        .open(uart_path) {
        let mut gadget_writer = BufWriter::with_capacity(8, gadget_file);
        let mut buf: [u8; 4] = [0; 4];
        let mouses = hid_api_rs::get_mouses();

        let mut last_time: u128 = 0;

        loop {
            match &file.read(&mut buf) {
                Ok(value) => {
                    if value != &0 {
                        if mouses.is_empty() {
                            return;
                        }

                        let mouse_x = ((buf[1] as i16) << 8) | (buf[0] as i16);
                        let mouse_y = ((buf[3] as i16) << 8) | (buf[2] as i16);

                        let duration_size = SystemTime::now().duration_since(UNIX_EPOCH).expect("error").as_nanos();
                        println!("UART Latency: {}, {}, {}", Decimal::from(duration_size - last_time) / dec!(1000000.0), mouse_x, mouse_y);
                        last_time = duration_size;

                        let mouse_state = mouses[0].get_state();

                        if mouse_state.right_button {
                            let mouse_raw = MouseRaw {
                                relative_x: mouse_x,
                                relative_y: mouse_y,
                                left_button: mouse_state.left_button,
                                right_button: mouse_state.right_button,
                                middle_button: mouse_state.middle_button,
                                ..MouseRaw::default()
                            };

                            if let Err(error) = mouse::push_mouse_event(mouse_raw, None, &mut gadget_writer) {
                                println!("Failed to push mouse event: {error}");
                            };
                        }
                    }
                },
                Err(_) => continue
            };
        }
    }
}

pub fn start_local_server(address: &str, port: u16, gadget_file: File) {
    let socket = UdpSocket::bind((address, port)).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut gadget_writer = BufWriter::with_capacity(8, gadget_file);
    let mut buf: [u8; 4] = [0; 4];

    let mouses = hid_api_rs::get_mouses();

    let mut mouse_x: i16;
    let mut mouse_y: i16;

    loop {
        if let Ok(size) = socket.recv(&mut buf) {
            if size != 0 {
                mouse_x = ((buf[1] as i16) << 8) | (buf[0] as i16);
                mouse_y = ((buf[3] as i16) << 8) | (buf[2] as i16);

                let mouse_state = mouses[0].get_state();

                // let ignore_aim = buf[4] > 0;
                // let mut left = buf[5] > 0;
                // let mut right = buf[6] > 0;
                // let mut middle = buf[7] > 0;
                //
                // let mouse_state = mouses[0].get_state();
                //
                // if !left {
                //     left = mouse_state.left_button;
                // }
                //
                // if !right {
                //     right = mouse_state.right_button;
                // }
                //
                // if !middle {
                //     middle = mouse_state.middle_button;
                // }
                //
                // if mouse_state.right_button || ignore_aim {
                if mouse_state.right_button {
                    let mouse_raw = MouseRaw {
                        relative_x: mouse_x,
                        relative_y: mouse_y,
                        left_button: mouse_state.left_button,
                        right_button: mouse_state.right_button,
                        middle_button: mouse_state.middle_button,
                        ..MouseRaw::default()
                    };

                    if let Err(error) = mouse::push_mouse_event(mouse_raw, None, &mut gadget_writer) {
                        println!("Failed to push mouse event: {error}");
                    };
                }
            }
        }
    }
}

pub fn start_state_sender(address: &str, port: u16) {
    let mut buffer: [u8; 9] = [0; 9];
    let mouse_socket = UdpSocket::bind("0.0.0.0:34254").unwrap();

    let mut last_state = &MouseState::default();

    let mouses = hid_api_rs::get_mouses();

    mouse_socket.connect((address, port)).unwrap();

    loop {
        if mouses.is_empty() {
            continue;
        }

        let mouse_state = mouses[0].get_state();
        let movement = mouses[0].get_movement();

        match movement.try_recv() {
            Ok(movement) => {
                buffer[0] = match mouse_state.left_button {
                    true => 1u8,
                    false => 0u8
                };

                buffer[1] = match mouse_state.right_button {
                    true => 1u8,
                    false => 0u8
                };

                buffer[2] = match mouse_state.middle_button {
                    true => 1u8,
                    false => 0u8
                };

                buffer[3] = match mouse_state.four_button {
                    true => 1u8,
                    false => 0u8
                };

                buffer[4] = match mouse_state.five_button {
                    true => 1u8,
                    false => 0u8
                };

                buffer[5] = (movement.relative_x >> 0) as u8;
                buffer[6] = (movement.relative_x >> 8) as u8;

                buffer[7] = (movement.relative_y >> 0) as u8;
                buffer[8] = (movement.relative_y >> 8) as u8;

                _ = mouse_socket.send(&buffer);
            }
            Err(_) => {
                if last_state.left_button != mouse_state.left_button || last_state.right_button != mouse_state.right_button {
                    buffer[0] = match mouse_state.left_button {
                        true => 1u8,
                        false => 0u8
                    };

                    buffer[1] = match mouse_state.right_button {
                        true => 1u8,
                        false => 0u8
                    };

                    buffer[2] = match mouse_state.middle_button {
                        true => 1u8,
                        false => 0u8
                    };

                    buffer[3] = match mouse_state.four_button {
                        true => 1u8,
                        false => 0u8
                    };

                    buffer[4] = match mouse_state.five_button {
                        true => 1u8,
                        false => 0u8
                    };

                    buffer[5] = (0 >> 0) as u8;
                    buffer[6] = (0 >> 8) as u8;

                    buffer[7] = (0 >> 0) as u8;
                    buffer[8] = (0 >> 8) as u8;

                    _ = mouse_socket.send(&buffer);
                }
            }
        };
        last_state = mouse_state;
    }
}
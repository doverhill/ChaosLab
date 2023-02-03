use crate::{ StormError, ServiceHandle, ChannelHandle, StormAction, StormEvent };

use std::io::prelude::*;
use std::net::{TcpStream, Shutdown};
use std::sync::Mutex;
use uuid::Uuid;

#[allow(dead_code)]
enum SyscallNumber {
    ServiceCreate = 10,
    ServiceDestroy = 11,
    ServiceConnect = 12,

    ChannelDestroy = 21,
    ChannelMessage = 22,

    EventWait = 30,

    ProcessCreate = 40,
    ProcessDestroy = 41,
    ProcessSetInfo = 42,
    ProcessEmit = 43,

    ThreadCreate = 50,
    ThreadDestroy = 51
}

#[allow(dead_code)]
pub enum EmitType {
    Error = 1,
    Warning = 2,
    Information = 3,
    Debug = 4
}

lazy_static! {
    static ref KERNEL_CONNECTION: Mutex<TcpStream> = {
        let connection = TcpStream::connect("127.0.0.1:1337").unwrap();
        Mutex::new(connection)
    };
}

pub fn service_create(protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<ServiceHandle, StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ServiceCreate as i32);
    write_text(connection, Some(protocol_name));
    write_text(connection, Some(vendor_name));
    write_text(connection, Some(device_name));
    write_uuid(connection, Some(device_id));

    let result = StormError::from_i32(read_i32(connection));
    if result == StormError::None {
        Ok(ServiceHandle(read_u64(connection)))
    }
    else {
        Err(result)
    }
}

pub fn service_destroy(handle: ServiceHandle) -> Result<(), StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ServiceDestroy as i32);
    write_u64(connection, handle.raw_handle());

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(()),
        error => Err(error),
    }
}

pub fn service_connect(protocol_name: &str, vendor_name: Option<&str>, device_name: Option<&str>, device_id: Option<Uuid>) -> Result<ChannelHandle, StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ServiceConnect as i32);
    write_text(connection, Some(protocol_name));
    write_text(connection, vendor_name);
    write_text(connection, device_name);
    write_uuid(connection, device_id);

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(ChannelHandle(read_u64(connection))),
        error => Err(error),
    }
}

pub fn channel_destroy(handle: ChannelHandle) -> Result<(), StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ChannelDestroy as i32);
    write_u64(connection, handle.raw_handle());

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(()),
        error => Err(error),
    }
}

pub fn channel_message(handle: ChannelHandle, message: u64) -> Result<(), StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ChannelMessage as i32);
    write_u64(connection, handle.raw_handle());
    write_u64(connection, message);

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(()),
        error => Err(error),
    }
}

pub fn event_wait(handle: Option<u64>, action: Option<StormAction>, message: Option<u64>, timeout_milliseconds: i32) -> Result<StormEvent, StormError> {
    println!("syscall: event_wait");
    let send_action = match action {
        Some(action) => Some(action.to_i32()),
        None => None,
    };

    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    println!("got connection");
    write_i32(connection, SyscallNumber::EventWait as i32);
    write_optional_u64(connection, handle);
    write_optional_i32(connection, send_action);
    write_optional_u64(connection, message);
    write_i32(connection, timeout_milliseconds);

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => {
            println!("syscall: event_wait: got success");

            let target_handle = read_u64(connection);
            let argument_handle = read_u64(connection);
            let action = StormAction::from_i32(read_i32(connection));
            let parameter = read_u64(connection);
            Ok(match action {
                StormAction::ServiceConnected => StormEvent::ServiceConnected(ServiceHandle(target_handle), ChannelHandle(argument_handle)),
                StormAction::ChannelMessaged => StormEvent::ChannelMessaged(ChannelHandle(target_handle), parameter),
                StormAction::ChannelDestroyed => StormEvent::ChannelDestroyed(ChannelHandle(target_handle)),
            })
        },
        error => {
            println!("syscall: event_wait: got success");
            Err(error)
        },
    }
}

pub fn process_destroy() -> Result<(), StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ProcessDestroy as i32);

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(()),
        error => Err(error),
    }
}

pub fn process_set_info(process_name: &str) -> Result<(), StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ProcessSetInfo as i32);
    write_text(connection, Some(process_name));

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(()),
        error => Err(error),
    }
}

pub fn process_emit(emit_type: EmitType, error: StormError, text: Option<&str>) -> Result<(), StormError> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ProcessEmit as i32);
    write_i32(connection, emit_type as i32);
    write_i32(connection, StormError::to_i32(error));
    write_text(connection, text);

    match StormError::from_i32(read_i32(connection)) {
        StormError::None => Ok(()),
        error => Err(error),
    }
}

pub fn cleanup() {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    connection.shutdown(Shutdown::Both).unwrap();
}

fn write_i32(mut connection: &TcpStream, value: i32) {
    connection.write(&value.to_ne_bytes()).unwrap();
}

fn write_optional_i32(connection: &TcpStream, value: Option<i32>) {
    if let Some(value) = value {
        write_bool(connection, true);
        write_i32(connection, value);
    }
    else {
        write_bool(connection, false);
    }
}

fn write_u32(mut connection: &TcpStream, value: u32) {
    connection.write(&value.to_ne_bytes()).unwrap();
}

fn write_u64(mut connection: &TcpStream, value: u64) {
    connection.write(&value.to_ne_bytes()).unwrap();
}

fn write_optional_u64(connection: &TcpStream, value: Option<u64>) {
    if let Some(value) = value {
        write_bool(connection, true);
        write_u64(connection, value);
    }
    else {
        write_bool(connection, false);
    }
}

fn write_bool(mut connection: &TcpStream, value: bool) {
    if value {
        connection.write(&[1]).unwrap();
    } else {
        connection.write(&[0]).unwrap();
    }
}

fn write_text(mut connection: &TcpStream, text: Option<&str>) {
    match text {
        Some(value) => {
            write_bool(connection, true);
            write_u32(connection, value.len() as u32);
            connection.write(value.as_bytes()).unwrap();
        },
        None => {
            write_bool(connection, false);
        }
    }
}

fn write_uuid(mut connection: &TcpStream, uuid: Option<Uuid>) {
    match uuid {
        Some(value) => {
            write_bool(connection, true);
            connection.write(value.as_bytes()).unwrap();
        },
        None => {
            write_bool(connection, false);
        }
    }
}

fn read_i32(mut connection: &TcpStream) -> i32 {
    let mut data = [0; 4];
    connection.read(&mut data).unwrap();
    i32::from_ne_bytes(data)
}

fn read_u64(mut connection: &TcpStream) -> u64 {
    let mut data = [0; 8];
    connection.read(&mut data).unwrap();
    u64::from_ne_bytes(data)
}
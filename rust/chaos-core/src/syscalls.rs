use crate::error::Error;
use crate::handle::Handle;
use crate::action::Action;

use std::io::prelude::*;
use std::net::{TcpStream, Shutdown};
use std::sync::Mutex;
use uuid::Uuid;

enum SyscallNumber {
    Debug = 1,
    ServiceCreate = 2,
    ServiceConnect = 3,
    ServiceDestroy = 4,
    ChannelCreate = 5,
    ChannelDestroy = 6,
    EventWait = 7,
    ProcessCreate = 8,
    ProcessEmit = 9,
    ProcessDestroy = 10,
    ThreadCreate = 11,
    ThreadDestroy = 12
}

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

fn write_i32(mut connection: &TcpStream, value: i32) -> () {
    connection.write(&value.to_ne_bytes());
}

fn write_u32(mut connection: &TcpStream, value: u32) -> () {
    connection.write(&value.to_ne_bytes());
}

fn write_u64(mut connection: &TcpStream, value: u64) -> () {
    connection.write(&value.to_ne_bytes());
}

fn write_bool(mut connection: &TcpStream, value: bool) -> () {
    if value {
        connection.write(&[1]);
    } else {
        connection.write(&[0]);
    }
}

fn write_text(mut connection: &TcpStream, text: Option<&str>) -> () {
    match text {
        Some(value) => {
            write_bool(connection, true);
            write_u32(connection, value.len() as u32);
            connection.write(value.as_bytes());
        },
        None => {
            write_bool(connection, false);
        }
    }
}

fn write_uuid(mut connection: &TcpStream, uuid: Option<Uuid>) -> () {
    match uuid {
        Some(value) => {
            write_bool(connection, true);
            connection.write(value.as_bytes());
        },
        None => {
            write_bool(connection, false);
        }
    }
}

fn read_i32(mut connection: &TcpStream) -> i32 {
    let mut data = [0; 4];
    connection.read(&mut data);
    i32::from_ne_bytes(data)
}

fn read_u64(mut connection: &TcpStream) -> u64 {
    let mut data = [0; 8];
    connection.read(&mut data);
    u64::from_ne_bytes(data)
}

pub fn process_emit(emit_type: EmitType, error: Error, text: Option<&str>) -> Option<Error> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ProcessEmit as i32);
    write_i32(connection, emit_type as i32);
    write_i32(connection, error as i32);
    write_text(connection, text);

    let result = Error::from_i32(read_i32(connection)).unwrap();
    if result == Error::None {
        None
    }
    else {
        Some(result)
    }
}

pub fn service_create(protocol_name: &str, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Handle, Error> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ServiceCreate as i32);
    write_text(connection, Some(protocol_name));
    write_text(connection, Some(vendor_name));
    write_text(connection, Some(device_name));
    write_uuid(connection, Some(device_id));

    let result = Error::from_i32(read_i32(connection)).unwrap();
    if result == Error::None {
        Ok(Handle::new(read_u64(connection)))
    }
    else {
        Err(result)
    }
}

pub fn service_connect(protocol_name: &str, vendor_name: Option<&str>, device_name: Option<&str>, device_id: Option<Uuid>) -> Result<Handle, Error> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::ServiceConnect as i32);
    write_text(connection, Some(protocol_name));
    write_text(connection, vendor_name);
    write_text(connection, device_name);
    write_uuid(connection, device_id);

    let result = Error::from_i32(read_i32(connection)).unwrap();
    if result == Error::None {
        Ok(Handle::new(read_u64(connection)))
    }
    else {
        Err(result)
    }
}

pub fn event_wait(timeout_milliseconds: i32) -> Result<(Handle, Action), Error> {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    write_i32(connection, SyscallNumber::EventWait as i32);
    write_i32(connection, timeout_milliseconds);

    let result = Error::from_i32(read_i32(connection)).unwrap();
    if result == Error::None {
        Ok((Handle::new(read_u64(connection)), Action::from_i32(read_i32(connection)).unwrap()))
    }
    else {
        Err(result)
    }
}

pub fn cleanup() -> () {
    let connection = &*KERNEL_CONNECTION.lock().unwrap();
    connection.shutdown(Shutdown::Both);
}
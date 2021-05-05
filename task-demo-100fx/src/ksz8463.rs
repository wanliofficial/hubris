use core::convert::TryInto;
use ringbuf::*;
use userlib::*;

#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum ResponseCode {
    BadArg = 2,
}

const fn register_offset(address: u16) -> u16 {
    let addr10_2 = address >> 2;
    let mask_shift = 2 /* turn around bits */ + (2 * ((address >> 1) & 0x1));
    (addr10_2 << 6) | ((0x3 as u16) << mask_shift)
}

#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq)]
#[repr(u16)]
pub enum Register {
    CIDER = register_offset(0x000),
    SGCR1 = register_offset(0x002),
    SGCR2 = register_offset(0x004),
    SGCR3 = register_offset(0x006),
    SGCR6 = register_offset(0x00c),
    SGCR7 = register_offset(0x00e),
    MACAR1 = register_offset(0x010),
    MACAR2 = register_offset(0x012),
    MACAR3 = register_offset(0x014),
}

#[derive(Copy, Clone, PartialEq)]
enum RegisterAccess {
    None,
    Read(Register),
    Write(Register),
    Data(u16),
    Error(ResponseCode),
}

ringbuf!(RegisterAccess, 16, RegisterAccess::None);

pub fn read(spi: TaskId, r: Register) -> Result<u16, ResponseCode> {
    let cmd = (r as u16).to_be_bytes();
    let request = [cmd[0], cmd[1], 0, 0];
    let mut response: [u8; 4] = [0; 4];

    ringbuf_entry!(RegisterAccess::Read(r));

    let (code, _) = sys_send(
        spi,
        3,
        &[],
        &mut [],
        &[
            Lease::from(&request as &[u8]),
            Lease::from(&mut response as &mut [u8]),
        ],
    );

    match code {
        0 => {
            let v = u16::from_le_bytes(response[2..].try_into().unwrap());
            ringbuf_entry!(RegisterAccess::Data(v));
            Ok(v)
        }
        2 => {
            ringbuf_entry!(RegisterAccess::Error(ResponseCode::BadArg));
            Err(ResponseCode::BadArg)
        }
        _ => panic!("invalid response"),
    }
}

pub fn write(spi: TaskId, r: Register, v: u16) -> Result<(), ResponseCode> {
    let cmd = (r as u16 | 0x8000).to_be_bytes(); // Set MSB to indicate write.
    let data = v.to_le_bytes();
    let request = [cmd[0], cmd[1], data[0], data[1]];

    ringbuf_entry!(RegisterAccess::Write(r));
    ringbuf_entry!(RegisterAccess::Data(v));

    let (code, _) =
        sys_send(spi, 2, &[], &mut [], &[Lease::from(&request as &[u8])]);

    match code {
        0 => Ok(()),
        2 => {
            ringbuf_entry!(RegisterAccess::Error(ResponseCode::BadArg));
            Err(ResponseCode::BadArg)
        }
        _ => panic!("invalid response"),
    }
}

pub fn enabled(spi: TaskId) -> Result<bool, ResponseCode> {
    Ok(read(spi, Register::CIDER)? & 0x1 != 0)
}

pub fn enable(spi: TaskId) -> Result<(), ResponseCode> {
    write(spi, Register::CIDER, 1)?;
    Ok(())
}

pub fn disable(spi: TaskId) -> Result<(), ResponseCode> {
    write(spi, Register::CIDER, 0)?;
    Ok(())
}

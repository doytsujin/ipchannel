use super::*;
use crate::Sender as _;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{convert::TryInto as _, fmt::Debug, io};
use test_case::test_case;

fn check_buf<T>(buf: &[u8], target: T)
where
    T: DeserializeOwned + PartialEq + Debug,
{
    assert_eq!(
        (buf.len() - 8) as u64,
        u64::from_le_bytes(buf[..8].try_into().unwrap()),
    );
    assert_eq!(bincode::deserialize::<T>(&buf[8..]).unwrap(), target);
}

fn check_send<T>(target: T)
where
    T: Serialize + DeserializeOwned + Clone + PartialEq + Debug,
{
    let mut buf = Vec::new();
    let mut sender = Sender::new(&mut buf, 16);
    sender.send(target.clone()).unwrap();
    check_buf(&buf, target);
}

#[test]
fn test_send_string() {
    check_send("foobar".to_string());
}

#[test]
fn test_send_struct() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct CustomStruct {
        field: u32,
        another_field: String,
        third_field: Vec<u32>,
        enum_field: Result<u32, u32>,
        tuple_field: (u32, u32),
    }

    let payload = CustomStruct {
        field: 42,
        another_field: "foobar".into(),
        third_field: vec![1, 2, 3],
        enum_field: Err(5),
        tuple_field: (13, 37),
    };
    check_send(payload)
}

#[test]
fn test_buf_size() {
    let mut buf = Vec::new();
    let mut sender = Sender::new(&mut buf, 16);
    let payload = "a very long string, totally longer than 16 bytes, I promise".to_string();

    assert_eq!(sender.buf.capacity(), 16);
    sender.send(payload.clone()).unwrap();
    assert_eq!(sender.buf.capacity(), 16);
    check_buf(&buf, payload);
}

#[test_case(true, io::ErrorKind::NotFound; "write: not found")]
#[test_case(true, io::ErrorKind::PermissionDenied; "write: permission denied")]
#[test_case(true, io::ErrorKind::ConnectionRefused; "write: connection refused")]
#[test_case(true, io::ErrorKind::ConnectionReset; "write: connection reset")]
#[test_case(true, io::ErrorKind::ConnectionAborted; "write: connection aborted")]
#[test_case(true, io::ErrorKind::NotConnected; "write: not connected")]
#[test_case(true, io::ErrorKind::AddrInUse; "write: address in use")]
#[test_case(true, io::ErrorKind::AddrNotAvailable; "write: address not available")]
#[test_case(true, io::ErrorKind::BrokenPipe; "write: broken pipe")]
#[test_case(true, io::ErrorKind::AlreadyExists; "write: already exists")]
#[test_case(true, io::ErrorKind::InvalidInput; "write: invalid input")]
#[test_case(true, io::ErrorKind::InvalidData; "write: invalid data")]
#[test_case(true, io::ErrorKind::TimedOut; "write: timed out")]
#[test_case(true, io::ErrorKind::WriteZero; "write: write returned zero")]
#[test_case(true, io::ErrorKind::Other; "write: other error")]
#[test_case(true, io::ErrorKind::UnexpectedEof; "write: unexpected eof")]
#[test_case(false, io::ErrorKind::NotFound; "flush: not found")]
#[test_case(false, io::ErrorKind::PermissionDenied; "flush: permission denied")]
#[test_case(false, io::ErrorKind::ConnectionRefused; "flush: connection refused")]
#[test_case(false, io::ErrorKind::ConnectionReset; "flush: connection reset")]
#[test_case(false, io::ErrorKind::ConnectionAborted; "flush: connection aborted")]
#[test_case(false, io::ErrorKind::NotConnected; "flush: not connected")]
#[test_case(false, io::ErrorKind::AddrInUse; "flush: address in use")]
#[test_case(false, io::ErrorKind::AddrNotAvailable; "flush: address not available")]
#[test_case(false, io::ErrorKind::BrokenPipe; "flush: broken pipe")]
#[test_case(false, io::ErrorKind::AlreadyExists; "flush: already exists")]
#[test_case(false, io::ErrorKind::InvalidInput; "flush: invalid input")]
#[test_case(false, io::ErrorKind::InvalidData; "flush: invalid data")]
#[test_case(false, io::ErrorKind::TimedOut; "flush: timed out")]
#[test_case(false, io::ErrorKind::WriteZero; "flush: write returned zero")]
#[test_case(false, io::ErrorKind::Other; "flush: other error")]
#[test_case(false, io::ErrorKind::UnexpectedEof; "flush: unexpected eof")]
fn test_failed_write(error_in_write: bool, kind: io::ErrorKind) {
    struct BadWriter(bool, io::ErrorKind);

    impl io::Write for BadWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.0 {
                Err(io::Error::new(self.1, String::new()))
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.0 {
                panic!("Flushing without write")
            } else {
                Err(io::Error::new(self.1, String::new()))
            }
        }
    }

    let mut sender = Sender::new(BadWriter(error_in_write, kind), 16);
    let err = *sender.send("foobar foobar foobar foobar").unwrap_err();
    if let bincode::ErrorKind::Io(err) = err {
        assert_eq!(err.kind(), kind);
    } else {
        panic!("Sender returned non-io error: {}", err);
    }
    assert_eq!(sender.buf.capacity(), 16);
}

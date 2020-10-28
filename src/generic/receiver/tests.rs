use super::*;
use crate::Receiver as _;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, io};
use test_case::test_case;

fn gen_buf<T>(target: T) -> Vec<u8>
where
    T: Serialize,
{
    let mut result = vec![0; 8];
    bincode::serialize_into(&mut result, &target).unwrap();
    let size = (result.len() - 8) as u64;
    result[..8].copy_from_slice(&u64::to_le_bytes(size));
    result
}

fn check_receive<T>(target: T)
where
    T: Serialize + DeserializeOwned + Clone + PartialEq + Debug,
{
    let buf = gen_buf(&target);
    let mut receiver = Receiver::new(buf.as_slice(), 16);
    let result = receiver.receive().unwrap();
    assert_eq!(target, result);
}

#[test]
fn test_receive_string() {
    check_receive("foobar".to_string());
}

#[test]
fn test_receive_struct() {
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
    check_receive(payload)
}

#[test]
fn test_buf_size() {
    let target =
        "a very very long string which will obviously not fit into Receiver buffer".to_string();
    let buf = gen_buf(target.clone());
    let mut receiver: Receiver<_, String> = Receiver::new(buf.as_slice(), 16);
    assert_eq!(receiver.buf.capacity(), 16);
    assert_eq!(receiver.receive().unwrap(), target);
    assert_eq!(receiver.buf.capacity(), 16);
}

#[test_case(true, io::ErrorKind::NotFound; "read size: not found")]
#[test_case(true, io::ErrorKind::PermissionDenied; "read size: permission denied")]
#[test_case(true, io::ErrorKind::ConnectionRefused; "read size: connection refused")]
#[test_case(true, io::ErrorKind::ConnectionReset; "read size: connection reset")]
#[test_case(true, io::ErrorKind::ConnectionAborted; "read size: connection aborted")]
#[test_case(true, io::ErrorKind::NotConnected; "read size: not connected")]
#[test_case(true, io::ErrorKind::AddrInUse; "read size: address in use")]
#[test_case(true, io::ErrorKind::AddrNotAvailable; "read size: address not available")]
#[test_case(true, io::ErrorKind::BrokenPipe; "read size: broken pipe")]
#[test_case(true, io::ErrorKind::AlreadyExists; "read size: already exists")]
#[test_case(true, io::ErrorKind::InvalidInput; "read size: invalid input")]
#[test_case(true, io::ErrorKind::InvalidData; "read size: invalid data")]
#[test_case(true, io::ErrorKind::TimedOut; "read size: timed out")]
#[test_case(true, io::ErrorKind::WriteZero; "read size: write returned zero")]
#[test_case(true, io::ErrorKind::Other; "read size: other error")]
#[test_case(true, io::ErrorKind::UnexpectedEof; "read size: unexpected eof")]
#[test_case(false, io::ErrorKind::NotFound; "read: not found")]
#[test_case(false, io::ErrorKind::PermissionDenied; "read: permission denied")]
#[test_case(false, io::ErrorKind::ConnectionRefused; "read: connection refused")]
#[test_case(false, io::ErrorKind::ConnectionReset; "read: connection reset")]
#[test_case(false, io::ErrorKind::ConnectionAborted; "read: connection aborted")]
#[test_case(false, io::ErrorKind::NotConnected; "read: not connected")]
#[test_case(false, io::ErrorKind::AddrInUse; "read: address in use")]
#[test_case(false, io::ErrorKind::AddrNotAvailable; "read: address not available")]
#[test_case(false, io::ErrorKind::BrokenPipe; "read: broken pipe")]
#[test_case(false, io::ErrorKind::AlreadyExists; "read: already exists")]
#[test_case(false, io::ErrorKind::InvalidInput; "read: invalid input")]
#[test_case(false, io::ErrorKind::InvalidData; "read: invalid data")]
#[test_case(false, io::ErrorKind::TimedOut; "read: timed out")]
#[test_case(false, io::ErrorKind::WriteZero; "read: write returned zero")]
#[test_case(false, io::ErrorKind::Other; "read: other error")]
#[test_case(false, io::ErrorKind::UnexpectedEof; "read: unexpected eof")]
fn test_failed_read(success_reading_size: bool, kind: io::ErrorKind) {
    struct BadReader(bool, io::ErrorKind);

    impl io::Read for BadReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.0 && buf.len() == 8 {
                buf.copy_from_slice(&128u64.to_le_bytes());
                Ok(8)
            } else {
                Err(io::Error::new(self.1, "".to_string()))
            }
        }
    }

    let mut receiver: Receiver<_, String> =
        Receiver::new(BadReader(success_reading_size, kind), 16);
    assert_eq!(receiver.buf.capacity(), 16);
    let err = *receiver.receive().unwrap_err();
    if let bincode::ErrorKind::Io(err) = err {
        assert_eq!(err.kind(), kind);
    } else {
        panic!("Receiver returned non-io error: {}", err);
    }
}

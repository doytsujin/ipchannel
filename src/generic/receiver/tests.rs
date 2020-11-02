use super::*;
use crate::Receiver as _;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, io};
use test_case::test_case;

fn gen_buf<T>(target: T) -> Vec<u8>
where
    T: Serialize,
{
    bincode::serialize(&target).unwrap()
}

fn check_receive<T>(target: T)
where
    T: Serialize + DeserializeOwned + Clone + PartialEq + Debug,
{
    let buf = gen_buf(&target);
    let mut receiver = Receiver::new(buf.as_slice());
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

#[test_case(io::ErrorKind::NotFound; "read: not found")]
#[test_case(io::ErrorKind::PermissionDenied; "read: permission denied")]
#[test_case(io::ErrorKind::ConnectionRefused; "read: connection refused")]
#[test_case(io::ErrorKind::ConnectionReset; "read: connection reset")]
#[test_case(io::ErrorKind::ConnectionAborted; "read: connection aborted")]
#[test_case(io::ErrorKind::NotConnected; "read: not connected")]
#[test_case(io::ErrorKind::AddrInUse; "read: address in use")]
#[test_case(io::ErrorKind::AddrNotAvailable; "read: address not available")]
#[test_case(io::ErrorKind::BrokenPipe; "read: broken pipe")]
#[test_case(io::ErrorKind::AlreadyExists; "read: already exists")]
#[test_case(io::ErrorKind::InvalidInput; "read: invalid input")]
#[test_case(io::ErrorKind::InvalidData; "read: invalid data")]
#[test_case(io::ErrorKind::TimedOut; "read: timed out")]
#[test_case(io::ErrorKind::WriteZero; "read: write returned zero")]
#[test_case(io::ErrorKind::Other; "read: other error")]
#[test_case(io::ErrorKind::UnexpectedEof; "read: unexpected eof")]
fn test_failed_read(kind: io::ErrorKind) {
    struct BadReader(io::ErrorKind);

    impl io::Read for BadReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(self.0, "".to_string()))
        }
    }

    let mut receiver: Receiver<_, String> = Receiver::new(BadReader(kind));
    let err = *receiver.receive().unwrap_err();
    if let bincode::ErrorKind::Io(err) = err {
        assert_eq!(err.kind(), kind);
    } else {
        panic!("Receiver returned non-io error: {}", err);
    }
}

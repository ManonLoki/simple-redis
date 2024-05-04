use bytes::{Buf, BytesMut};

fn main() {
    let mut data = BytesMut::new();
    data.extend_from_slice(b"Hello World! Hello Rust!");

    assert_eq!(&data[0..], b"Hello World! Hello Rust!");

    // 使用advance 将光标移动到 Hello后的空格
    data.advance(6);

    assert_eq!(&data.chunk(), b"World! Hello Rust!");
}

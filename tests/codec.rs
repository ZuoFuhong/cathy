use protobuf::Message;
use rust_im::chat_room::MsgToUser;

#[test]
fn test_big_endian() {
    let v: u16 = 1234;
    // encode
    let mut b: [u8; 2] = [0; 2];
    b[0] = (v >> 8) as u8;
    b[1] = v as u8;

    println!("{:X}", b[0]);
    println!("{:X}", b[1]);

    // decode
    let dv = b[1] as u16 | (b[0] as u16) << 8;
    println!("{}", dv)
}

#[test]
fn test_overflow() {
    /*
       0000 0100 1101 0010  // 1234补码
                 1101 0010  // 高位截断

                 1101 0010  // 无符号数取补码（就是原码）

                 1010 1101  // 有符号"负数"取反码
                 1010 1110  // 补码的补码就是原码
    */
    let i: i16 = 1234;
    println!("{}", i as u8); // output: 210
    println!("{}", i as i8); // output: -46
}

#[test]
fn test_proto() {
    let mut mtu_pb = MsgToUser::new();
    mtu_pb.seq = 1;
    mtu_pb.from_uid = 2;
    mtu_pb.to_uid = 3;
    mtu_pb.text = "hello".to_string();

    let pb_bytes = mtu_pb.write_to_bytes().unwrap();
    let new_mtu_pb = MsgToUser::parse_from_bytes(pb_bytes.as_slice()).unwrap();
    println!("{:?}", new_mtu_pb)
}

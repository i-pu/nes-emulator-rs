fn main() {
    let a: u8 = 0b00010111;
    // minus
    let b: u8 = 0b10010101;
    println!("u:{:?}, i:{:?}", (a + b), (a + b) as i8);
    let ia: i8 = a as i8;
    let ib: i8 = b as i8;
    println!("u:{:?}, i:{:?}", (ia + ib) as u8, (ia + ib));
    let ib16: u16 = ib as u16;
    let b16: u16 = b as u16;
    println!("ub16: {:?}, ib16: {:?}", b16, ib16);
}

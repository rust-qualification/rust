//@ run-pass
// Field Access Expressions: a80f79e0-2198-433e-951a-7555436fd041

use std::mem;

#[repr(C)]
union MyUnion {
    x: i32,
}

fn main() {
    let mut u = MyUnion { x: 2 };
    u.x = 3;
    unsafe {
        assert_eq!(u.x, 3);

        let read_transmuted: u32 = mem::transmute(u.x);
        let write_transmuted: u32 = mem::transmute(3);
        assert_eq!(read_transmuted, write_transmuted);
    }
}
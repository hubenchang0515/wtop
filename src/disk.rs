// 参考: 可以通过 ioctl  操作 /dev/xxx 获取更详细的信息

use std::ffi::c_char;

pub const DISK_ROOT_FILE:&str = "/";

#[derive(Debug)]
pub struct Disk {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

extern "C" {
    fn disk_stat(path:*const c_char, total:&mut u64, free:&mut u64) -> bool;
}

impl Disk {
    pub fn new() -> Disk {
        Disk {
            total: 0,
            used: 0,
            free: 0,
        }
    }

    pub fn read(path:&str) -> Disk {
        let mut disk = Disk::new();
        unsafe{
            let c_str = std::ffi::CString::new(path).unwrap();
            disk_stat(c_str.as_ptr(), &mut disk.total, &mut disk.free);
        }
        disk.used = disk.total - disk.free;
        disk
    }
}
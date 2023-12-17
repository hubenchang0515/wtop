// 参考: 可以通过 ioctl  操作 /dev/xxx 获取更详细的信息

use std::ffi::c_char;

pub const DISK_LABEL_PATH:&str = "/dev/disk/by-label/";
pub const DISK_MOUNT_FILE:&str = "/proc/mounts";
const MOUNT_ITEMS:usize = 6;

#[derive(Debug)]
pub struct Disk {
    pub label: String,
    pub path: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

extern "C" {
    fn disk_stat(path:*const c_char, total:&mut u64, free:&mut u64) -> bool;
}

impl Disk {
    pub fn new(label:&str) -> Disk {
        let label = String::from(label);
        let label = label.replace("\\040", " ");
        let label = label.replace("\\x20", " ");
        Disk {
            label: label,
            path: String::new(),
            total: 0,
            used: 0,
            free: 0,
        }
    }

    pub fn read(&mut self, path:&str) {
        let path = String::from(path);
        let path = path.replace("\\040", " ");
        let path = path.replace("\\x20", " ");
        unsafe{
            let c_str = std::ffi::CString::new(path).unwrap();
            disk_stat(c_str.as_ptr(), &mut self.total, &mut self.free);
        }
        self.used = self.total - self.free;
    }

    pub fn list_labels(label_dir_path:&str) -> Vec<String> {
        let mut labels = Vec::new();
        for entry in std::fs::read_dir(label_dir_path).unwrap() {
            labels.push(String::from(entry.unwrap().file_name().to_str().unwrap()));
        }
        labels
    }

    pub fn find_device(label_dir_path:&str, label:&str) -> String {
        let dir = std::path::Path::new(label_dir_path);
        let link = std::fs::read_link(dir.join(label)).unwrap();
        String::from(dir.join(link).canonicalize().unwrap().to_str().unwrap())
    }

    pub fn find_path(mount_file_path:&str, device_path:&str) -> String {
        let mut path = String::new();
        let content = std::fs::read_to_string(mount_file_path).unwrap();
        for line in content.split("\n") {
            let items:Vec<&str> = line.split_whitespace().collect();
            if items.len() >= MOUNT_ITEMS && items[0].trim() == device_path {
                path = String::from(items[1]);
                break;
            }
        }
        path
    }
}
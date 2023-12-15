// 参考: https://man7.org/linux/man-pages/man5/proc.5.html 以及 man free
// 文件: /proc/meminfo

pub const MEMINFO_FILE:&str = "/proc/meminfo";
const MEM_ITEMS:usize = 3;

#[derive(Debug)]
pub struct Memory {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub shared: u64,
    pub buffers: u64,
    pub cached: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
    pub swap_cached: u64,
}

impl Memory {
    fn new() -> Memory {
        Memory {
            total: 0,
            used: 0,
            free: 0,
            shared: 0,
            buffers: 0,
            cached: 0,
            available: 0,
            swap_total: 0,
            swap_used: 0,
            swap_free: 0,
            swap_cached: 0,
        }
    }

    pub fn read(path:&str) -> Memory {
        let mut memory = Memory::new();
        let content = std::fs::read_to_string(path).unwrap();
        for line in content.split("\n") {
            let items:Vec<&str> = line.split_whitespace().collect();
            if items.len() < MEM_ITEMS {
                continue;
            }

            if items[0].starts_with("MemTotal") {
                memory.total = items[1].parse().unwrap();
            } else if items[0].starts_with("MemFree") {
                memory.free = items[1].parse().unwrap();
            } else if items[0].starts_with("Shmem") {
                memory.shared = items[1].parse().unwrap();
            } else if items[0].starts_with("Buffers") {
                memory.buffers = items[1].parse().unwrap();
            } else if items[0].starts_with("Cached") || items[0].starts_with("SReclaimable") {
                memory.cached += items[1].parse::<u64>().unwrap();
            } else if items[0].starts_with("MemAvailable") {
                memory.available = items[1].parse().unwrap();
            } else if items[0].starts_with("SwapTotal") {
                memory.swap_total = items[1].parse().unwrap();
            } else if items[0].starts_with("SwapFree") {
                memory.swap_free = items[1].parse().unwrap();
            } else if items[0].starts_with("SwapCached") {
                memory.swap_cached = items[1].parse().unwrap();
            }
        }
        memory.used = memory.total - memory.free - memory.buffers - memory.cached;
        memory.swap_used = memory.swap_total - memory.swap_free - memory.swap_cached;
        memory
    }
}
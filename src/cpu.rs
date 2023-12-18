
// 参考: https://man7.org/linux/man-pages/man5/proc.5.html

pub const CPU_STAT_FILE:&str = "/proc/stat";
pub const CPU_THERMAL_FILE:&str = "/sys/devices/virtual/thermal/thermal_zone0/temp";

const CPU_ITEMS:usize = 11;

#[derive(Debug)]
pub struct CpuCoreTime {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

pub struct CpuCoreTimeDelta(CpuCoreTime);

#[allow(dead_code)]
impl CpuCoreTime {
    pub fn new() -> CpuCoreTime {
        CpuCoreTime{
            user: 0,
            nice: 0,
            system: 0,
            idle: 0,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        }
    }

    pub fn total(&self) -> u64 {
        self.user + 
        self.nice + 
        self.system + 
        self.idle + 
        self.iowait + 
        self.irq + 
        self.softirq + 
        self.steal + 
        self.guest + 
        self.guest_nice
    }

    pub fn delta(&self, time: &CpuCoreTime) -> CpuCoreTimeDelta {
        if self.total() > time.total() {
            CpuCoreTimeDelta {
                0:CpuCoreTime{
                    user: self.user - time.user,
                    nice: self.nice - time.nice,
                    system: self.system - time.system,
                    idle: self.idle - time.idle,
                    iowait: self.iowait - time.iowait,
                    irq: self.irq - time.irq,
                    softirq: self.softirq - time.softirq,
                    steal: self.steal - time.steal,
                    guest: self.guest - time.guest,
                    guest_nice: self.guest_nice - time.guest_nice,
                },
            }
        } else {
            time.delta(self)
        }
    }
}

impl CpuCoreTimeDelta {
    pub fn usage(&self) -> f32 {
        (self.0.total() as f32 - self.0.idle as f32) / self.0.total() as f32 * 100.0
    }
}

#[derive(Debug)]
pub struct CpuCore {
    path: String,
    pub name: String,
}

impl CpuCore {
    pub fn new(path:&str, name:&str) -> CpuCore {
        CpuCore {
            path: String::from(path),
            name: String::from(name),
        }
    }

    pub fn get_time(&self) -> CpuCoreTime {
        let content = std::fs::read_to_string(&self.path).unwrap_or_default();
        let mut time = CpuCoreTime {
            user: 0,
            nice: 0,
            system: 0,
            idle: 0,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        };
        for line in content.split("\n") {
            let items:Vec<&str> = line.split_whitespace().collect();
            if items.len() >= CPU_ITEMS && items[0].trim() == self.name {
                time.user = items[1].parse().unwrap();
                time.nice = items[2].parse().unwrap();
                time.system = items[3].parse().unwrap();
                time.idle = items[4].parse().unwrap();
                time.iowait = items[5].parse().unwrap();
                time.irq = items[6].parse().unwrap();
                time.softirq = items[7].parse().unwrap();
                time.steal = items[8].parse().unwrap();
                time.guest = items[9].parse().unwrap();
                time.guest_nice = items[10].parse().unwrap();
                break;
            }
        }

        time
    }
}

pub struct Cpu {
    pub cores: Vec<CpuCore>
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { 
            cores: Vec::new()
        }
    }

    pub fn scan(&mut self, path:&str) {
        self.cores.clear();
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for line in content.split("\n") {
            let items:Vec<&str> = line.split_whitespace().collect();
            if items.len() >= CPU_ITEMS && items[0].trim().starts_with("cpu") && items[0].trim() != "cpu" {
                self.cores.push(CpuCore::new(path, items[0]));
            }
        }
    }

    pub fn thermal(path:&str) -> f32 {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let content = content.trim();
        content.parse::<f32>().unwrap_or_default() / 1000.0
    }
}

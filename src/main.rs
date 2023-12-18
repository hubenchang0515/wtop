use std::{io::{Write, BufRead}, io::BufReader, time::Duration};
mod cpu;
mod memory;
mod disk;

const INDEX_HTML:&str = include_str!("index.html");
const ECHARTS_JS:&str = include_str!("echarts.min.js");

struct Record {
    index_html: String,
    echarts_js: String,
    cpu_infos: String,
    memory_infos: String,
    disk_infos: String,
}

impl Record {
    fn new() -> Record {
        Record { 
            index_html: String::new(), 
            echarts_js: String::new(),
            cpu_infos: String::new(),
            memory_infos: String::new(),
            disk_infos: String::new(),
        }
    }
}

fn make_http_response(content_type:&str, content:&str) -> String {
    format!("HTTP/1.1 200 OK\r\nContent-Type:{}\r\nContent-Length:{}\r\n\r\n{}", content_type, content.len(), content)
}

fn make_cpu_infos(names:&Vec<String>, usages:&Vec<String>, thermal:f32) -> String {
    let content = format!(r#"{{"labels":[{}],"usages":[{}],"thermal":{}}}"#, 
        names.join(","), 
        usages.join(","),
        thermal);
    make_http_response("text/json", &content)
}

fn make_memory_infos(mem: &memory::Memory) -> String {
    let content = format!(r#"{{"used":{},"buffers":{},"cached":{},"free":{},"swap_used":{},"swap_cached":{},"swap_free":{}}}"#,
        mem.used,
        mem.buffers,
        mem.cached,
        mem.free,
        mem.swap_used,
        mem.swap_cached,
        mem.swap_free);
    make_http_response("text/json", &content)
}

fn make_disk_infos(disks: &Vec<disk::Disk>) -> String {
    let mut content = Vec::new();
    for disk in disks {
        content.push(format!(r#"{{"label":"{}","used":{},"free":{}}}"#,
                                disk.label,
                                disk.used,
                                disk.free));
    }

    let content = content.join(",");
    let content = format!("[{}]",content);
    make_http_response("text/json", &content)
}

fn main() {
    let record = Record::new();
    let rwlock = std::sync::Arc::new(std::sync::RwLock::new(record));
    
    let lock = rwlock.clone();
    std::thread::spawn(move || {
        let mut cpu = cpu::Cpu::new();
        let mut record: std::sync::RwLockWriteGuard<'_, Record> = lock.write().unwrap();
        record.index_html = make_http_response("text/html", &INDEX_HTML);
        record.echarts_js = make_http_response("application/javascript", &ECHARTS_JS);
        drop(record);
        loop {
            let mut names = Vec::new();
            let mut usages = Vec::new();
            let mut core_times = Vec::new();
            cpu.scan(cpu::CPU_STAT_FILE);
            for core in &cpu.cores {
                let name = format!("\"{}\"", core.name);
                names.push(name);
                core_times.push(core.get_time());
            }
            std::thread::sleep(Duration::from_secs(1));
            for i in 0..cpu.cores.len() {
                let delta = cpu.cores[i].get_time().delta(&core_times[i]);
                let usage = format!("{}", delta.usage());
                usages.push(usage);
            }
            let mut record: std::sync::RwLockWriteGuard<'_, Record> = lock.write().unwrap();
            record.cpu_infos = make_cpu_infos(&names, &usages, cpu::Cpu::thermal(cpu::CPU_THERMAL_FILE));
            drop(record);

            let mem = memory::Memory::read(memory::MEMINFO_FILE);
            let mut record: std::sync::RwLockWriteGuard<'_, Record> = lock.write().unwrap();
            record.memory_infos = make_memory_infos(&mem);
            drop(record);

            let mut disks = Vec::new();
            let mut has_rootfs = false;
            let disk_labels = disk::Disk::list_labels(disk::DISK_LABEL_PATH);
            for label in disk_labels {
                let mut disk = disk::Disk::new(&label);
                let device = disk::Disk::find_device(disk::DISK_LABEL_PATH, &label);
                let path = disk::Disk::find_path(disk::DISK_MOUNT_FILE, &device);
                disk.read(&path);
                disks.push(disk);
                if label == "rootfs" || path == "/" {
                    has_rootfs = true;
                }
            }

            if !has_rootfs {
                let mut disk = disk::Disk::new("rootfs");
                disk.read("/");
                disks.push(disk);
            }
            let mut record: std::sync::RwLockWriteGuard<'_, Record> = lock.write().unwrap();
            record.disk_infos = make_disk_infos(&disks);
            drop(record);
        }
    });

    let server = std::net::TcpListener::bind("0.0.0.0:1995").unwrap();
    println!("http://localhost:1995");
    for stream in server.incoming() {
        let lock = rwlock.clone();
        std::thread::spawn(move || {
            let record = lock.read().unwrap();
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(&stream);
            let mut request = String::new();
            if reader.read_line(&mut request).is_err() {
                return;
            }

            if request.contains("GET / HTTP/1.1") {
                stream.write_all(&record.index_html.as_bytes()).unwrap();
            } else if request.contains("GET /echarts.min.js HTTP/1.1") {
                stream.write_all(&record.echarts_js.as_bytes()).unwrap();
            } else if request.contains("GET /cpu HTTP/1.1") {
                stream.write_all(&record.cpu_infos.as_bytes()).unwrap();
            } else if request.contains("GET /memory HTTP/1.1") {
                stream.write_all(&record.memory_infos.as_bytes()).unwrap();
            } else if request.contains("GET /disk HTTP/1.1") {
                stream.write_all(&record.disk_infos.as_bytes()).unwrap();
            }
        });
    }
}

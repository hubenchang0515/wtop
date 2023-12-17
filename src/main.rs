use std::{io::{Write, BufRead}, io::BufReader, time::Duration};
mod cpu;
mod memory;
mod disk;

const INDEX_HTML:&str = include_str!("index.html");
const ECHARTS_JS:&str = include_str!("echarts.min.js");

struct CoreTimeRecord {
    core: cpu::CpuCore,
    last_time: cpu::CpuCoreTime,
    delta: cpu::CpuCoreTimeDelta,
}

fn make_http_response(content_type:&str, content:&str) -> String {
    format!("HTTP/1.1 200 OK\r\nContent-Type:{}\r\nContent-Length:{}\r\n\r\n{}", content_type, content.len(), content)
}

fn make_cpu_infos(cpus:&Vec<String>, usages:&Vec<String>, thermal:f32) -> String {
    let content = format!(r#"{{"labels":[{}],"usages":[{}],"thermal":{}}}"#, 
        cpus.join(","), 
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
    let cpu = cpu::Cpu::new(cpu::CPU_STAT_FILE);
    let records = Vec::<CoreTimeRecord>::new();
    let m = std::sync::Arc::new(std::sync::Mutex::new(records));
    let m2 = m.clone();
    
    std::thread::spawn(move || {
        loop {
            for core in &cpu.cores {
                let mut records = m2.lock().unwrap();
                let time = core.get_time();

                let mut exist = false;
                for record in &mut *records {
                    if record.core.name == core.name {
                        let delta = time.delta(&record.last_time);
                        record.last_time = time;
                        record.delta = delta;
                        exist = true;
                        break;
                    }
                }

                if !exist { 
                    let time = core.get_time();
                    (*records).push(CoreTimeRecord{
                        core: cpu::CpuCore::new(cpu::CPU_STAT_FILE, &core.name),
                        last_time: time,
                        delta: cpu::CpuCoreTimeDelta::new(),
                    });
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });

    let index_html_response = make_http_response("text/html", &INDEX_HTML).into_bytes();
    let echarts_js_response = make_http_response("application/javascript", &ECHARTS_JS).into_bytes();
    let server = std::net::TcpListener::bind("0.0.0.0:1995").unwrap();
    println!("http://localhost:1995");
    for stream in server.incoming() {
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(&stream);
        let mut request = String::new();
        if reader.read_line(&mut request).is_err() {
            continue;
        }

        if request.contains("GET / HTTP/1.1") {
            stream.write_all(&index_html_response).unwrap();
        } else if request.contains("GET /echarts.min.js HTTP/1.1") {
            stream.write_all(&echarts_js_response).unwrap();
        } else if request.contains("GET /cpu HTTP/1.1") {
            let records = m.lock().unwrap();
            let mut cpus = Vec::<String>::new();
            let mut usages = Vec::<String>::new();
            for record in &(*records) {
                if record.core.name == "cpu" {
                    continue;
                }
                let name = format!("\"{}\"", &record.core.name);
                let usage = format!("{}", record.delta.usage());
                cpus.push(name);
                usages.push(usage)
            }
            let response = make_cpu_infos(&cpus, &usages, cpu::Cpu::thermal(cpu::CPU_THERMAL_FILE));
            stream.write_all(response.as_bytes()).unwrap();
        } else if request.contains("GET /memory HTTP/1.1") {
            let mem = memory::Memory::read(memory::MEMINFO_FILE);
            let response = make_memory_infos(&mem);
            stream.write_all(response.as_bytes()).unwrap();
        } else if request.contains("GET /disk HTTP/1.1") {
            let labels = disk::Disk::list_labels(disk::DISK_LABEL_PATH);
            let mut disks = Vec::new();
            let mut has_rootfs = false;
            for label in labels {                
                let device = disk::Disk::find_device(disk::DISK_LABEL_PATH, &label);
                let path = disk::Disk::find_path(disk::DISK_MOUNT_FILE, &device);
                let mut disk = disk::Disk::new(&label);
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
            let response = make_disk_infos(&disks);
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
    
}

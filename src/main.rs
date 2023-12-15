use std::{io::{Write, BufRead}, io::BufReader, time::Duration};
mod cpu;
mod memory;
mod disk;

const HTML:&str = 
r#"HTTP/1.1 200 OK

<html>
    <head>
        <title>WTOP</title>
        <script src="https://cdn.jsdelivr.net/npm/echarts@5.4.3/dist/echarts.min.js"></script>
    </head>
    <body>
        <div id="main" style="width: 100%;height: 100%;"></div>
        <script type="text/javascript">
            let option = {
                title: [
                    {text: 'CPU Usage'},
                    {text: 'Memory Usage', top: '50%'},
                    {text: 'Disk Usage', top: '50%', left: '50%'}
                ],
                tooltip: {
                    trigger: 'item',
                    formatter: '{a} <br/>{b}: {c}'
                },
                grid: [
                    {
                        left: '5%',
                        right: '5%',
                        height: '40%',
                        containLabel: true
                    },
                    {
                        top: '60%',
                        left: '5%',
                        right: '55%',
                        containLabel: true
                    },
                    {
                        top: '60%',
                        left: '55%',
                        right: '5%',
                        containLabel: true
                    }
                ],
                xAxis: [
                    {
                        gridIndex: 0,
                        type: 'category',
                        data: ['cpu0', 'cpu1', 'cpu2', 'cpu3'],
                    }
                ],
                yAxis: [
                    {gridIndex: 0, type: 'value', min: 0, max: 100}
                ],
                series: [
                    {
                        name: 'CPU',
                        xAxisIndex: 0,
                        yAxisIndex: 0,
                        type: 'bar',
                        data: [100, 100, 100, 100],
                        itemStyle: {
                            "normal": {
                                color: (params) => {
                                    if (params.value > 90) {
                                        return '#e91e63'
                                    } else if (params.value > 60) {
                                        return '#ffa726'
                                    } else if (params.value > 20) {
                                        return '#03a9f4'
                                    } else {
                                        return '#00e676'
                                    }
                                }
                            }
                        }
                    },
                    {
                        name: 'Memory',
                        type: 'pie',
                        center: ['25%', '75%'],
                        radius: [0, '25%'],
                        color: ['#f50057', '#fdd835', '#ff5722', '#03a9f4'],
                        data: [
                            {name: 'Used', value: 10}, 
                            {name: 'Buffers', value: 10},
                            {name: 'Cached', value: 10},
                            {name: 'Free', value: 100},
                        ],
                        label: {
                            formatter: (params) => {
                                const unit = ['KB', 'MB', 'GB', 'TB'];
                                let v = params.value;
                                let i = 0;
                                while (v > 1024 && i + 1 < unit.length) {
                                    v = v / 1024
                                    i = i + 1
                                }
                                return `${params.name}: ${v.toFixed(2)}${unit[i]}(${params.percent}%)`;
                            },
                        }
                    },
                    {
                        name: 'Swap',
                        type: 'pie',
                        center: ['25%', '75%'],
                        radius: ['30%', '40%'],
                        color: ['#f50057', '#ff5722', '#03a9f4'],
                        data: [
                            {name: 'Used', value: 10}, 
                            {name: 'Cached', value: 10},
                            {name: 'Free', value: 100}
                        ],
                        label: {
                            formatter: (params) => {
                                const unit = ['KB', 'MB', 'GB', 'TB'];
                                let v = params.value;
                                let i = 0;
                                while (v > 1024 && i + 1 < unit.length) {
                                    v = v / 1024
                                    i = i + 1
                                }
                                return `${params.name}: ${v.toFixed(2)}${unit[i]}(${params.percent}%)`;
                            },
                        }
                    },
                    {
                        name: 'Disk',
                        type: 'pie',
                        center: ['75%', '75%'],
                        radius: ['0%', '40%'],
                        color: ['#f50057', '#03a9f4'],
                        data: [
                            {name: 'Used', value: 10}, 
                            {name: 'Free', value: 100}
                        ],
                        label: {
                            formatter: (params) => {
                                const unit = ['B', 'KB', 'MB', 'GB', 'TB'];
                                let v = params.value;
                                let i = 0;
                                while (v > 1024 && i + 1 < unit.length) {
                                    v = v / 1024
                                    i = i + 1
                                }
                                return `${params.name}: ${v.toFixed(2)}${unit[i]}(${params.percent}%)`;
                            },
                        }
                    }
                ]
            };

            setInterval(() => {
                let cpuRequest = new XMLHttpRequest();
                cpuRequest.open('GET', '/cpu', true);
                cpuRequest.onreadystatechange = () => {
                    if (cpuRequest.readyState != 4 || cpuRequest.status != 200)
                        return;
                    
                    let response = JSON.parse(cpuRequest.response);
                    option.xAxis[0].data = response.labels;
                    option.series[0].data = response.usages;
                    let chart = echarts.init(document.getElementById('main'));
                    chart.setOption(option);
                }
                cpuRequest.send();

                let memRequest = new XMLHttpRequest();
                memRequest.open('GET', '/memory', true);
                memRequest.onreadystatechange = () => {
                    if (memRequest.readyState != 4 || memRequest.status != 200)
                        return;

                    let response = JSON.parse(memRequest.response);
                    option.series[1].data[0].value = response.used;
                    option.series[1].data[1].value = response.buffers;
                    option.series[1].data[2].value = response.cached;
                    option.series[1].data[3].value = response.free;
                    option.series[2].data[0].value = response.swap_used;
                    option.series[2].data[1].value = response.swap_cached;
                    option.series[2].data[2].value = response.swap_free;
                    let chart = echarts.init(document.getElementById('main'));
                    chart.setOption(option);
                }
                memRequest.send();

                let diskRequest = new XMLHttpRequest();
                diskRequest.open('GET', '/disk', true);
                diskRequest.onreadystatechange = () => {
                    if (diskRequest.readyState != 4 || diskRequest.status != 200)
                        return;

                    let response = JSON.parse(diskRequest.response);
                    option.series[3].data[0].value = response.used;
                    option.series[3].data[1].value = response.free;
                    let chart = echarts.init(document.getElementById('main'));
                    chart.setOption(option);
                }
                diskRequest.send();
            }, 1000);
        </script>
    </body>
</html>
"#;

struct CoreTimeRecord {
    core: cpu::CpuCore,
    last_time: cpu::CpuCoreTime,
    delta: cpu::CpuCoreTimeDelta,
}

fn make_cpu_infos(cpus:&Vec<String>, usages:&Vec<String>) -> String {
    format!("HTTP/1.1 200 OK\r
Content-Type: text/json\r
\r
{{
    \"labels\":[{}], 
    \"usages\":[{}]
}}", 
    cpus.join(","), 
    usages.join(","))
}

fn make_memory_infos(mem: &memory::Memory) -> String {
    format!("HTTP/1.1 200 OK\r
Content-Type: text/json\r
\r
{{
    \"used\": {}, 
    \"buffers\": {}, 
    \"cached\": {}, 
    \"free\": {}, 
    \"swap_used\": {},
    \"swap_cached\": {},
    \"swap_free\": {}
}}",
    mem.used,
    mem.buffers,
    mem.cached,
    mem.free,
    mem.swap_used,
    mem.swap_cached,
    mem.swap_free)
}

fn make_disk_infos(disk: &disk::Disk) -> String {
    format!("HTTP/1.1 200 OK\r
Content-Type: text/json\r
\r
{{
    \"used\": {}, 
    \"free\": {}
}}",
    disk.used,
    disk.free)
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
            stream.write_all(HTML.as_bytes()).unwrap()
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
            let response = make_cpu_infos(&cpus, &usages);
            stream.write_all(response.as_bytes()).unwrap();
        } else if request.contains("GET /memory HTTP/1.1") {
            let mem = memory::Memory::read(memory::MEMINFO_FILE);
            let response = make_memory_infos(&mem);
            stream.write_all(response.as_bytes()).unwrap();
        } else if request.contains("GET /disk HTTP/1.1") {
            let disk = disk::Disk::read(disk::DISK_ROOT_FILE);
            let response = make_disk_infos(&disk);
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
    
}

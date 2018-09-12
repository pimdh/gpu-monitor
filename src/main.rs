extern crate regex;
extern crate csv;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate prettytable;

use std::str;
use std::process::Command;
use std::io::Cursor;
use csv::ReaderBuilder;
use regex::Regex;
use std::thread;
use std::env;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;


#[derive(Debug, Deserialize)]
struct RawGpuRecord {
    index: usize,
    name: String,
    total_memory: String,
    used_memory: String,
    utilization: String,
}

#[derive(Debug)]
struct GpuRecord {
    index: usize,
    name: String,
    total_memory: f64,
    memory_usage: f64,
    utilization: f64
}

#[derive(Debug)]
struct HostRecord {
    hostname: String,
    gpu_records: Vec<GpuRecord>
}

fn string_to_f64(s: &String) -> f64 {
    lazy_static! {
        static ref RE: Regex = Regex::new("[^0-9.]+").unwrap();
    }
    RE.replace_all(s, "").parse::<f64>().unwrap()
}


fn parse_record(r: RawGpuRecord) -> GpuRecord {
    let mem_total = string_to_f64(&r.total_memory);
    let mem_used = string_to_f64(&r.used_memory);
    let util = string_to_f64(&r.utilization);
    GpuRecord {
        index: r.index,
        name: r.name[1..].to_string(),
        total_memory: mem_total,
        memory_usage: mem_used / mem_total,
        utilization: util / 100.
    }
}

fn gpu_record_row(r: &GpuRecord) -> Row {
    Row::new(vec![
        Cell::new(&format!("{}", r.index)),
        Cell::new(&r.name),
        Cell::new(&format!("{:.0}", r.total_memory)),
        Cell::new(&format!("{:.0}", r.memory_usage * 100.0)),
        Cell::new(&format!("{:.0}", r.utilization * 100.0)),
    ])
}

fn gpu_record_table(rs: &Vec<GpuRecord>) -> Table {
    let mut table = Table::new();
    table.add_row(row!["Index", "Name", "Total mem (MB)", "Mem usage (%)", "Utilization (%)"]);
    for r in rs {
        table.add_row(gpu_record_row(r));
    }
    table
}

fn host_record_row(record: &HostRecord) -> Row {
    row![record.hostname, gpu_record_table(&record.gpu_records)]
}

fn host_records_table(records: &Vec<HostRecord>) -> Table {
    let mut table = Table::new();
    table.add_row(row!["Hostname", "GPUs"]);
    for record in records {
        table.add_row(host_record_row(record));
    }
    table
}


fn fetch(hostname: &String) -> Option<Vec<u8>>  {
    let out = Command::new("ssh")
            .arg(hostname)
            .arg("nvidia-smi")
            .arg("--query-gpu=index,gpu_name,memory.total,memory.used,utilization.gpu")
            .arg("--format=csv")
            .output()
            .expect("failed to execute process");
    match out.status.success() {
        true  => Some(out.stdout),
        false => None
    }
}

fn parse_csv(data: Vec<u8>) -> Result<Vec<GpuRecord>, csv::Error> {
    let rdr = Cursor::new(data);
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(rdr);
    rdr.deserialize().skip(1).map(|r| r.map(parse_record)).collect()
}

fn fetch_host(hostname: &String) -> HostRecord {
    let ssh_output = fetch(&hostname).unwrap();
    HostRecord {
        hostname: hostname.clone(),
        gpu_records: parse_csv(ssh_output).unwrap(),
    }
}

fn fetch_hosts(hostnames: Vec<String>) -> Vec<HostRecord> {
    let handles = hostnames.into_iter().map(|h| {
        thread::spawn(move || {
            fetch_host(&h)
        })
    });
    handles.map(|h| h.join().unwrap()).collect()
}


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let records = fetch_hosts(args);
    host_records_table(&records).printstd();
}

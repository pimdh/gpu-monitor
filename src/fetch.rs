extern crate regex;
extern crate csv;

use std::thread;
use std::process::Command;
use std::io::Cursor;
use csv::ReaderBuilder;
use regex::Regex;

#[derive(Debug, Deserialize)]
struct RawGpuRecord {
    index: usize,
    name: String,
    total_memory: String,
    used_memory: String,
    utilization: String,
}

#[derive(Debug)]
pub struct GpuRecord {
    pub index: usize,
    pub name: String,
    pub total_memory: f64,
    pub used_memory: f64,
    pub utilization: f64
}

#[derive(Debug)]
pub struct HostRecord {
    pub hostname: String,
    pub gpu_records: Vec<GpuRecord>
}

fn parse_record(r: RawGpuRecord) -> GpuRecord {
    let total_memory = string_to_f64(&r.total_memory);
    let used_memory = string_to_f64(&r.used_memory);
    let util = string_to_f64(&r.utilization);
    GpuRecord {
        index: r.index,
        name: r.name[1..].to_string(),
        total_memory,
        used_memory,
        utilization: util / 100.
    }
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

pub fn fetch_hosts(hostnames: Vec<String>) -> Vec<HostRecord> {
    let handles = hostnames.into_iter().map(|h| {
        thread::spawn(move || {
            fetch_host(&h)
        })
    });
    handles.map(|h| h.join().unwrap()).collect()
}

fn string_to_f64(s: &String) -> f64 {
    lazy_static! {
        static ref RE: Regex = Regex::new("[^0-9.]+").unwrap();
    }
    RE.replace_all(s, "").parse::<f64>().unwrap()
}

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
    pub gpu_records: Vec<GpuRecord>
}

#[derive(Debug)]
pub enum HostError {
    Ssh(String),
    Csv(csv::Error),
}

#[derive(Debug)]
pub struct HostResult {
    pub hostname: String,
    pub result: Result<HostRecord, HostError>
}

use std::fmt;
use std::error;

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HostError::Ssh(ref s) => write!(f, "SSH error: {}", s),
            HostError::Csv(ref e) => e.fmt(f),
        }
    }
}
impl error::Error for HostError {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HostError::Ssh(_) => None,
            HostError::Csv(ref e) => Some(e),
        }
    }
}
impl From<csv::Error> for HostError {
    fn from(err: csv::Error) -> HostError {
        HostError::Csv(err)
    }
}


fn parse_record(r: &RawGpuRecord) -> GpuRecord {
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

fn fetch(hostname: &str) -> Result<Vec<u8>, String>  {
    let out = Command::new("ssh")
            .arg(hostname)
            .arg("nvidia-smi")
            .arg("--query-gpu=index,gpu_name,memory.total,memory.used,utilization.gpu")
            .arg("--format=csv")
            .output()
            .expect("failed to execute process");
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8(out.stderr).unwrap_or_else(|_|"parse error".to_string()))
    }
}

fn parse_csv(data: Vec<u8>) -> Result<HostRecord, csv::Error> {
    let rdr = Cursor::new(data);
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(rdr);
    let gpu_records = rdr.deserialize().skip(1).map(|r| r.map(|x| parse_record(&x))).collect::<Result<_, _>>()?;
    Ok(HostRecord { gpu_records })
}

fn fetch_host(hostname: &str) -> Result<HostRecord, HostError> {
    let ssh_output = fetch(&hostname).map_err(HostError::Ssh)?;
    Ok(parse_csv(ssh_output)?)
}

pub fn fetch_hosts(hostnames: Vec<String>) -> Vec<HostResult> {
    let handles = hostnames.into_iter().map(|h| {
        thread::spawn(move || {
            let result = fetch_host(&h);
            HostResult {
                hostname: h.clone(),
                result,
            }
        })
    });
    handles.map(|h| h.join().unwrap()).collect()
}

fn string_to_f64(s: &str) -> f64 {
    lazy_static! {
        static ref RE: Regex = Regex::new("[^0-9.]+").unwrap();
    }
    RE.replace_all(s, "").parse::<f64>().unwrap()
}

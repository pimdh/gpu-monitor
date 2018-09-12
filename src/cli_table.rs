use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

use fetch::{GpuRecord, HostResult};

fn gpu_record_row(r: &GpuRecord) -> Row {
    Row::new(vec![
        Cell::new(&format!("{}", r.index)),
        Cell::new(&r.name),
        Cell::new(&format!("{:.2}", r.total_memory / 1000.0)),
        Cell::new(&format!("{:.2}", r.used_memory / 1000.0)),
        Cell::new(&format!("{:.2}", r.utilization * 100.0)),
    ])
}

fn gpu_record_table(rs: &Vec<GpuRecord>) -> Table {
    let mut table = Table::new();
    table.add_row(row!["Index", "Name", "Total mem (GB)", "Used mem (GB)", "Util (%)"]);
    for r in rs {
        table.add_row(gpu_record_row(r));
    }
    table
}

fn host_record_row(result: &HostResult) -> Row {
    match result.result {
        Ok(ref record) => row![result.hostname, gpu_record_table(&record.gpu_records)],
        Err(ref err) => row![result.hostname, format!("{}", err)]
    }
}

pub fn host_records_table(records: &Vec<HostResult>) -> Table {
    let mut table = Table::new();
    table.add_row(row!["Hostname", "GPUs"]);
    for record in records {
        table.add_row(host_record_row(record));
    }
    table
}

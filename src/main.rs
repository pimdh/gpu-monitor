extern crate regex;
extern crate csv;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate prettytable;

mod cli_table;
mod fetch;

use fetch::fetch_hosts;
use cli_table::host_records_table;

use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let records = fetch_hosts(args);
    host_records_table(&records).printstd();
}

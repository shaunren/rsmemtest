extern crate getopts;
extern crate pbr;
extern crate num_cpus;
extern crate crossbeam;
extern crate rsmemtest;

use getopts::Options;
use std::env;


use std::mem::size_of;
use pbr::ProgressBar;

use rsmemtest::{TestBlock, TesterMessage, test_thread};



fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_memory_bytes(mem_opt_str: &String) -> usize {
    let num_bytes = mem_opt_str.parse::<usize>().unwrap() * 1024 * 1024;
    num_bytes
}

fn format_error_message(num_errors: u64) -> String {
    format!("Errors: {:06}    ", num_errors)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("m", "", "mem", "amount of memory to test in MiB");
    opts.optopt("t", "", "threads", "number of threads");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") || !matches.opt_present("m") {
        print_usage(&program, opts);
        return;
    }
    let bytes_to_test = parse_memory_bytes(&matches.opt_str("m").unwrap());

    let num_threads = match matches.opt_str("t") {
        Some(n) => n.parse().unwrap(), // FIXME check error
        None    => num_cpus::get(),
    };

    let num_blocks = bytes_to_test / size_of::<TestBlock>();
    let mut test_buffer: Vec<TestBlock> = Vec::with_capacity(num_blocks);
    unsafe { test_buffer.set_len(num_blocks); }

    let mut pb = ProgressBar::new((bytes_to_test / (1024 * 1024)) as u64);
    let mut num_bytes_covered = 0;
    let mut num_errors = 0;

    pb.show_speed = false;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.message(&format_error_message(0));

    let (tx, rx) = crossbeam::unbounded();

    crossbeam::scope(|scope| {
        for blocks in test_buffer.chunks_mut(num_blocks / num_threads) {
            let tx = tx.clone();
            scope.spawn(move |_| test_thread(blocks, tx));
        }

        for msg in rx {
            match msg {
                TesterMessage::CoveredBytes(bytes) => {
                    num_bytes_covered += bytes;
                    pb.set((((num_bytes_covered * 3 / 40) % bytes_to_test)/ (1024 * 1024)) as u64);
                },
                TesterMessage::FoundError => {
                    num_errors += 1;
                    pb.message(&format_error_message(num_errors));
                }
            }
        }

        pb.finish();
    }).unwrap();
}


use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

mod diff;
mod range;

use diff::{diff, Difference::*};

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    my_file: PathBuf,
    #[structopt(parse(from_os_str))]
    original_file: PathBuf,
    #[structopt(parse(from_os_str))]
    your_file: PathBuf,
}

fn read(path: &Path) -> String {
    let mut buf = String::new();
    File::open(path).unwrap().read_to_string(&mut buf).unwrap();
    buf
}

fn main() {
    let args: Cli = Cli::from_args();

    let diffs = diff(
        &read(&args.original_file),
        &read(&args.my_file),
        &read(&args.your_file),
    );
    for diff in diffs {
        match diff {
            NotChanged(s) | Add(_, s) | Modify(_, _, s) => {
                print!("{}", s);
            }
            Remove(_, _) => {}
            Conflict(o, a, b) => {
                println!("<<<<<<< {}", args.my_file.to_string_lossy());
                print!("{}", a);
                println!("||||||| {}", args.original_file.to_string_lossy());
                print!("{}", o);
                println!("=======");
                print!("{}", b);
                println!(">>>>>>> {}", args.your_file.to_string_lossy());
            }
        }
    }
}

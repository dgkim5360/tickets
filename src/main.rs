extern crate structopt;

use structopt::StructOpt;

use tickets::{self, opt};

fn main() {
    let opt = opt::Opt::from_args();
    // println!("{:?}", opt);

    let (exit_code, sys_message) = tickets::match_action(opt);
    tickets::die(exit_code, sys_message);
}

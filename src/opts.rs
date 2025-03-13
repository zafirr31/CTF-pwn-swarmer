use structopt::StructOpt;
use derive_setters::Setters;

#[derive(StructOpt, Setters, Clone, Debug)]
pub struct Args {
    #[structopt(short, long)]
    #[structopt(default_value = "5")]
    pub num_processes: u8,

    #[structopt(short, long)]
    pub flag_format: String,

    #[structopt(short, long)]
    #[structopt(default_value = "9223372036854775808")]
    pub timeout: u64,

    pub script: String,

    #[structopt(short, long)]
    pub verbose: bool,
}

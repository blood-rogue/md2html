use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Command {
    #[arg(long, short)]
    pub file_path: PathBuf,

    #[arg(long, short, default_value_t = String::from("out"))]
    pub out_dir: String,

    #[arg(long, short)]
    pub domain_name: String,
}

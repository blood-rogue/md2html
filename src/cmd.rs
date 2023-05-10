use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Command {
    #[arg(long, short)]
    pub file_path: PathBuf,

    #[arg(long, short, default_value_t = String::from("out"))]
    pub out_dir: String,

    #[arg(long, short, default_value_t = String::from("localhost"))]
    pub domain_name: String,

    #[arg(long, short = 'O')]
    pub output_ast: bool,
}

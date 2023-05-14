use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Command {
    #[arg(long, short)]
    pub file_path: PathBuf,

    #[arg(long, short, default_value_t = String::from("out"))]
    pub out_dir: String,

    #[arg(long, short, default_value_t = String::from("localhost"))]
    pub domain_name: String,

    #[arg(long, short = 'O')]
    pub output_ast: bool,

    #[arg(long, short = 'v')]
    pub verbose: bool,

    #[arg(long, short, default_value_t = String::from("./styles.css"))]
    pub style_sheet: String,

    #[arg(long, short, default_value_t = String::from("./logo.jpg"))]
    pub logo: String,

    #[arg(short = 'F', long)]
    pub force: bool,
}

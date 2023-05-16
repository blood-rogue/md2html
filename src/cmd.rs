use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Command {
    /// The path to the markdown file
    #[arg(long, short)]
    pub file_path: String,

    /// The output directory in which to place files (generated html, logo and styles)
    #[arg(long, short, default_value_t = String::from("out"))]
    pub out_dir: String,

    /// The domain name of the blog to identify external websites
    #[arg(long, short, default_value_t = String::from("localhost"))]
    pub domain_name: String,

    /// Output the HTML and Markdown struct debug info
    #[arg(long, short = 'O')]
    pub output_ast: bool,

    /// Log events
    #[arg(long, short = 'v')]
    pub verbose: bool,

    #[arg(long, short, default_value_t = String::from("./authors.toml"))]
    pub authors_db: String,

    /// Force overwrite file to the output directory
    #[arg(short = 'F', long)]
    pub force: bool,
}

use anyhow::Result;
use clap::Parser;
use directories::BaseDirs;

use timekeep::cli::{Cli, Commands};
use timekeep::{cli, DataFiles};

fn main() -> Result<()> {
    let base_dirs = BaseDirs::new().unwrap();
    let files = DataFiles::new(base_dirs.data_dir())?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Start {
            project_name,
            description,
            overwrite,
        } => cli::start(&files, project_name, description, overwrite)?,
        Commands::End => cli::end(&files)?,
        Commands::Add => println!("Not yet implemented"),
        Commands::View => cli::view(&files)?,
    };

    Ok(())
}

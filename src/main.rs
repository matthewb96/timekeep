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
            start_time,
            description,
            overwrite,
        } => cli::start(&files, project_name, start_time, description, overwrite)?,
        Commands::End { end_time, discard } => cli::end(&files, end_time, discard)?,
        Commands::Add {
            project_name,
            start_time,
            end_time,
            description,
        } => cli::add(&files, project_name, start_time, end_time, description)?,
        Commands::View { filter, from, to } => cli::view(&files, *filter, from, to)?,
    };

    Ok(())
}

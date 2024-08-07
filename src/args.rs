use clap_derive::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Make any Linux distribution repeatable!
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a Version command
    Version {
        #[command(subcommand)]
        command: VersionCommands,
    },
    /// Build the current Version
    Build,
    /// Commands related to live medium
    Installation {
        #[command(subcommand)]
        command: InstallationCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum InstallationCommands {
    /// Part 1
    Part1 {
        #[command(subcommand)]
        command: InstallOrUpdate,
        #[arg(short, long)]
        setup: bool,
        #[arg(short, long)]
        partitioning: bool,
    },
    /// Part 2
    Part2 {
        #[arg(short, long)]
        setup: bool,
        #[arg(long)]
        system: bool,
        #[arg(long)]
        shell: bool,
        #[arg(short, long)]
        user: bool,
        #[arg(long)]
        services: bool,
        #[arg(short, long)]
        packages: bool,
        #[arg(short, long)]
        grub: bool,
        #[arg(short, long)]
        mkinitcpio: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum InstallOrUpdate {
    /// Wipe entire system and partition
    Install,
    /// Leave important partitions, wipe everything else
    Update,
}

#[derive(Subcommand, Debug)]
pub enum ChrootCommands {
    /// Install important packages
    InstallImportant,
    /// Install Grub
    InstallGrub,
}

#[derive(Subcommand, Debug)]
pub enum VersionCommands {
    /// List all system Versions
    List,
    /// The difference between 2 Versions
    Diff {
        /// Version to act as base
        old: usize,
        /// Version to act as changes
        new: usize,
    },
    /// Command related to the 'current' Version
    Current {
        #[command(subcommand)]
        command: CurrentCommands,
    },
    /// Align the indexes
    Align,

    /// Delete Commands
    Delete {
        #[command(subcommand)]
        command: DeleteCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CurrentCommands {
    /// Build the 'current' Version (You can always roll back later)
    Build,
    /// Commit the current Version
    Commit,
    /// Rollback to a previous Version (You still need to build after rolling back)
    Rollback(Rollback),
    /// Set the 'current' Version to the latest Version
    ToLatest,
    /// diff to config.toml
    Diff {
        /// Version to act as base
        other: usize,
    },
}

#[derive(Parser, Debug)]
pub struct SetCurrent {
    /// Version to jump to
    pub to: usize,
}

#[derive(Parser, Debug)]
pub struct Rollback {
    /// Versions to rollback to
    pub index: usize,
}

#[derive(Subcommand, Debug)]
pub enum DeleteCommands {
    Last(LastDel),
    Version(VersionDel),
}

#[derive(Parser, Debug)]
pub struct LastDel {
    /// Number of versions to be deleted
    pub number: usize,
}

#[derive(Parser, Debug)]
pub struct VersionDel {
    /// Version to delete
    pub index: usize,
}

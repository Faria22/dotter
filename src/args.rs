use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// A small dotfile manager.
#[derive(Debug, Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    /// Location of the .dotter directory. If specified, all other paths will be relative to this directory unless overridden
    #[clap(long, value_parser, global = true)]
    pub dotter_dir: Option<PathBuf>,

    /// Location of the global configuration
    #[clap(
        short,
        long,
        value_parser,
        default_value = ".dotter/global.toml",
        global = true
    )]
    pub global_config: PathBuf,

    /// Location of the local configuration
    #[clap(
        short,
        long,
        value_parser,
        default_value = ".dotter/local.toml",
        global = true
    )]
    pub local_config: PathBuf,

    /// Location of cache file
    #[clap(long, value_parser, default_value = ".dotter/cache.toml")]
    pub cache_file: PathBuf,

    /// Directory to cache into.
    #[clap(long, value_parser, default_value = ".dotter/cache")]
    pub cache_directory: PathBuf,

    /// Location of optional pre-deploy hook
    #[clap(long, value_parser, default_value = ".dotter/pre_deploy.sh")]
    pub pre_deploy: PathBuf,

    /// Location of optional post-deploy hook
    #[clap(long, value_parser, default_value = ".dotter/post_deploy.sh")]
    pub post_deploy: PathBuf,

    /// Location of optional pre-undeploy hook
    #[clap(long, value_parser, default_value = ".dotter/pre_undeploy.sh")]
    pub pre_undeploy: PathBuf,

    /// Location of optional post-undeploy hook
    #[clap(long, value_parser, default_value = ".dotter/post_undeploy.sh")]
    pub post_undeploy: PathBuf,

    /// Dry run - don't do anything, only print information.
    /// Implies -v at least once
    #[clap(short = 'd', long = "dry-run", global = true)]
    pub dry_run: bool,

    /// Verbosity level - specify up to 3 times to get more detailed output.
    /// Specifying at least once prints the differences between what was before and after Dotter's run
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbosity: u8,

    /// Quiet - only print errors
    #[clap(short, long, value_parser, global = true)]
    pub quiet: bool,

    /// Force - instead of skipping, overwrite target files if their content is unexpected.
    /// Overrides --dry-run.
    #[clap(short, long, value_parser, global = true)]
    pub force: bool,

    /// Assume "yes" instead of prompting when removing empty directories
    #[clap(short = 'y', long = "noconfirm", global = true)]
    pub noconfirm: bool,

    /// Take standard input as an additional files/variables patch, added after evaluating
    /// `local.toml`. Assumes --noconfirm flag because all of stdin is taken as the patch.
    #[clap(short, long, value_parser, global = true)]
    pub patch: bool,

    /// Amount of lines that are printed before and after a diff hunk.
    #[clap(long, value_parser, default_value = "3")]
    pub diff_context_lines: usize,

    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Debug, Clone, Subcommand, Default)]
pub enum Action {
    /// Deploy the files to their respective targets. This is the default subcommand.
    #[default]
    Deploy,

    /// Delete all deployed files from their target locations.
    /// Note that this operates on all files that are currently in cache.
    Undeploy,

    /// Initialize global.toml with a single package containing all the files in the current
    /// directory pointing to a dummy value and a local.toml that selects that package.
    Init,

    /// Run continuously, watching the repository for changes and deploying as soon as they
    /// happen. Can be ran with `--dry-run`
    #[cfg(feature = "watch")]
    Watch,

    /// Generate shell completions
    GenCompletions {
        /// Set the shell for generating completions [values: bash, elvish, fish, powerShell, zsh]
        #[clap(long, short)]
        shell: Shell,

        /// Set the out directory for writing completions file
        #[clap(long)]
        to: Option<PathBuf>,
    },
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dotter_dir: None,
            global_config: PathBuf::from(".dotter/global.toml"),
            local_config: PathBuf::from(".dotter/local.toml"),
            cache_file: PathBuf::from(".dotter/cache.toml"),
            cache_directory: PathBuf::from(".dotter/cache"),
            pre_deploy: PathBuf::from(".dotter/pre_deploy.sh"),
            post_deploy: PathBuf::from(".dotter/post_deploy.sh"),
            pre_undeploy: PathBuf::from(".dotter/pre_undeploy.sh"),
            post_undeploy: PathBuf::from(".dotter/post_undeploy.sh"),
            dry_run: false,
            verbosity: 0,
            quiet: false,
            force: false,
            noconfirm: false,
            patch: false,
            diff_context_lines: 3,
            action: None,
        }
    }
}

pub fn get_options() -> Options {
    // We need to detect which flags were actually provided by the user
    // We'll check if dotter_dir was provided and if individual paths were NOT provided
    let args: Vec<String> = std::env::args().collect();
    let has_dotter_dir = args.iter().any(|arg| arg == "--dotter-dir");
    let has_global_config = args.iter().any(|arg| arg == "-g" || arg == "--global-config");
    let has_local_config = args.iter().any(|arg| arg == "-l" || arg == "--local-config");
    let has_cache_file = args.iter().any(|arg| arg == "--cache-file");
    let has_cache_directory = args.iter().any(|arg| arg == "--cache-directory");
    let has_pre_deploy = args.iter().any(|arg| arg == "--pre-deploy");
    let has_post_deploy = args.iter().any(|arg| arg == "--post-deploy");
    let has_pre_undeploy = args.iter().any(|arg| arg == "--pre-undeploy");
    let has_post_undeploy = args.iter().any(|arg| arg == "--post-undeploy");
    
    let mut opt = Options::parse();
    if opt.dry_run {
        opt.verbosity = std::cmp::max(opt.verbosity, 1);
    }
    opt.verbosity = std::cmp::min(3, opt.verbosity);
    if opt.patch {
        opt.noconfirm = true;
    }
    
    // Apply dotter_dir to all paths that weren't explicitly set
    if has_dotter_dir {
        if let Some(ref dotter_dir) = opt.dotter_dir {
            if !has_global_config {
                opt.global_config = dotter_dir.join("global.toml");
            }
            if !has_local_config {
                opt.local_config = dotter_dir.join("local.toml");
            }
            if !has_cache_file {
                opt.cache_file = dotter_dir.join("cache.toml");
            }
            if !has_cache_directory {
                opt.cache_directory = dotter_dir.join("cache");
            }
            if !has_pre_deploy {
                opt.pre_deploy = dotter_dir.join("pre_deploy.sh");
            }
            if !has_post_deploy {
                opt.post_deploy = dotter_dir.join("post_deploy.sh");
            }
            if !has_pre_undeploy {
                opt.pre_undeploy = dotter_dir.join("pre_undeploy.sh");
            }
            if !has_post_undeploy {
                opt.post_undeploy = dotter_dir.join("post_undeploy.sh");
            }
        }
    }
    
    opt
}

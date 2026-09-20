use clap::{Args, Parser, Subcommand};
use nusave::{
    EditError, ReadError, Save, WriteError, apply_assignments, properties, raw_list_text,
    save_file_name, summary_text, write_atomic,
};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror::Error;

const AFTER_HELP: &str = "\
Commands target slot 0 inside SAVEGAME_FOLDER unless --slot selects another slot.
Slots 0..2 contain game progress; slot 3 contains the standalone SuperOptions record.
The default folder is res/SavedGames.
List shows a compact interpreted summary; list --raw emits reusable assignments.

Integers accept decimal or 0x hex; floats accept decimal; byte arrays use hex:0011aaff.
Flags accept the names documented in the README joined with |; byte arrays also accept text:NAME.
Checksum and slot_code are refreshed automatically; --keep-derived preserves them.
Create refuses existing output. Edit writes atomically; --output refuses existing output.";

#[derive(Parser, Debug)]
#[command(
    name = "nusave",
    version,
    about = "Inspect, explain, edit, and create Nu engine savegames",
    after_help = AFTER_HELP
)]
struct Cli {
    /// Folder containing LEGO Star Wars: The Complete Saga savegames.
    #[arg(value_name = "SAVEGAME_FOLDER", default_value = "res/SavedGames")]
    folder: PathBuf,

    /// Save slot to read, edit, or create (game: 0..2; options: 3).
    #[arg(long, global = true, value_parser = clap::value_parser!(u8).range(0..=3))]
    slot: Option<u8>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show a compact, interpreted view of the selected save.
    List(ListArgs),
    /// Edit the selected slot atomically, or write an edited copy with --output.
    Edit(EditArgs),
    /// Create the selected slot, optionally using another save as its template.
    Create(CreateArgs),
}

#[derive(Args, Debug, Default)]
struct ListArgs {
    /// Show only interpreted rows containing this text.
    #[arg(long, value_name = "TEXT")]
    filter: Option<String>,

    /// Emit every stored property as reusable key=value assignments.
    #[arg(long)]
    raw: bool,
}

#[derive(Args, Debug)]
struct EditArgs {
    /// Write an edited copy instead of replacing the selected slot.
    #[arg(long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Read additional key=value assignments from a file.
    #[arg(long, value_name = "FILE")]
    params: Vec<PathBuf>,

    /// Do not refresh checksum and slot_code.
    #[arg(long)]
    keep_derived: bool,

    /// Property assignments applied after all --params files.
    #[arg(value_name = "KEY=VALUE", num_args = 0..)]
    assignments: Vec<String>,
}

#[derive(Args, Debug)]
struct CreateArgs {
    /// Copy an existing save before applying assignments.
    #[arg(long, value_name = "SAVE", conflicts_with = "options")]
    from: Option<PathBuf>,

    /// Create a zero-initialized standalone SuperOptions save.
    #[arg(long, conflicts_with = "from")]
    options: bool,

    /// Read additional key=value assignments from a file.
    #[arg(long, value_name = "FILE")]
    params: Vec<PathBuf>,

    /// Do not refresh checksum and slot_code.
    #[arg(long)]
    keep_derived: bool,

    /// Property assignments applied after all --params files.
    #[arg(value_name = "KEY=VALUE", num_args = 0..)]
    assignments: Vec<String>,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("--options must use slot 3")]
    OptionsSlot,
    #[error("cannot read parameter file {}: {source}", path.display())]
    ReadParameters {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("parameter file {} contains non-UTF-8 data", path.display())]
    NonUtf8Parameters { path: PathBuf },
    #[error("cannot create savegame folder {}: {source}", path.display())]
    CreateFolder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    ReadSave(#[from] ReadError),
    #[error(transparent)]
    Edit(#[from] EditError),
    #[error(transparent)]
    WriteSave(#[from] WriteError),
}

fn read_assignments(path: &Path, assignments: &mut Vec<String>) -> Result<(), CliError> {
    let contents = fs::read(path).map_err(|source| CliError::ReadParameters {
        path: path.to_owned(),
        source,
    })?;
    for raw_line in contents.split(|byte| *byte == b'\n') {
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
        if line.is_empty() || line[0] == b'#' {
            continue;
        }
        assignments.push(String::from_utf8(line.to_vec()).map_err(|_| {
            CliError::NonUtf8Parameters {
                path: path.to_owned(),
            }
        })?);
    }
    Ok(())
}

fn collect_assignments(files: Vec<PathBuf>, direct: Vec<String>) -> Result<Vec<String>, CliError> {
    let mut assignments = Vec::new();
    for file in files {
        read_assignments(&file, &mut assignments)?;
    }
    assignments.extend(direct);
    Ok(assignments)
}

fn finish_edit(
    replace_input: bool,
    output: &Path,
    mut save: Save,
    assignments: Vec<String>,
    keep_derived: bool,
) -> Result<(), CliError> {
    apply_assignments(&mut save, assignments)?;
    if !keep_derived {
        save.derive();
    }
    write_atomic(output, &save, replace_input)?;
    eprintln!(
        "nusave: wrote {} bytes to {}",
        save.bytes.len(),
        output.display()
    );
    Ok(())
}

fn run(cli: Cli) -> Result<(), CliError> {
    let command = cli.command.unwrap_or(Command::List(ListArgs::default()));
    let slot_number = match &command {
        Command::Create(args) if args.options => {
            if cli.slot.is_some_and(|slot| slot != 3) {
                return Err(CliError::OptionsSlot);
            }
            3
        }
        _ => cli.slot.unwrap_or(0),
    };
    let slot = cli.folder.join(save_file_name(slot_number));
    match command {
        Command::List(args) => {
            let save = Save::read(&slot)?;
            let fields = properties(&save);
            let filter = args.filter.as_deref().unwrap_or("");
            if args.raw {
                println!(
                    "{}",
                    raw_list_text(&save, &fields, filter).trim_end_matches('\n')
                );
            } else {
                let color = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
                println!(
                    "{}",
                    summary_text(&save, &fields, filter, color).trim_end_matches('\n')
                );
            }
            Ok(())
        }
        Command::Edit(args) => {
            let assignments = collect_assignments(args.params, args.assignments)?;
            let output = args.output.unwrap_or_else(|| slot.clone());
            let save = Save::read(&slot)?;
            let replace_input = output == slot;
            finish_edit(replace_input, &output, save, assignments, args.keep_derived)
        }
        Command::Create(args) => {
            let assignments = collect_assignments(args.params, args.assignments)?;
            let save = if let Some(from) = args.from {
                Save::read(&from)?
            } else {
                Save::fresh(args.options)
            };
            fs::create_dir_all(&cli.folder).map_err(|source| CliError::CreateFolder {
                path: cli.folder.clone(),
                source,
            })?;
            finish_edit(false, &slot, save, assignments, args.keep_derived)
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nusave: {error}");
            ExitCode::FAILURE
        }
    }
}

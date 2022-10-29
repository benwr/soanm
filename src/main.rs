use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

use clap::{Parser, Subcommand};
use magic_wormhole::{transfer::APP_CONFIG, Code, Wormhole, WormholeError};
use tempfile::TempDir;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initiate an enrollment
    Sponsor {
        /// The location of the configuration files
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// The first stage to run. This is useful in development when a stage fails and you need
        /// to modify it.
        #[arg(short, long)]
        starting_stage: Option<usize>,
    },
    /// Enroll this device, receiving configuration from a remote location
    Enroll { wormhole_code: String },
}

#[derive(Debug)]
enum Error {
    Wormhole(WormholeError),
    IO(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<WormholeError> for Error {
    fn from(e: WormholeError) -> Self {
        Self::Wormhole(e)
    }
}

async fn enroll(mut hole: Wormhole, tmpdir: &std::path::Path) -> Result<(), Error> {

    let tarball_bytes = hole.receive().await?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball_bytes.as_slice()));
    archive.unpack(tmpdir)?;
    let mut entries = std::fs::read_dir(tmpdir)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let metadata = e.metadata()?;
        if metadata.is_file() {
            let stdin = hole.receive().await?;
            let mut child = std::process::Command::new(e.path())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
            let child_stdin = child.stdin.as_mut().expect("Can't set child process stdin");
            child_stdin.write_all(stdin.as_slice())?;
            drop(child_stdin);

            let output = child.wait_with_output()?;
            hole.send(output.stdout).await?;
        } else {
            eprintln!(
                "Not running {:?}; This is likely to cause input mismatches, since there\n\
                      must be exactly one initiation program per enrollee program",
                e.path()
            );
        }
    }

    Ok(())
}

async fn sponsor(mut hole: Wormhole, starting_stage: usize) -> Result<(), Error> {
    // 1. Tar and feather the enrollee programs
    let mut tarball = vec![];
    let enc = flate2::write::GzEncoder::new(&mut tarball, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", "enroll")?;
    drop(tar);

    // 2. Send them over
    // tarball.read_to_end(&mut contents)?;
    hole.send(tarball).await?;

    // 3. For each initiator program,
    let mut entries = std::fs::read_dir("sponsor")?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());
    for e in entries[starting_stage..].into_iter() {
        let metadata = e.metadata()?;
        if metadata.is_file() {
            //   a. Execute it
            let child = std::process::Command::new(e.path())
                .stdout(Stdio::piped())
                .spawn()?;
            let output = child.wait_with_output()?;
            //   b. Send its output to the remote
            hole.send(output.stdout).await?;
        }

        //   c. Receive any output from the remote
        let output = hole.receive().await?;

        //   d. Save that output
        let mut outfile = std::fs::File::create(PathBuf::from("results").join(e.path()))?;
        outfile.write(&output)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    match &args.command {
        Command::Enroll { wormhole_code } => {
            let dir = TempDir::new()?;
            std::env::set_current_dir(dir.path())?;
            let (welcome, hole) =
                Wormhole::connect_with_code(APP_CONFIG, Code(wormhole_code.to_string())).await?;
            eprintln!("{}", welcome.welcome.unwrap_or(String::new()));
            enroll(hole, dir.path()).await?;
        }
        Command::Sponsor {
            path,
            starting_stage,
        } => {
            let p: PathBuf = path.as_ref().unwrap_or(&PathBuf::from(".")).to_path_buf();
            std::env::set_current_dir(&p)?;
            let (welcome, holefuture) = Wormhole::connect_without_code(APP_CONFIG, 10).await?;
            let hole = holefuture.await?;
            eprintln!("{}", welcome.welcome.unwrap_or(String::new()));
            eprintln!("On the enrollee, run:\n\n");
            eprintln!("curl -fsSL https://github.com/benwr/soanm/releases/[determine host type] > soanm && ./soanm {}", welcome.code);
            sponsor(hole, starting_stage.unwrap_or(0)).await?;
        }
    }
    Ok(())
}

#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use owo_colors::OwoColorize;
use std::{
    env,
    fs::{self},
    io::{self, ErrorKind},
    path::Path,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Checks if a directory or file is empty, or if a symbolic link points to a path that does not exist. Only supports UTF-8 paths.
#[derive(Parser)]
#[command(author, version, about)]
struct EmpdArgs {
    /// Delete the file or directory if it is empty
    #[arg(short, long)]
    delete_if_empty: bool,
    /// Path to test
    #[arg(index = 1_usize)]
    path: String,
}

const CHECK_MARK: &str = "✔️";
const X: &str = "🗙";

fn main() -> Result<(), i32> {
    // TODO
    env::set_var("RUST_BACKTRACE", "1");
    // TODO
    env::set_var("RUST_LOG", "debug");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let result = start();

    match result {
        Ok(re) => re,
        Err(er) => {
            tracing::error!(
                backtrace = %er.backtrace(),
                error = %er,
            );

            Err(1_i32)
        }
    }
}

#[allow(clippy::too_many_lines)]
fn start() -> anyhow::Result<Result<(), i32>> {
    let EmpdArgs {
        delete_if_empty,
        path,
    } = EmpdArgs::parse();

    let path_path = Path::new(&path);

    let path_path_str = path_path
        .to_str()
        .context("Could not convert path to a UTF-8 string")?;

    let result = fs::symlink_metadata(path_path);

    let exit_code = match result {
        Err(er) => match er.kind() {
            ErrorKind::NotFound => {
                eprintln!("Path \"{}\" does not exist", path_path_str.bold());

                Err(11_i32)
            }
            ErrorKind::PermissionDenied => {
                eprintln!("Permission to path \"{}\" was denied", path_path_str.bold());

                Err(12_i32)
            }
            _ => {
                anyhow::bail!(er);
            }
        },
        Ok(me) => {
            match me {
                me if me.is_dir() => {
                    let canonicalize_result = canonicalize(path_path_str, path_path)?
                        .context("Could not canonicalize directory path")?;

                    let read_dir = path_path.read_dir().context("Could not read directory")?;

                    let mut directories = 0_u32;
                    let mut files = 0_u32;
                    let mut symlinks = 0_u32;

                    for re in read_dir {
                        let di = re.context("Could not access directory entry")?;

                        let fi = di
                            .file_type()
                            .context("Could not get the directory entry's file type")?;

                        match fi {
                            fi if fi.is_dir() => {
                                directories += 1_u32;
                            }
                            fi if fi.is_file() => {
                                files += 1_u32;
                            }
                            fi if fi.is_symlink() => {
                                symlinks += 1_u32;
                            }
                            _ => {
                                anyhow::bail!(
                                    "Encountered directory entry that is not a directory, file, or symlink"
                                );
                            }
                        }
                    }

                    let total_items = directories + files + symlinks;

                    if total_items > 0_u32 {
                        println!(
                            " {}  Path \"{}\" is a {} (directories: {}, files: {}, symlinks: {}, total items: {})",
                            X.bold().red(),
                            canonicalize_result.bold(),
                            "non-empty directory".bold().red(),
                            bold_if_greater_than_zero(directories),
                            bold_if_greater_than_zero(files),
                            bold_if_greater_than_zero(symlinks),
                            bold_if_greater_than_zero(total_items)
                        );

                        Err(31_i32)
                    } else {
                        println!(
                            " {}  Path \"{}\" is an {}",
                            CHECK_MARK.bold().green(),
                            canonicalize_result.bold(),
                            "empty directory".bold().green()
                        );

                        if delete_if_empty {
                            eprintln!(
                                "Are you sure you want to delete empty directory \"{}\"? (\"y\")\n\
                                (Note that no file locking or revalidation is performed, and the directory may be non-empty by the time you respond to this prompt!)",
                                canonicalize_result.bold()
                            );

                            let input = &mut String::new();

                            io::stdin().read_line(input)?;

                            if input == "y\n" {
                                // TODO Status of path could have changed by now
                                fs::remove_dir(path_path)?;

                                println!(
                                    "Deleted empty directory \"{}\"",
                                    canonicalize_result.bold()
                                );

                                Ok(())
                            } else {
                                println!("Input was not \"y\", not deleting empty directory");

                                Err(32_i32)
                            }
                        } else {
                            Ok(())
                        }
                    }
                }
                me if me.is_file() => {
                    let canonicalize_result = canonicalize(path_path_str, path_path)?
                        .context("Could not canonicalize file path")?;

                    let len = me.len();

                    if len > 0_u64 {
                        println!(
                            " {}  Path \"{}\" is a {} (bytes: {})",
                            X.bold().red(),
                            canonicalize_result.bold(),
                            "non-empty file".bold().red(),
                            len.bold()
                        );

                        Err(21_i32)
                    } else {
                        println!(
                            " {}  Path \"{}\" is an {}",
                            CHECK_MARK.bold().green(),
                            canonicalize_result.bold(),
                            "empty file".bold().green()
                        );

                        if delete_if_empty {
                            eprintln!(
                                "Are you sure you want to delete empty file \"{}\"? (\"y\")\n\
                                (Note that no file locking or revalidation is performed, and the file may be non-empty by the time you respond to this prompt!)",
                                canonicalize_result.bold()
                            );

                            let input = &mut String::new();

                            io::stdin().read_line(input)?;

                            if input == "y\n" {
                                // TODO Status of path could have changed by now
                                fs::remove_file(path_path)?;

                                println!("Deleted empty file \"{}\"", canonicalize_result.bold());

                                Ok(())
                            } else {
                                println!("Input was not \"y\", not deleting empty file");

                                Err(22_i32)
                            }
                        } else {
                            Ok(())
                        }
                    }
                }
                me if me.is_symlink() => {
                    let link_path_buf = path_path
                        .read_link()
                        .context("Could not read symbolic link")?;

                    let link_path_buf_str = link_path_buf
                        .to_str()
                        .context("Could not convert symbolic link path to a UTF-8 string")?;

                    let canonicalize_result = canonicalize(path_path_str, path_path)?;

                    #[allow(clippy::single_match_else)]
                    {
                        match canonicalize_result {
                            Some(st) => {
                                println!(
                                    " {}  Path \"{}\" (non-canonicalized) is a symbolic link to \"{}\" (resolves to \"{st}\")",
                                    X.bold().red(),
                                    path_path_str.bold(),
                                    link_path_buf_str.bold()
                                );

                                Err(41_i32)
                            }
                            None => {
                                println!(
                                    " {}  Path \"{}\" (non-canonicalized) is a symbolic link to non-existent file \"{}\" (non-canonicalized)",
                                    CHECK_MARK.bold().green(),
                                    path_path_str.bold(),
                                    link_path_buf_str.bold()
                                );

                                if delete_if_empty {
                                    eprintln!(
                                        "Are you sure you want to delete symbolic link \"{}\" (non-canonicalized) pointing to non-existent file \"{}\"? (non-canonicalized) (\"y\")\n\
                                        (Note that no file locking or revalidation is performed, and the symbolic link destination may exist by the time you respond to this prompt!)",
                                        path_path_str.bold(),
                                        link_path_buf_str.bold()
                                    );

                                    let input = &mut String::new();

                                    io::stdin().read_line(input)?;

                                    if input == "y\n" {
                                        // TODO
                                        // Status of path could have changed by now
                                        fs::remove_file(path_path)?;

                                        println!(
                                            "Deleted symbolic link \"{}\" (non-canonicalized)",
                                            path_path_str.bold()
                                        );

                                        Ok(())
                                    } else {
                                        println!("Input was not \"y\", not deleting symbolic link");

                                        Err(42_i32)
                                    }
                                } else {
                                    Ok(())
                                }
                            }
                        }
                    }
                }
                _ => {
                    anyhow::bail!("Path \"{path_path_str}\" is not a directory, file, or symlink")
                }
            }
        }
    };

    if let Err(it) = exit_code {
        eprintln!("Exiting with non-zero exit code {}", it.bold());
    }

    Ok(exit_code)
}

fn bold_if_greater_than_zero(input: u32) -> String {
    if input > 0_u32 {
        input.bold().to_string()
    } else {
        input.to_string()
    }
}

fn canonicalize(path_str: &str, path_path: &Path) -> anyhow::Result<Option<String>> {
    let canonicalize_result = fs::canonicalize(path_path);

    let option = match canonicalize_result {
        Ok(pa) => {
            let path_buf_str = pa
                .to_str()
                .context("Could not convert path to a UTF-8 string")?;

            eprintln!(
                "Canonicalized input path \"{}\" to \"{}\"",
                path_str.bold(),
                path_buf_str.bold()
            );

            Some(path_buf_str.to_owned())
        }
        Err(er) => match er.kind() {
            ErrorKind::NotFound => {
                eprintln!(
                        "Could not canonicalize input path \"{}\" because it or the file it resolves to does not exist",
                        path_str.bold()
                    );

                None
            }
            _ => {
                anyhow::bail!(er);
            }
        },
    };

    Ok(option)
}

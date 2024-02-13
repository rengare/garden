use edit::{edit_file, Builder};
use miette::Diagnostic;
use owo_colors::OwoColorize;
use std::{
    fs,
    io::{self, Write},
    ops::Not,
    path::PathBuf,
};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum GardenVarietyError {
    #[error(transparent)]
    #[diagnostic(code(garden::io_error))]
    IoError(#[from] std::io::Error),

    #[error("failed to create tempfile: {0}")]
    #[diagnostic(code(garden::tempfile_create_error))]
    TempfileCreationError(std::io::Error),

    #[error("failed to keep tempfile: {0}")]
    #[diagnostic(code(garden::tempfile_keep_error))]
    TempfileKeepError(#[from] tempfile::PersistError),

    #[error("Unable to read tempfile after passing edit control to user:\ntempfile: {filepath}\n{io_error}")]
    #[diagnostic(
        code(garden::tempfile_read_error),
        help("Make sure your editor isn't moving the file away from the temporary location")
    )]
    TempfileReadError {
        filepath: PathBuf,
        io_error: std::io::Error,
    },
}

pub fn write(
    garden_path: PathBuf,
    title: Option<String>,
) -> miette::Result<(), GardenVarietyError> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&garden_path)
        .map_err(|e| GardenVarietyError::TempfileCreationError(e))?
        .keep()?;

    let template = format!("# {}", title.as_deref().unwrap_or(""));
    file.write_all(template.as_bytes())?;
    edit_file(&filepath)?;
    let contents =
        fs::read_to_string(&filepath).map_err(|e| GardenVarietyError::TempfileReadError {
            filepath: filepath.clone(),
            io_error: e,
        })?;

    let document_title = title.or_else(|| title_from_content(&contents));

    let filename = match document_title {
        Some(raw_title) => confirm_filename(&raw_title),
        None => ask_for_filename(),
    }
    .map(|title| slug::slugify(title))?;

    for attempt in 0.. {
        let mut dest = garden_path.join(if attempt == 0 {
            filename.clone()
        } else {
            format!("{filename}{:03}", -attempt)
        });
        dest.set_extension("md");
        if dest.exists() {
            continue;
        }
        fs::rename(filepath, &dest)?;
        break;
    }

    Ok(())
}

fn ask_for_filename() -> io::Result<String> {
    rprompt::prompt_reply("Enter a filename: ".blue().bold())
}

fn confirm_filename(raw_title: &str) -> io::Result<String> {
    loop {
        let result = rprompt::prompt_reply(&format!(
            "current title: {}
Do you want a different title? (y/N): ",
            &raw_title,
        ))?;

        match result.as_str() {
            "y" | "Y" => break ask_for_filename(),
            "n" | "N" | "" => {
                break Ok(raw_title.to_string());
            }
            _ => {
                // ask again because something went wrong
            }
        };
    }
}

fn title_from_content(input: &str) -> Option<String> {
    input.lines().find_map(|line| {
        line.strip_prefix("# ")
            .and_then(|title| title.is_empty().not().then_some(title.to_string()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_from_empty_string() {
        let input = "";
        let result = title_from_content(input);
        assert_eq!(result, None);
    }

    #[test]
    fn title_from_string() {
        let expected = "Hello, world!";
        let input = &format!("# {}", &expected);
        let result = title_from_content(&input.as_str());

        assert_eq!(result, Some(expected.to_string()));
    }

    #[test]
    fn title_from_content_with_only_hash() {
        let input = "# ";
        let result = title_from_content(input);
        assert_eq!(result, None);
    }
}

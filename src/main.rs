use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use ironworks::{
    Ironworks,
    excel::Excel,
    sqpack::{Install, SqPack},
};
use regex::Regex;
mod exd_schema;
mod export;
mod formatter;

#[derive(Debug, Parser)]
struct Args {
    /// Set the input directory containing the game install.
    ///
    /// The directory must be the base directory for the game install, i.e., it must contain the `game` subdirectory.
    #[arg(short, long = "input", value_name = "PATH")]
    input_dir: PathBuf,
    /// Set the output directory.
    ///
    /// Directories are created if they are missing. The default creates a new `output` directory in the current working directory.
    #[arg(short, long = "output", value_name = "PATH", default_value = "output")]
    output_dir: PathBuf,
    /// Set the output format used for SeStrings.
    #[arg(short, long, default_value = "markdown")]
    string_format: StringFormat,
    /// Set the color scheme used when formatting Strings as HTML.
    ///
    /// The argument is ignored for other formats as they don't generate color specific output.
    #[arg(short, long, default_value = "dark")]
    color_scheme: ColorScheme,
    /// Include sheets that matching the regex. If ommited, all sheets are included.
    #[arg(long = "include", value_name = "REGEX", default_value = None)]
    include_regex: Option<Regex>,
    /// Exclude sheets that matching the regex. If ommited, no sheets are excluded.
    #[arg(long = "exclude", value_name = "REGEX", default_value = None)]
    exclude_regex: Option<Regex>,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum StringFormat {
    Markdown,
    PlainText,
    Html,
    MacroString,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[repr(u8)]
enum ColorScheme {
    Dark = 0,
    Light = 1,
    ClassicFf = 2,
    ClearBlue = 3,
    ClearWhite = 4,
    ClearGreen = 5,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args {
        input_dir,
        output_dir,
        string_format,
        color_scheme,
        include_regex,
        exclude_regex,
    } = Args::parse();

    if !input_dir.is_dir()
        || !input_dir.join("game").is_dir()
        || !input_dir.join("game").join("sqpack").is_dir()
    {
        return Err("Invalid input game directory".into());
    }

    let ironworks = Ironworks::new().with_resource(SqPack::new(Install::at(&input_dir)));
    let languages = export::available_languages(&ironworks);
    let mut excel = Excel::new(ironworks);
    let input = export::build_input(&excel, color_scheme, string_format)?;

    for language in languages {
        excel.set_default_language(language);
        let sheets = excel.list().expect("Could not retrieve sheet list.");

        println!(
            "Exporting {} sheets",
            export::language_code(&language).to_uppercase()
        );

        for sheet in sheets.iter().filter(|sheet| {
            include_regex
                .as_ref()
                .is_none_or(|regex| regex.is_match(&sheet))
                && exclude_regex
                    .as_ref()
                    .is_none_or(|exclude| !exclude.is_match(&sheet))
        }) {
            match export::sheet(&excel, language, &sheet, &input, &output_dir, string_format) {
                Ok(_) => (),
                // Log failed sheets and continue
                Err(err) => eprintln!("Failed to export {}. {}", sheet, err),
            }
        }
    }

    // Quick debugging for schema updates

    // for language in languages {
    //     excel.set_default_language(language);
    //     export::sheet(&excel, language, &String::from("Mount"))?;
    // }

    // let language = Language::English;
    // excel.set_default_language(language);
    // export::sheet(&excel, language, "Mount")?;

    Ok(())
}

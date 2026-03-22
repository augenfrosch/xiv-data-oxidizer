use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use ironworks::{
    Ironworks,
    excel::Excel,
    sqpack::{Install, SqPack},
};
mod exd_schema;
mod export;
mod formatter;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, alias = "input")]
    input_dir: PathBuf,
    #[arg(short, long, alias = "output", default_value = "output")]
    output_dir: PathBuf,
    #[arg(short, long, default_value = "markdown")]
    string_format: StringFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StringFormat {
    Markdown,
    PlainText,
    Html,
    RawRepresentation,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args {
        input_dir,
        output_dir,
        string_format,
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

    for language in languages {
        excel.set_default_language(language);
        let sheets = excel.list().expect("Could not retrieve sheet list.");

        println!(
            "Exporting {} sheets",
            export::language_code(&language).to_uppercase()
        );

        for sheet in sheets.iter() {
            match export::sheet(&excel, language, &sheet, &output_dir, string_format) {
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

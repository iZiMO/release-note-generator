use csv::Error;
use clap::Parser;
use std::{collections::HashMap, io::Write};
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    translations: String,
    #[arg(short, long)]
    app_config: String
}

#[derive(Deserialize, Serialize, Debug)]
struct AppConfig {
    name: String,
    languages: Vec<String>
}

#[derive(Deserialize, Serialize, Debug)]
struct AllAppConfig {
    apps: Vec<AppConfig>
}

// E.g. en-US or ms
struct Locale {
    language: String,
    territory: Option<String>
}

fn check_file_exists(file_name: &str) -> Result<(), Error> {
    let _file = fs::File::open(file_name)?;
    Ok(())
}

fn parse_locale(raw: &str) -> Locale {
    let parts: Vec<&str> = raw.split("-").collect();
    let language = parts[0].to_string();
    let territory = if parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        None
    };
    Locale {
        language,
        territory
    }
}

/// Generates an XML(ish) file containing the translations for a single app
/// # Arguments
/// * `app_name` - The name of the app, the folder in which the output file will be created
/// * `translation_name` - The name of the translation, used for the file name
/// * `locales` - The locales required by the app (defined in app-config.toml)
/// * `translations` - The translations for the current row of the CSV file
fn create_output_files(app_name: &String, translation_name: &String, locales: &Vec<Locale>, translations: &HashMap<String, String>) {
    let output_dir = String::from("./output/") + &app_name;
    fs::create_dir_all(&output_dir).expect("Error creating output directory");
    let output_file_name = output_dir + "/" + translation_name + ".xml";
    let mut output_file = fs::File::create(output_file_name.clone()).expect("Error creating output file");
    let mut output = String::new();
    for locale in locales {
        let notes = translations.get(&locale.language);
        if notes.is_none() {
            println!("No translation for {}", locale.language);
            continue;
        }

        let mut tag = locale.language.clone();
        if let Some(territory) = &locale.territory {
            tag += &("-".to_owned() + territory);
        }
        output += &format!("\n<{}>", tag);
        output += &format!("\n{}", notes.unwrap());
        output += &format!("\n</{}>", tag);
    }
    output_file.write_all(output.as_bytes()).expect("Error writing to output file");
    println!("  {}", output_file_name.clone());
}

fn read_app_config(app_config_filename: String) -> Result<HashMap<String,Vec<Locale>>, Error> {
    let config: AllAppConfig = {
        let config_text = fs::read_to_string(app_config_filename).expect("Error reading app-config.toml");
        toml::from_str(&config_text).expect("Error reading stream")
    };
    let mut result: HashMap<String, Vec<Locale>> = HashMap::new();
    for app in config.apps {
        result.insert(
            app.name,
            app.languages.iter().map(|raw| parse_locale(raw)).collect()
        );
    }
    Ok(result)
}

fn main() -> Result<(), Error>{
    let matches = Args::parse();
    let mut file_exists = check_file_exists(&matches.translations);
    if file_exists.is_err() {
        println!("Error opening translations file: {}", matches.translations);
        std::process::exit(1);
    }

    file_exists = check_file_exists(&matches.app_config);
    if file_exists.is_err() {
        println!("Error opening app config file: {}", matches.app_config);
        std::process::exit(1);
    }

    if std::path::Path::new("./output").exists() {
        fs::remove_dir_all("./output").expect("Failed to clean output directory");
    }

    let app_config = read_app_config(matches.app_config).expect("Error parsing app config");

    let mut reader = csv::ReaderBuilder::new()
        .from_path(matches.translations)?;

    let mut record_num = 0;
    for record in reader.deserialize() {
        let translations: HashMap<String, String> = record?;
        let translation_name = match translations.get("Description") {
            Some(name) => name.clone(),
            None => {
                println!("No translation name found - defaulting to record number");
                record_num.to_string()
            }
        };

        println!("Creating output files for {}", translation_name);
        for (app_name, locales) in &app_config {
            create_output_files(app_name, &translation_name,  locales, &translations);
        }
        record_num += 1;
        println!();
    }
    
    Ok(())
}

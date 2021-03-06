use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

type ExtExportMap = HashMap<String, String>;
static FILTER_DATA_PATH: &str = "data/custom_project";
static EXPORT_FILES_MAP: &str = "data/custom_project/config.json";

/// Gets the target locations for copying the project files to the RP adn BP
/// based on the data in the map.
///
/// # Arguments
/// - `fp` - the path to the file that needs to be copied relative to the
///   data directory.
fn get_target_path_from_hash_map(
        fp: &Path, map: &ExtExportMap
) -> Option<PathBuf> {
    let file_name = fp.file_name()?.to_str()?;
    // Find matching file extension and the export target
    let (extension, target) = {
        let result = map.iter()
            .find(|(k, _)| file_name.ends_with(k.as_str()))?;
        (result.0.as_str(), result.1.as_str())
    };
    // If the file name is just an extension or the file name is and underscore
    // with extension (e.g _.bpe.json) then use the parent fodler as the
    // actual file name.
    let (base_name, base_path): (String, PathBuf);
    if file_name == extension || file_name == &format!("_{}", extension) {
        base_name = fp.parent()?.file_name()?.to_str()?.to_string()
            + &extension;
        base_path = fp.parent()?.parent()?.to_path_buf();
    } else {
        base_name = fp.file_name()?.to_str()?.to_string();
        base_path = fp.parent()?.to_path_buf();
    }
    // Fix the path separators (e.g "/" -> "\\")
    let target: PathBuf = PathBuf::from(target).iter().collect();
    // Return
    Some(target.join(base_path).join(base_name))
}


/// Recursively copies the files starting from the curr_dir with export paths
/// relative to the root_dir generated based on the data in the
/// export_map. This function is used in copy_files_by_roots function.
/// 
/// # Arguments
/// - `curr_dir` - the directory to copy the files from
/// - `root_dir` - the root directory to copy the files to in most cases it
///     will be the same as the curr_dir, it's used for the recursive call
/// - `working_dir` - the working directory of the script, the target paths
///     of the exporter are relative to this directory
/// - `export_map` - the map of file extensions and rules to generate the
///     export paths
fn copy_files(
    curr_dir: &Path, root_dir: &Path, working_dir: &Path,
    export_map: &ExtExportMap,
) -> Result<(), Box<dyn Error>> {
    // Walk files in current directory
    let dir = match fs::read_dir(curr_dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to read directory: {}", curr_dir.display());
            return Err(Box::new(e));
        }
    };
    for fp in dir {
        let fp = fp?.path();

        // Directory - recurse
        if fp.is_dir() {
            copy_files(&fp, root_dir, working_dir, export_map)?;
            continue;
        }

        // Not a directory - find the target and copy file
        // Find the tartet
        let root_len = root_dir.components().count();
        let short_fp = fp.components().skip(root_len).collect::<PathBuf>();
        let target_path = match get_target_path_from_hash_map(
            &short_fp, export_map
        ) {
            Some(p) => working_dir.join(p),
            None => {
                eprintln!(
                    "Unable to map \"{}\" to the pack file. Skipped.",
                    fp.display()
                );
                continue;
            }
        };

        // Copy file
        if target_path.exists() {
            eprintln!(
                "WARNING! File \"{}\" already exists. Skipped.",
                target_path.display()
            );
            continue;
        }
        fs::create_dir_all(target_path.parent().unwrap())?;
        match fs::copy(&fp, &target_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "WARNING! Unable to copy \"{}\" to \"{}\": {}",
                    fp.display(), target_path.display(), e
                );
            }
        }
    }
    return Ok(());
}


/// Copies the files from the data directory of the project files path to the
/// target directories which are found based on the export map.
///
/// # Arguments
/// - `working_dir` - the path to the working directory of regolith
/// - `export_map` - the map that contains the mapping of file extensions to
///     the target directories.
/// - `roots` - the list of paths, relative to the filter data path, that
///     serve as the root of the source files to be copied to RP and BP.
fn copy_files_by_roots(
    working_dir: &Path, export_map: &ExtExportMap, roots: &Vec<String>,
) -> Result<(), Box<dyn Error>>{
    for root in roots {
        let root = working_dir.join(FILTER_DATA_PATH).join(root);
        println!("Copying files from \"{}\"", root.display());
        copy_files(&root, &root, working_dir, export_map)?;
    }
    return Ok(());
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let working_dir = match args.len() {
        1 => PathBuf::from(""),
        _ => PathBuf::from(&args[1]),
    };
    // Load JSON from EXPORT_FILES_MAP
    let export_map_path = working_dir.join(EXPORT_FILES_MAP);
    let config: serde_json::Value = match fs::read_to_string(export_map_path) {
        Ok(s) => serde_json::from_str(&s)?,
        Err(e) => {
            eprintln!("Unable to read \"{}\": {}", EXPORT_FILES_MAP, e);
            return Ok(());
        }
    };
    // Get extensions_map from the config
    let export_map: ExtExportMap = match config["extensions_map"].as_object() {
        Some(m) =>  m.iter()
        .map(|(k, v)| (k.to_string(), v.as_str().unwrap().to_string()))
        .collect(),
        None => {
            eprintln!(
                "Failed to parse \"extensions_map\" property in config \
                file: \"{}\"", EXPORT_FILES_MAP
            );
            return Ok(())
        }
    };
    // Get roots from the config
    let roots: Vec<String> = match config["roots"].as_array() {
        Some(r) => r.iter().map(|v| v.as_str().unwrap().to_string()).collect(),
        None => {
            eprintln!(
                "Failed to parse \"roots\" property in config file: \"{}\"",
                EXPORT_FILES_MAP);
                return Ok(())
            }
        };
    // Copy the files from the data directory to packs
    println!("Copying files to packs...");
    match copy_files_by_roots(&working_dir, &export_map, &roots) {
        Ok(_) => {}
        Err(err) => eprintln!("{}", err),
    }
    Ok(())
}

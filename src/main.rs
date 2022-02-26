use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

type ExtExportMap = HashMap<String, String>;
static PROJECT_FILES_PATH: &str = "data/custom_project/project";
static EXPORT_FILES_MAP: &str = "data/custom_project/export_map.json";

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
    if file_name == extension || file_name == &format!("_.{}", extension) {
        base_name = fp.parent()?.file_name()?.to_str()?.to_string()
            + "." + &extension;
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

/// Copies the files from the data directory of the project files path to the
/// target directories which are found based on the export map.
///
/// # Arguments
/// - `working_dir` - the path to the working directory of regolith
/// - `export_map` - the map that contains the mapping of file extensions to
///     the target directories.
fn copy_files(
        working_dir: &Path, export_map: &ExtExportMap
) -> Result<(), Box<dyn Error>> {
    // Walks the data directory
    fn cp_filers(
        curr_dir: &Path, project_file_path: &Path, working_dir: &Path,
        export_map: &ExtExportMap,
    ) -> Result<(), Box<dyn Error>> {
        // Walk files in current directory
        for fp in fs::read_dir(curr_dir)? {
            let fp = fp?.path();

            // Directory - recurse
            if fp.is_dir() {
                cp_filers(&fp, project_file_path, working_dir, export_map)?;
                continue;
            }

            // Not a directory - find the target and copy file
            // Find the tartet
            let root_len = project_file_path.components().count();
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
                    "File \"{}\" already exists. Skipped.",
                    target_path.display()
                );
                continue;
            }
            fs::create_dir_all(target_path.parent().unwrap())?;
            match fs::copy(&fp, &target_path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "Unable to copy \"{}\" to \"{}\": {}",
                        fp.display(), target_path.display(), e
                    );
                }
            }
        }
        return Ok(());
    }
    let project_files = working_dir.join(PROJECT_FILES_PATH);
    cp_filers(&project_files, &project_files, working_dir, export_map)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let working_dir = match args.len() {
        1 => PathBuf::from(""),
        _ => PathBuf::from(&args[1]),
    };
    // Load JSON from EXPORT_FILES_MAP
    let export_map_path = working_dir.join(EXPORT_FILES_MAP);
    let export_map: ExtExportMap = match fs::read_to_string(export_map_path) {
        Ok(s) => serde_json::from_str(&s)?,
        Err(e) => {
            eprintln!("Unable to read \"{}\": {}", EXPORT_FILES_MAP, e);
            return Ok(());
        }
    };
    // Copy the files from the data directory to packs
    match copy_files(&working_dir, &export_map) {
        Ok(_) => {}
        Err(err) => eprintln!("{}", err),
    }
    Ok(())
}

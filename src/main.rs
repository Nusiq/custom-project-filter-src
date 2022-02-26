use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

static FILTER_DATA: &str = "restructured_project";

/// Gets the file path of Minecraft RP/BP resource file relative to data
/// path, that follows specific naming convention, and based on that name,
/// returns the path where the file should be copied to in actual resouce pack
/// or behavior pack.
///
/// # Arguments
/// - `fp` - the path to the file that needs to be copied relative to the
///   data directory.
fn get_target_path(fp: &Path) -> Option<PathBuf> {
    let rp = Path::new("RP");
    let bp = Path::new("BP");
    let mut file_name_split = fp
        .file_name()?
        .to_str()?
        .split('.')
        .collect::<Vec<&str>>();
    // Get extension: JSON gts additional subextension e.g. the extension of
    // "example.rc.json" is "rc.json". Any other typ of file gets single value.
    let mut extension = String::from(file_name_split.pop()?);
    if extension == "json" {
        extension = String::from(file_name_split.pop()?) + ".json";
    };
    // If the file nam is just an extension then use the parent fodler as the
    // base name.
    let base_name: String;
    let base_path: PathBuf;
    if file_name_split.len() == 0 {
        base_name = fp.parent()?.file_name()?.to_str()?.to_string()
            + "." + &extension;
        base_path = fp.parent()?.parent()?.to_path_buf();
    } else {
        base_name = fp.file_name()?.to_str()?.to_string();
        base_path = fp.parent()?.to_path_buf();
    }
    let file = base_path.join(base_name);
    match extension.as_str() {
        "lang" => Some(rp.join("texts").join(file)),
        "mcfunction" => Some(bp.join("functions").join(file)),
        "mcstructure" => Some(bp.join("structures").join(file)),
        "wav" | "ogg" | "fsb" | "mp4" => Some(rp.join("sounds").join(file)),
        "png" | "tga" => Some(rp.join("textures").join(file)),
        "bpac.json" => Some(bp.join("animation_controllers").join(file)),
        "rpac.json" => Some(rp.join("animation_controllers").join(file)),
        "bpa.json" => Some(bp.join("animations").join(file)),
        "rpa.json" => Some(rp.join("animation").join(file)),
        "bpe.json" => Some(bp.join("entities").join(file)),
        "rpe.json" => Some(rp.join("entity").join(file)),
        "bpb.json" => Some(bp.join("blocks").join(file)),
        "bpi.json" => Some(bp.join("items").join(file)),
        "rpi.json" => Some(rp.join("item").join(file)),
        "i.json" => Some(bp.join("items").join(file)), // BP item (new format)
        "biome.json" => Some(bp.join("biomes").join(file)),
        "f.json" => Some(bp.join("features").join(file)),
        "fr.json" => Some(bp.join("feature_rules").join(file)),
        "at.json" => Some(rp.join("attachables").join(file)),
        "fog.json" => Some(rp.join("fogs").join(file)),
        "geo.json" => Some(rp.join("models").join("entity").join(file)),
        "rc.json" => Some(rp.join("render_controllers").join(file)),
        "sr.json" => Some(bp.join("spawn_rules").join(file)),
        "p.json" => Some(rp.join("particles").join(file)),
        "r.json" => Some(bp.join("recipes").join(file)),
        "lt.json" => Some(bp.join("loot_tables").join(file)),
        "tt.json" => Some(bp.join("trading").join(file)),
        _ => None,
    }
}

/// Copies the files from the data directory of the filter to the resource pack
/// or behavior pack, based on the outputs from the get_target_path function.
///
/// # Arguments
/// - `working_dir` - the path to the working directory of regoliht
fn copy_filees(working_dir: &Path) -> Result<(), Box<dyn Error>> {
    fn walk(
            curr_dir: &Path, root_dir: &Path, working_dir: &Path
    ) -> Result<(), Box<dyn Error>> {
        let root_len = root_dir.components().count();
        for entry in fs::read_dir(curr_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(&path, root_dir, working_dir)?;
            } else {
                let relative_path = path.components().skip(root_len)
                    .collect::<PathBuf>();
                let target_path = match get_target_path(&relative_path) {
                    Some(p) => working_dir.join(p),
                    None => {
                        eprintln!(
                            "Unable to map \"{}\" to the pack file. Skipped.",
                            relative_path.display()
                        );
                        continue;
                    }
                };
                // println!("{} -> {}", path.display(), target_path.display());
                // Unwrap is safe because "get_target_path" always returns
                // path with parent
                fs::create_dir_all(target_path.parent().unwrap())?;
                match fs::copy(&path, &target_path) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!(
                            "Unable to copy \"{}\" to \"{}\": {}",
                            path.display(), target_path.display(), e
                        );
                    }
                }
            }
        }
        return Ok(());
    }
    let data_dir = working_dir.join("data").join(FILTER_DATA);
    walk(&data_dir, &data_dir, working_dir)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let working_dir = match args.len() {
        1 => PathBuf::from(""),
        _ => PathBuf::from(&args[1]),
    };
    match copy_filees(&working_dir) {
        Ok(_) => {}
        Err(err) => println!("{}", err),
    }
}

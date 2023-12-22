fn main() {
  let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  println!("cargo:rustc-link-search=native={}\\lib", project_dir);

  let target_path = std::path::Path::new(&project_dir).join("target");

  if target_path.exists() {
    let debug_path = target_path.join("debug");
    let release_path = target_path.join("release");
    let dll_files = std::fs::read_dir(std::path::Path::new(&(project_dir + "\\dll")))
      .unwrap()
      .map(|res| res.map(|e| e.path()))
      .collect::<Result<Vec<_>, std::io::Error>>()
      .unwrap();

    for path in dll_files {
      if debug_path.exists() {
        std::fs::copy(&path, debug_path.join(&path.file_name().unwrap())).unwrap();
      }
      if release_path.exists() {
        std::fs::copy(&path, release_path.join(&path.file_name().unwrap())).unwrap();
      }
    }
  }
}

use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

pub fn zip_file(infile: &str, outfile: &str) -> zip::result::ZipResult<()> {
    let path = Path::new(infile);
    let file = File::open(&path)?;

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    let mut zip = ZipWriter::new(File::create(outfile)?);
    zip.start_file(path.file_name().unwrap().to_str().unwrap(), options)?;
    std::io::copy(&mut file.try_clone()?, &mut zip)?;

    zip.finish()?;

    Ok(())
}

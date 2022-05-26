use actix_files as fs;
use actix_web::{get, Error, HttpRequest};
use std::path::PathBuf;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_file);
}

#[get("/assets/{filename:.*}")]
async fn view_file(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let mut path: PathBuf = PathBuf::from("public/assets/");
    let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    path.push(req_path);

    let file = fs::NamedFile::open(path)?;

    Ok(file.use_last_modified(true))
}

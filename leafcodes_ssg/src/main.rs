use anyhow::{anyhow, Ok, Result};
use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Router};
use handlebars::Handlebars;
use hotwatch::blocking::{Flow, Hotwatch};
use pulldown_cmark::{html, Parser};
use serde_json::json;
use std::{
    ffi::OsStr,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tower_http::services::ServeDir;
use walkdir::{DirEntry, WalkDir};

const BUILD_PATH: &str = "../build/";
const SOURCE_PATH: &str = "../site_src/";

const CONTENT_DIR: &str = "content";
const INCLUDE_DIR: &str = "include";
const TEMPLATE_DIR: &str = "template";

#[tokio::main]
async fn main() -> Result<()> {
    match std::env::args().nth(1) {
        Some(str) if str.as_str() == "build" => build(&mut Handlebars::new()),
        Some(str) if str.as_str() == "dev" => dev(Handlebars::new()).await,
        _ => Err(anyhow!(
            "Please provide either 'build' or 'dev' as first argument."
        )),
    }
}

fn build(handlebars: &mut Handlebars) -> Result<()> {
    register_templates(handlebars)
        .and(write_html(handlebars))
        .and(copy_includes())
}

async fn dev(mut handlebars: Handlebars<'static>) -> Result<()> {
    handlebars.set_dev_mode(true);
    build(&mut handlebars)?;

    tokio::task::spawn_blocking(move || {
        let watch_handler = move |_| {
            write_html(&handlebars).unwrap();
            copy_includes().unwrap();
            println!("Site rebuilt!");
            Flow::Continue
        };
        let mut hotwatch = Hotwatch::new().unwrap();
        hotwatch.watch(SOURCE_PATH, watch_handler).unwrap();
        hotwatch.run();
    });

    let service = get_service(ServeDir::new(BUILD_PATH)).handle_error(handle_error);
    let app = Router::new().fallback(service);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("Serving site on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

fn register_templates(handlebars: &mut Handlebars) -> Result<()> {
    let template_path = [SOURCE_PATH, TEMPLATE_DIR].concat();
    for path in get_file_paths(template_path.as_str()) {
        let name = &path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or(anyhow!("Bad file path in template directory"))?;
        handlebars.register_template_file(name, &path)?;
    }

    Ok(())
}

fn write_html(handlebars: &Handlebars) -> Result<()> {
    // Clean build directory
    _ = fs::remove_dir_all(BUILD_PATH);

    let content_path = [SOURCE_PATH, CONTENT_DIR].concat();
    for path in get_file_paths(content_path.as_str()) {
        let md_content = fs::read_to_string(&path)?;
        let md_parser = Parser::new_ext(&md_content, pulldown_cmark::Options::all());

        let mut html_content = String::new();
        html::push_html(&mut html_content, md_parser);

        // TODO: select correct template
        let html_full = handlebars.render("index", &json!({ "main": html_content }))?;

        let bare_path = path.strip_prefix(&content_path)?;
        let html_path = Path::new(BUILD_PATH).join(bare_path).with_extension("html");

        fs::create_dir_all(html_path.parent().unwrap())?;
        fs::write(&html_path, html_full)?;
    }

    Ok(())
}

fn copy_includes() -> Result<()> {
    let include_path = [SOURCE_PATH, INCLUDE_DIR].concat();
    for from_path in get_file_paths(include_path.as_str()) {
        let bare_path = from_path.strip_prefix(&include_path)?;
        let to_path = Path::new(BUILD_PATH).join(bare_path);

        fs::create_dir_all(to_path.parent().unwrap())?;
        fs::copy(from_path, to_path)?;
    }

    Ok(())
}

fn get_file_paths(path: &str) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.metadata().unwrap().is_file())
        .map(DirEntry::into_path)
}

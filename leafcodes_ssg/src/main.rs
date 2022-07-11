use anyhow::{anyhow, Ok, Result};
use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Router};
use const_format::concatcp;
use handlebars::Handlebars;
use hotwatch::blocking::{Flow, Hotwatch};
use pulldown_cmark::{html, LinkDef, Parser};
use serde::Serialize;
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

const CONTENT_PATH: &str = concatcp!(SOURCE_PATH, "content");
const INCLUDE_PATH: &str = concatcp!(SOURCE_PATH, "include");
const TEMPLATE_PATH: &str = concatcp!(SOURCE_PATH, "template");

#[derive(Serialize, Debug)]
struct Metadata {
    template: String,
    title: String,
    date: String,
}

#[derive(Debug)]
struct Page {
    content: String,
    metadata: Metadata,
    out_path: PathBuf,
}

#[derive(Serialize, Debug)]
struct BlogPost<'a> {
    title: &'a str,
    date: &'a str,
    filename: &'a str,
}

fn main() -> Result<()> {
    match std::env::args().nth(1) {
        Some(str) if str.as_str() == "build" => {
            let mut handlebars = Handlebars::new();
            register_templates(&mut handlebars)?;
            build(&mut handlebars)
        }
        Some(str) if str.as_str() == "dev" => dev(),
        _ => Err(anyhow!("Please provide either 'build' or 'dev' as first argument.")),
    }
}

fn register_templates(handlebars: &mut Handlebars) -> Result<()> {
    for path in get_file_paths(TEMPLATE_PATH) {
        let name = &path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or(anyhow!("Bad file path in template directory"))?;
        handlebars.register_template_file(name, &path)?;
    }

    Ok(())
}

fn build(handlebars: &mut Handlebars) -> Result<()> {
    _ = fs::remove_dir_all(BUILD_PATH); // Clean build directory
    let pages = parse_pages()?;
    let blog_posts = process_blog_posts(&pages)?;
    generate_html(handlebars, &pages, &blog_posts)?;
    copy_includes()
}

#[tokio::main]
async fn dev() -> Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    register_templates(&mut handlebars)?;
    build(&mut handlebars)?;

    tokio::task::spawn_blocking(move || {
        let watch_handler = move |_| {
            build(&mut handlebars).unwrap();
            println!("Site rebuilt!");
            Flow::Continue
        };
        let mut hotwatch = Hotwatch::new().unwrap();
        hotwatch.watch(SOURCE_PATH, watch_handler).unwrap();
        hotwatch.run();
    });

    async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
    }

    let service = get_service(ServeDir::new(BUILD_PATH)).handle_error(handle_error);
    let app = Router::new().fallback(service);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("Serving site on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

fn parse_pages() -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    for path in get_file_paths(CONTENT_PATH) {
        let md_content = fs::read_to_string(&path)?;
        let md_parser = Parser::new_ext(&md_content, pulldown_cmark::Options::all());
        let metadata = extract_metadata(&md_parser);

        let mut content = String::new();
        html::push_html(&mut content, md_parser);

        let bare_path = path.strip_prefix(CONTENT_PATH)?;
        let out_path = Path::new(BUILD_PATH).join(bare_path).with_extension("html");

        pages.push(Page { content, metadata, out_path })
    }

    Ok(pages)
}

fn process_blog_posts(pages: &[Page]) -> Result<Vec<BlogPost>> {
    let mut blog_posts = pages
        .iter()
        .filter(|page| page.metadata.template.eq("blog_post"))
        .map(|page| BlogPost {
            title: &page.metadata.title,
            date: &page.metadata.date,
            filename: page.out_path.as_path().file_name().unwrap().to_str().unwrap(),
        })
        .collect::<Vec<BlogPost>>();

    // sort by date desc
    blog_posts.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(blog_posts)
}

fn generate_html(handlebars: &Handlebars, pages: &[Page], blog_posts: &[BlogPost]) -> Result<()> {
    for page in pages {
        let mut data = json!({ "content": &page.content, "metadata": &page.metadata });
        if page.metadata.template.eq("blog_index") {
            data.as_object_mut().unwrap().insert("blog_posts".to_string(), json!(blog_posts));
        }

        let html = handlebars.render(&page.metadata.template, &data)?;

        fs::create_dir_all(page.out_path.as_path().parent().unwrap())?;
        fs::write(&page.out_path, html)?;
    }

    Ok(())
}

fn copy_includes() -> Result<()> {
    for from_path in get_file_paths(INCLUDE_PATH) {
        let bare_path = from_path.strip_prefix(INCLUDE_PATH)?;
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

fn extract_metadata(parser: &Parser) -> Metadata {
    let refdefs = parser.reference_definitions();
    let to_dest = |def: &LinkDef| def.dest.to_string();
    let extract = |key: &str| refdefs.get(key).map(to_dest).unwrap_or(String::new());
    Metadata {
        template: extract("_metadata_:template"),
        title: str::replace(&extract("_metadata_:title"), "_", " "),
        date: extract("_metadata_:date"),
    }
}

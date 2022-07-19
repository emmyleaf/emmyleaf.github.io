use anyhow::{anyhow, Ok, Result};
use const_format::concatcp;
use handlebars::Handlebars;
use minify_html::{minify, Cfg};
use pulldown_cmark::{html, LinkDef, Parser};
use serde::Serialize;
use serde_json::json;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

const BUILD_PATH: &str = "../build/";
const SOURCE_PATH: &str = "../site_src/";

const BLOG_PATH: &str = concatcp!(BUILD_PATH, "blog");
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
    build(&init_handlebars()?)
}

fn init_handlebars() -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(cfg!(feature = "dev"));
    for path in get_file_paths(TEMPLATE_PATH) {
        let name = &path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or(anyhow!("Bad file path in template directory"))?;
        handlebars.register_template_file(name, &path)?;
    }
    Ok(handlebars)
}

fn build(handlebars: &Handlebars) -> Result<()> {
    _ = fs::remove_dir_all(BUILD_PATH); // Clean build directory
    let pages = parse_pages()?;
    let blog_posts = process_blog_posts(&pages)?;
    generate_pages_html(handlebars, &pages)?;
    generate_blog_index_html(handlebars, &blog_posts)?;
    copy_includes()
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

fn generate_pages_html(handlebars: &Handlebars, pages: &[Page]) -> Result<()> {
    let minify_cfg = Cfg { minify_js: true, ..Cfg::spec_compliant() };
    for page in pages {
        let data = json!({ "content": &page.content, "metadata": &page.metadata });
        let html = handlebars.render(&page.metadata.template, &data)?;
        let minified_html = minify(html.as_bytes(), &minify_cfg);

        fs::create_dir_all(page.out_path.as_path().parent().unwrap())?;
        fs::write(&page.out_path, minified_html)?;
    }

    Ok(())
}

fn generate_blog_index_html(handlebars: &Handlebars, blog_posts: &[BlogPost]) -> Result<()> {
    let minify_cfg = Cfg { minify_js: true, ..Cfg::spec_compliant() };
    let data = json!({ "title": "blog", "blog_posts": blog_posts });
    let html = handlebars.render("blog_index", &data)?;
    let minified_html = minify(html.as_bytes(), &minify_cfg);
    let out_file = Path::new(BLOG_PATH).join("index.html");

    fs::create_dir_all(BLOG_PATH)?;
    fs::write(&out_file, minified_html)?;

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
    let to_title = |def: &LinkDef| def.title.as_deref().unwrap().to_string();
    let extract = |key: &str| refdefs.get(key).map(to_title).unwrap_or(String::new());
    Metadata {
        template: extract("_metadata_:template"),
        title: extract("_metadata_:title"),
        date: extract("_metadata_:date"),
    }
}

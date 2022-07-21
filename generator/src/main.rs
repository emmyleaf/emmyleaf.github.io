mod types;

use crate::types::{BlogIndex, BlogPost, Page, PageContent};
use anyhow::{Ok, Result};
use const_format::concatcp;
use minify_html::{minify, Cfg};
use pulldown_cmark::{html, LinkDef, Parser};
use ramhorns::Ramhorns;
use std::{
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

fn main() -> Result<()> {
    // clean build directory
    _ = fs::remove_dir_all(BUILD_PATH);
    let ramhorns = Ramhorns::from_folder(TEMPLATE_PATH)?;
    let pages = parse_pages()?;
    let blog_index = process_blog_index(&pages)?;
    generate_pages_html(&ramhorns, &pages)?;
    generate_blog_index_html(&ramhorns, &blog_index)?;
    copy_includes()
}

fn parse_pages() -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    for path in get_file_paths(CONTENT_PATH) {
        let md_content = fs::read_to_string(&path)?;
        let page_content = parse_page(md_content);

        let bare_path = path.strip_prefix(CONTENT_PATH)?;
        let out_path = Path::new(BUILD_PATH).join(bare_path).with_extension("html");

        pages.push(Page { page_content, out_path })
    }

    Ok(pages)
}

fn parse_page(md_content: String) -> PageContent {
    let mut parser = Parser::new_ext(&md_content, pulldown_cmark::Options::all());
    let mut content = String::new();
    html::push_html(&mut content, &mut parser);

    let refdefs = parser.reference_definitions();
    let to_title = |def: &LinkDef| def.title.as_deref().unwrap().to_string();
    let extract = |key: &str| refdefs.get(key).map(to_title).unwrap_or_default();
    PageContent {
        template: extract("_metadata_:template") + ".html",
        title: extract("_metadata_:title"),
        date: extract("_metadata_:date"),
        content,
    }
}

fn process_blog_index(pages: &[Page]) -> Result<BlogIndex> {
    let mut posts = pages
        .iter()
        .filter(|page| page.page_content.template.starts_with("blog_post"))
        .map(|page| BlogPost {
            title: &page.page_content.title,
            date: &page.page_content.date,
            filename: page.out_path.as_path().file_name().unwrap().to_str().unwrap(),
        })
        .collect::<Vec<BlogPost>>();

    // sort by date desc
    posts.sort_by(|a, b| b.date.cmp(a.date));

    Ok(BlogIndex { posts })
}

fn generate_pages_html(ramhorns: &Ramhorns, pages: &[Page]) -> Result<()> {
    let minify_cfg = Cfg { minify_js: true, ..Cfg::spec_compliant() };
    for Page { page_content, out_path } in pages {
        let template = ramhorns.get(&page_content.template).unwrap();
        let html = template.render(&page_content);
        let minified_html = minify(html.as_bytes(), &minify_cfg);

        fs::create_dir_all(out_path.as_path().parent().unwrap())?;
        fs::write(&out_path, minified_html)?;
    }

    Ok(())
}

fn generate_blog_index_html(ramhorns: &Ramhorns, blog_index: &BlogIndex) -> Result<()> {
    let minify_cfg = Cfg { minify_js: true, ..Cfg::spec_compliant() };
    let template = ramhorns.get("blog_index.html").unwrap();
    let html = template.render(blog_index);
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

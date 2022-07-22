mod types;

use anyhow::{Ok, Result};
use const_format::concatcp;
use minify_html::{minify, Cfg};
use pulldown_cmark::{html, LinkDef, Parser};
use ramhorns::Ramhorns;
use std::{
    fs,
    path::{Path, PathBuf},
};
use types::{BlogIndex, BlogPost, Page, PageContent, Project, ProjectIndex};
use walkdir::{DirEntry, WalkDir};

const BUILD_PATH: &str = "../build/";
const SOURCE_PATH: &str = "../site_src/";
const BLOG_PATH: &str = concatcp!(BUILD_PATH, "blog");
const PROJECTS_PATH: &str = concatcp!(BUILD_PATH, "projects");
const CONTENT_PATH: &str = concatcp!(SOURCE_PATH, "content");
const INCLUDE_PATH: &str = concatcp!(SOURCE_PATH, "include");
const TEMPLATE_PATH: &str = concatcp!(SOURCE_PATH, "template");

fn main() -> Result<()> {
    _ = fs::remove_dir_all(BUILD_PATH); // clean build directory

    let ramhorns = Ramhorns::from_folder(TEMPLATE_PATH)?;
    let minify_cfg = Cfg { minify_js: true, ..Cfg::spec_compliant() };
    let pages = parse_pages()?;
    let (blog_index, project_index) = process_indices(&pages)?;

    generate_pages_html(&ramhorns, &minify_cfg, &pages)?;
    generate_blog_index_html(&ramhorns, &minify_cfg, &blog_index)?;
    generate_project_index_html(&ramhorns, &minify_cfg, &project_index)?;
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
    pages.sort_by(|a, b| b.page_content.date.cmp(&a.page_content.date)); // sort by date desc
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
        summary: extract("_metadata_:summary"),
        tags: extract("_metadata_:tags"),
        content,
    }
}

fn process_indices(pages: &Vec<Page>) -> Result<(BlogIndex, ProjectIndex)> {
    let mut posts = Vec::new();
    let mut projects = Vec::new();
    for Page { page_content, out_path } in pages {
        let filename = out_path.as_path().file_name().unwrap().to_str().unwrap();
        match page_content.template.as_str() {
            "blog_post.html" => posts.push(BlogPost {
                title: &page_content.title,
                date: &page_content.date,
                filename,
            }),
            "project.html" => projects.push(Project {
                title: &page_content.title,
                summary: &page_content.summary,
                filename,
            }),
            _ => {}
        }
    }
    Ok((BlogIndex { posts }, ProjectIndex { projects }))
}

fn generate_pages_html(ramhorns: &Ramhorns, minify_cfg: &Cfg, pages: &[Page]) -> Result<()> {
    for Page { page_content, out_path } in pages {
        let template = ramhorns.get(&page_content.template).unwrap();
        let html = template.render(&page_content);
        let minified_html = minify(html.as_bytes(), minify_cfg);
        write_file(out_path, &minified_html)?;
    }
    Ok(())
}

fn generate_blog_index_html(
    ramhorns: &Ramhorns,
    minify_cfg: &Cfg,
    blog_index: &BlogIndex,
) -> Result<()> {
    let template = ramhorns.get("blog_index.html").unwrap();
    let html = template.render(blog_index);
    let minified_html = minify(html.as_bytes(), minify_cfg);
    let out_path = Path::new(BLOG_PATH).join("index.html");
    write_file(&out_path, &minified_html)
}

fn generate_project_index_html(
    ramhorns: &Ramhorns,
    minify_cfg: &Cfg,
    project_index: &ProjectIndex,
) -> Result<()> {
    let template = ramhorns.get("project_index.html").unwrap();
    let html = template.render(project_index);
    let minified_html = minify(html.as_bytes(), minify_cfg);
    let out_path = Path::new(PROJECTS_PATH).join("index.html");
    write_file(&out_path, &minified_html)
}

fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, contents)?;
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

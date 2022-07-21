use ramhorns::Content;
use std::path::PathBuf;

#[derive(Content, Debug)]
pub struct PageContent {
    pub template: String,
    pub title: String,
    pub date: String,
    pub content: String,
}

#[derive(Debug)]
pub struct Page {
    pub page_content: PageContent,
    pub out_path: PathBuf,
}

#[derive(Content, Debug)]
pub struct BlogPost<'a> {
    pub title: &'a str,
    pub date: &'a str,
    pub filename: &'a str,
}

#[derive(Content, Debug)]
pub struct BlogIndex<'a> {
    pub posts: Vec<BlogPost<'a>>,
}

#[derive(Default)]
pub struct MarkdownOptions;

pub fn markdown_to_html(md: &str, _: MarkdownOptions) -> String {
    String::from(md)
}

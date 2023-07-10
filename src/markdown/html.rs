#[derive(Default)]
pub struct MarkdownOptions;

#[allow(dead_code)]
pub fn markdown_to_html(md: &str, _: MarkdownOptions) -> String {
    String::from(md)
}

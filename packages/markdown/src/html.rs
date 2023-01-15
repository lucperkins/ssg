pub struct MarkdownOptions;

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {}
    }
}

pub fn markdown_to_html(md: &str, _: MarkdownOptions) -> String {
    String::from(md)
}

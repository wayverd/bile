use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Write},
};

use comrak::{
    Options, adapters::SyntaxHighlighterAdapter, html::write_opening_tag,
    markdown_to_html_with_plugins, options::Plugins,
};
use syntect::{
    Error,
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

use crate::SYNTAXES;

#[tracing::instrument(skip_all)]
pub fn render(input: &str) -> String {
    let adaptor = SyntectAdapter {
        syntax_set: &SYNTAXES,
    };

    let options = Options::default();

    let mut plugins = Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adaptor);

    markdown_to_html_with_plugins(input, &options, &plugins)
}

struct SyntectAdapter {
    syntax_set: &'static SyntaxSet,
}

impl SyntectAdapter {
    fn highlight_html(&self, code: &str, syntax: &SyntaxReference) -> Result<String, Error> {
        let mut html_generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, self.syntax_set, ClassStyle::Spaced);
        for line in LinesWithEndings::from(code) {
            html_generator.parse_html_for_line_which_includes_newline(line)?;
        }
        Ok(html_generator.finalize())
    }
}

impl SyntaxHighlighterAdapter for SyntectAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let fallback_syntax = "Plain Text";

        let lang: &str = match lang {
            Some(l) if !l.is_empty() => l,
            _ => fallback_syntax,
        };

        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| {
                self.syntax_set
                    .find_syntax_by_first_line(code)
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            });

        match self.highlight_html(code, syntax) {
            Ok(highlighted_code) => output.write_str(&highlighted_code),
            Err(_) => output.write_str(code),
        }
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        let mut attributes: HashMap<&str, &str> = HashMap::new();
        attributes.insert("class", "syntax-highlighting");
        write_opening_tag(output, "pre", attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        write_opening_tag(output, "code", attributes)
    }
}

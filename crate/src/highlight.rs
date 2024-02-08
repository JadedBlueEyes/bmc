use colored::*;
use std::io::Write;

use crate::lexer::Token;
use ariadne::{Label, Report, ReportKind, Source};
use logos::Logos;

pub fn highlight(source: &str, mut w: &mut dyn std::io::Write) {
    let mut lex = Token::lexer(&source).spanned();
    while let Some((token, span)) = lex.next() {
        let _ = match token {
            Ok(Token::LineComment) => write!(&mut w, "{}", source[span].dimmed()),
            Ok(Token::BlockComment) => write!(&mut w, "{}", source[span].dimmed()),
            Ok(Token::Integer) => write!(&mut w, "{}", source[span].blue()),
            Ok(Token::HexInteger) => write!(&mut w, "{}", source[span].blue()),
            Ok(Token::OctalInteger) => write!(&mut w, "{}", source[span].blue()),
            Ok(Token::BinaryInteger) => write!(&mut w, "{}", source[span].blue()),
            Ok(Token::Float) => write!(&mut w, "{}", source[span].blue()),
            Ok(Token::Directive) => write!(&mut w, "{}", source[span].yellow()),
            Ok(Token::Label) => write!(&mut w, "{}", source[span].green()),
            Ok(Token::Identifier) => write!(&mut w, "{}", source[span].cyan()),
            Ok(Token::DotSymbol) => write!(&mut w, "{}", source[span].cyan()),
            Ok(_) => write!(&mut w, "{}", &source[span]),
            Err(_) => {
                // let source_file = file.map(|f| f.as_str()).unwrap_or("input");
                Report::build(ReportKind::Error, (), span.start)
                    .with_message("Unknown token!")
                    .with_label(Label::new(span))
                    .finish()
                    .print(Source::from(&source))
                    .unwrap();
                Ok(())
            }
        };
    }
}

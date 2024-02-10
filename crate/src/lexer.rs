use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
// #[logos(skip r"\p{Default_Ignorable_Code_Point}+")]
#[logos(subpattern decimal = r"[0-9][_0-9]*")]
#[logos(subpattern hex = r"[0-9a-fA-F][_0-9a-fA-F]*")]
#[logos(subpattern octal = r"[0-7][_0-7]*")]
#[logos(subpattern binary = r"[0-1][_0-1]*")]
pub enum Token {
    // Or regular expressions.
    #[regex("//[^\n]*")]
    LineComment,
    #[regex("/\\*(?:[^*]|\\*[^/])*\\*/")]
    BlockComment,
    #[token("\r\n")]
    #[token("\n")]
    NewLine,
    // #[regex(r"\p{Default_Ignorable_Code_Point}")]
    #[regex(r"\p{Pattern_White_Space}")]
    WhiteSpace,

    #[regex("(?&decimal)")]
    Integer,
    #[regex("0[xX](?&hex)")]
    HexInteger,
    #[regex("0[oO](?&octal)")]
    OctalInteger,
    #[regex("0[bB](?&binary)")]
    BinaryInteger,
    #[regex(r#"(?&decimal)(?:e(?&decimal)|\.(?&decimal)(?:e(?&decimal))?)"#)]
    Float,

    #[regex("\\.[_a-zA-Z][_0-9a-zA-Z$]*")]
    Directive,
    #[regex("[_a-zA-Z][_0-9a-zA-Z$]*:")]
    Label,
    #[regex("[_a-zA-Z][_0-9a-zA-Z$]*")]
    Identifier,
    #[token(".")]
    DotSymbol,
}

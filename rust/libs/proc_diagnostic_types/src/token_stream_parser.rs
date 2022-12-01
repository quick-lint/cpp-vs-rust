pub struct TokenStreamParser {
    iterator: proc_macro::token_stream::IntoIter,
    pub current: Option<proc_macro::TokenTree>,
}

impl TokenStreamParser {
    pub fn new(stream: proc_macro::TokenStream) -> TokenStreamParser {
        let mut iterator = stream.into_iter();
        let current = iterator.next();
        TokenStreamParser {
            iterator: iterator,
            current: current,
        }
    }

    pub fn try_parse_punct_token(
        &mut self,
        c: char,
        spacing: proc_macro::Spacing,
    ) -> Option<proc_macro::Punct> {
        match &self.current {
            Some(proc_macro::TokenTree::Punct(punct))
                if punct.as_char() == c && punct.spacing() == spacing =>
            {
                let punct: proc_macro::Punct = punct.clone();
                self.skip();
                Some(punct)
            }
            _ => None,
        }
    }

    pub fn skip_punct_token(&mut self, c: char, spacing: proc_macro::Spacing) {
        if self.try_parse_punct_token(c, spacing).is_none() {
            panic!("expected {}", c);
        }
    }

    pub fn try_parse_comma(&mut self) -> Option<proc_macro::Punct> {
        self.try_parse_punct_token(',', proc_macro::Spacing::Alone)
    }

    pub fn skip_comma(&mut self) {
        if self.try_parse_comma().is_none() {
            panic!("expected comma");
        }
    }

    pub fn try_parse_keyword(&mut self, keyword: &str) -> Option<proc_macro::Ident> {
        match &self.current {
            Some(proc_macro::TokenTree::Ident(ident)) if ident.to_string() == keyword => {
                let ident: proc_macro::Ident = ident.clone();
                self.skip();
                Some(ident)
            }
            _ => None,
        }
    }

    pub fn skip_keyword(&mut self, keyword: &str) {
        if self.try_parse_keyword(keyword).is_none() {
            panic!("expected keyword {}", keyword);
        }
    }

    pub fn skip_punct(&mut self, punct: &str) {
        let chars: Vec<char> = punct.chars().collect();
        for c in &chars[0..chars.len() - 1] {
            if self
                .try_parse_punct_token(*c, proc_macro::Spacing::Joint)
                .is_none()
            {
                panic!("expected '{}'", punct);
            }
        }
        if self
            .try_parse_punct_token(*chars.last().unwrap(), proc_macro::Spacing::Alone)
            .is_none()
        {
            panic!("expected '{}'", punct);
        }
    }

    pub fn skip_lifetime(&mut self) {
        self.skip_punct_token('\'', proc_macro::Spacing::Joint);
        self.try_parse_ident()
            .expect("expected lifetime name after '");
    }

    pub fn try_parse_ident(&mut self) -> Option<proc_macro::Ident> {
        match &self.current {
            Some(proc_macro::TokenTree::Ident(ident)) => {
                let ident = ident.clone();
                self.skip();
                Some(ident)
            }
            _ => None,
        }
    }

    pub fn try_parse_brace(&mut self) -> Option<proc_macro::TokenStream> {
        self.try_parse_group(proc_macro::Delimiter::Brace)
    }

    pub fn try_parse_bracket(&mut self) -> Option<proc_macro::TokenStream> {
        self.try_parse_group(proc_macro::Delimiter::Bracket)
    }

    pub fn try_parse_paren(&mut self) -> Option<proc_macro::TokenStream> {
        self.try_parse_group(proc_macro::Delimiter::Parenthesis)
    }

    pub fn try_parse_group(
        &mut self,
        delimiter: proc_macro::Delimiter,
    ) -> Option<proc_macro::TokenStream> {
        match &self.current {
            Some(proc_macro::TokenTree::Group(group)) if group.delimiter() == delimiter => {
                let children = group.stream();
                self.skip();
                Some(children)
            }
            _ => None,
        }
    }

    pub fn try_parse_string(&mut self) -> Option<String> {
        match &self.current {
            Some(proc_macro::TokenTree::Literal(literal)) => {
                let literal_code: String = literal.to_string();
                if literal_code.starts_with('"') {
                    self.skip();
                    Some(decode_rust_string_literal(&literal_code))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_eof(&mut self) -> bool {
        self.current.is_none()
    }

    pub fn expect_eof(&mut self) {
        if !self.is_eof() {
            panic!("expected end of input");
        }
    }

    fn skip(&mut self) {
        self.current = self.iterator.next();
    }
}

fn decode_rust_string_literal(s: &str) -> String {
    // TODO(port): Implement a proper algorithm.
    let s = &s[1..(s.len() - 1)];
    s.replace("\\\\", "\\")
}

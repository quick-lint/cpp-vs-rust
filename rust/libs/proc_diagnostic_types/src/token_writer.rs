pub struct TokenWriter(Vec<proc_macro::TokenTree>);

impl TokenWriter {
    pub fn new() -> TokenWriter {
        TokenWriter(vec![])
    }

    pub fn token(&mut self, token: proc_macro::TokenTree) {
        self.0.push(token);
    }

    pub fn ident(&mut self, name: &str) {
        self.token(proc_macro::TokenTree::Ident(proc_macro::Ident::new(
            name,
            proc_macro::Span::call_site(),
        )))
    }

    pub fn lifetime(&mut self, name: &str) {
        self.token(proc_macro::TokenTree::Punct(proc_macro::Punct::new(
            '\'',
            proc_macro::Spacing::Joint,
        )));
        self.ident(name);
    }

    pub fn string(&mut self, value: &str) {
        self.0
            .push(proc_macro::TokenTree::Literal(proc_macro::Literal::string(
                value,
            )));
    }

    pub fn punct(&mut self, symbol: &str) {
        let chars: Vec<char> = symbol.chars().collect();
        for c in &chars[0..chars.len() - 1] {
            self.0
                .push(proc_macro::TokenTree::Punct(proc_macro::Punct::new(
                    *c,
                    proc_macro::Spacing::Joint,
                )));
        }
        self.0
            .push(proc_macro::TokenTree::Punct(proc_macro::Punct::new(
                *chars.last().unwrap(),
                proc_macro::Spacing::Alone,
            )));
    }

    pub fn literal_u16(&mut self, value: u16) {
        self.token(proc_macro::TokenTree::Literal(
            proc_macro::Literal::u16_suffixed(value),
        ));
    }

    pub fn literal_usize(&mut self, value: usize) {
        self.token(proc_macro::TokenTree::Literal(
            proc_macro::Literal::usize_suffixed(value),
        ));
    }

    pub fn build_brace<Factory: FnOnce(&mut TokenWriter)>(&mut self, callback: Factory) {
        let mut inner = TokenWriter::new();
        callback(&mut inner);
        self.brace(inner);
    }

    pub fn build_bracket<Factory: FnOnce(&mut TokenWriter)>(&mut self, callback: Factory) {
        let mut inner = TokenWriter::new();
        callback(&mut inner);
        self.bracket(inner);
    }

    pub fn build_paren<Factory: FnOnce(&mut TokenWriter)>(&mut self, callback: Factory) {
        let mut inner = TokenWriter::new();
        callback(&mut inner);
        self.paren(inner);
    }

    pub fn brace(&mut self, inner: TokenWriter) {
        self.group(inner.to_token_stream(), proc_macro::Delimiter::Brace);
    }

    pub fn bracket(&mut self, inner: TokenWriter) {
        self.group(inner.to_token_stream(), proc_macro::Delimiter::Bracket);
    }

    pub fn paren(&mut self, inner: TokenWriter) {
        self.group(inner.to_token_stream(), proc_macro::Delimiter::Parenthesis);
    }

    pub fn empty_paren(&mut self) {
        self.paren(TokenWriter::new());
    }

    pub fn group(&mut self, inner: proc_macro::TokenStream, delimiter: proc_macro::Delimiter) {
        self.0
            .push(proc_macro::TokenTree::Group(proc_macro::Group::new(
                delimiter, inner,
            )));
    }

    pub fn to_token_stream(self) -> proc_macro::TokenStream {
        self.0.into_iter().collect()
    }
}

" Vim script to help convert C++ code into Rust code.

" Google Test:
%s/^TEST\(_F\)\?(\k\+,\_\s*\(\k\+\)) {/#[test]fn \2() {/
%s/EXPECT_EQ/assert_eq!/g
%s/EXPECT_TRUE(\(.*\));/assert!(\1);/g
%s/EXPECT_FALSE(\(.*\));/assert!(!(\1));/g
%s/EXPECT_GT(\([^,]*\), \([^,]*\));/assert!(\1 > \2);/g
%s/EXPECT_GE(\([^,]*\), \([^,]*\));/assert!(\1 >= \2);/g
%s/EXPECT_LT(\([^,]*\), \([^,]*\));/assert!(\1 < \2);/g
%s/EXPECT_LE(\([^,]*\), \([^,]*\));/assert!(\1 <= \2);/g

%s/(!(\(\k\+\(\.\k\+\)*\)))/(!\1)/

" Standard stuff:
%s/\<alignof(\([^)]*\))/std::mem::align_of::<\1>()/g
%s/\<sizeof(\([^)]*\))/std::mem::size_of::<\1>()/g
%s/\<std::size_t\>/usize/g

" quick-lint-js-specific stuff:
%s/linked_bump_allocator<\(.*\)> alloc(/let mut alloc = LinkedBumpAllocator::<\1>::new(/g

" Method syntax:
%s/^\s\+\(\(const \)\?\<\(fn\|if\|while\)\@!\(\k\|:\)\+ [&*]\?\)\(\k\+\)(\(.*\)) const \(noexcept \)\?{/pub fn \5(\&self, \6) -> \1 {/
%s/^\s\+\(\(const \)\?\<\(fn\|if\|while\)\@!\(\k\|:\)\+ [&*]\?\)\(\k\+\)(\(.*\)) \(noexcept \)\?{/pub fn \5(\&mut self, \6) -> \1 {/
%s/\<this->\(\k\+\)_\>/self.\1/g
%s/\<this->/self./g

%s/for (\k\+ i = \(.*\); i < \(.*\); ++i) {/for i in \1..\2 {/

" %s/\<case \(.*\):$/\1 =>/g

%s/\<token_type::kw_\([a-z]\+\)\>/TokenType::KW\u\1/g
%s/\<token_type::\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/TokenType::\u\1\u\2\u\3\u\4/g
%s/\<token_type::\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/TokenType::\u\1\u\2\u\3/g
%s/\<token_type::\([a-z]\+\)_\([a-z]\+\)\>/TokenType::\u\1\u\2/g
%s/\<token_type::\([a-z]\+\)\>/TokenType::\u\1/g

%s/\<u8"\([^"\\]*\)"_sv\>/b"\1"/
%s/\<u8"\([^"\\]*\)"sv\>/b"\1"/
%s/\<u8"\([^"\\]*\)"/b"\1"/

%s/\<self\.check_tokens(\(.*\), {\(.*\)})/f.check_tokens(\1, \&[\2])/
%s/\<self\.check_tokens_with_errors(\_\s*\(.*\),\_\s*{\(.*\)},/f.check_tokens_with_errors(\1, \&[\2],/

%s/\<if (\(.*\)) {$/if \1 {/

%s/\.type\>/.type_/g

%s/\[\](padded_string_view input, const auto& errors) {/|input: PaddedStringView, errors: \&Vec<AnyDiag>| {/

%s/\<self\.diag_reporter->report(/report(self.diag_reporter, /

%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2\u\3\u\4\u\5\u\6\u\7\u\8/g
%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2\u\3\u\4\u\5\u\6\u\7/g
%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2\u\3\u\4\u\5\u\6/g
%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2\u\3\u\4\u\5/g
%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2\u\3\u\4/g
%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2\u\3/g
%s/\C\<diag_\([a-z]\+\)_\([a-z]\+\)\>/Diag\u\1\u\2/g

%s/u8R"(\(.\{-\}\))"/br#"\1"#/

%s/\<padded_string \(input\|code\)(\(b".*"\));/let \1 = PaddedString::from_slice(\2);/
%s/\<lexer l(&\(input\|code\), &null_diag_reporter::instance);/let mut l = Lexer::new(\1, null_diag_reporter());/
%s/\<lexer l(&\(input\|code\), &\(v\|errors\));/let mut l = Lexer::new(\1.view(), \&\2);/
%s/\<diag_collector \(errors\|v\);/let \1 = DiagCollector::new();/

%s/\<SCOPED_TRACE(\(.*\));/scoped_trace!(\1);/

%s/\<QLJS_ASSERT(\(.*\));/qljs_assert!(\1);/
%s/\<QLJS_ALWAYS_ASSERT(\(.*\));/qljs_always_assert!(\1);/
%s/\<QLJS_SLOW_ASSERT(\(.*\));/qljs_slow_assert!(\1);/

%s/EXPECT_THAT(errors.errors, IsEmpty())/qljs_assert_no_diags!(errors.clone_errors(), code.view())/
%s/\<lexer_transaction \(\k\+\) = l.begin_transaction();/let \1: LexerTransaction = l.begin_transaction();/
%s/\<l.\(roll_back_transaction\|commit_transaction\)(std::move(\(\k\+\)));/l.\1(\2);/

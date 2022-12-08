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

%s/\<u8"\([^"]*\)"_sv\>/"\1"/
%s/\<u8"\([^"]*\)"sv\>/"\1"/
%s/\<u8"\([^"]*\)"\>/"\1"/

%s/\<self\.check_tokens(\(.*\), {\(.*\)})/f.check_tokens(\1, \&[\2])/

%s/\<if (\(.*\)) {$/if \1 {/

%s/\.type\>/.type_/g

%s/\[\](padded_string_view input, const auto& errors) {/|input: PaddedStringView, errors: \&Vec<AnyDiag>| {/

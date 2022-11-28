" Vim script to help convert C++ code into Rust code.

" Google Test:
%s/^TEST(\k\+,\_\s*\(\k\+\)) {/#[test]fn \1() {/
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
%s/^\s\+\(\(const \)\?\<\(fn\)\@!\(\k\|:\)\+ [&*]\?\)\(\k\+\)(\(.*\)) const \(noexcept \)\?{/pub fn \5(\&self, \6) -> \1 {/
%s/^\s\+\(\(const \)\?\<\(fn\)\@!\(\k\|:\)\+ [&*]\?\)\(\k\+\)(\(.*\)) \(noexcept \)\?{/pub fn \5(\&mut self, \6) -> \1 {/
%s/\<this->\(\k\+\)_\>/self.\1/g
%s/\<this->/self./g

%s/for (\k\+ i = \(.*\); i < \(.*\); ++i) {/for i in \1..\2 {/

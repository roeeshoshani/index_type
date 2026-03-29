#[test]
fn test_compile_fail_contracts() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/derive_index_too_big_error_non_empty.rs");
    t.compile_fail("tests/ui/derive_index_type_invalid_error_attr.rs");
    t.compile_fail("tests/ui/derive_index_too_big_error_missing_msg.rs");
    t.compile_fail("tests/ui/derive_index_too_big_error_invalid_msg.rs");
}

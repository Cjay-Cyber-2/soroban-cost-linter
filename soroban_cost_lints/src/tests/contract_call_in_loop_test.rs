use crate::lint::contract_call_in_loop::ContractCallInLoop;
use crate::test_utils;

#[test]
fn test_contract_call_in_loop_ui() {
    test_utils::run_ui_test("contract_call_in_loop.rs");
}

#[test]
fn test_contract_call_in_loop_lint_metadata() {
    let lint = ContractCallInLoop;
    assert!(!lint.name().is_empty());
    assert!(!lint.desc().is_empty());
}

use aihack_core::transaction::TurnTransaction;

#[test]
fn transaction_keeps_a_clone_until_commit_transfers_the_working_state() {
    let original = vec![1_u8];
    let mut transaction = TurnTransaction::prepare(&original);
    transaction.working_mut().push(2);

    assert_eq!(original, vec![1]);
    assert_eq!(transaction.working(), &[1, 2]);
    assert_eq!(transaction.commit(), vec![1, 2]);
}

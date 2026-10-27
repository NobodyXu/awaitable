use awaitable::Awaitable;

static AWAITABLE: Awaitable<u32, u32> = Awaitable::const_new();

#[test]
fn const_new_in_static() {
    assert!(!AWAITABLE.is_done());

    AWAITABLE.reset(Some(1));
    assert_eq!(AWAITABLE.take_input().unwrap(), Some(1));

    AWAITABLE.done(2).unwrap();
    assert!(AWAITABLE.is_done());
    assert_eq!(AWAITABLE.take_output(), Some(2));
    assert!(AWAITABLE.is_consumed());
}

use near_sdk::env;

pub fn assert_attached_near() {
    assert!(env::attached_deposit() > 0, "No near attached.")
}

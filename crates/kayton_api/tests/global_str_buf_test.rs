use kayton_api::types::GlobalStrBuf;

#[test]
fn test_global_str_buf_from_string() {
    let test_string = "Hello, World!".to_string();
    let str_buf = GlobalStrBuf::new(test_string.clone());

    assert_eq!(str_buf.len, test_string.len());
    assert_eq!(str_buf.capacity, test_string.capacity());
    assert_eq!(str_buf.as_str(), Some(test_string.as_str()));
}

#[test]
fn test_global_str_buf_from_raw() {
    let test_str = "Hello, World!";
    let str_buf = GlobalStrBuf::from_raw(test_str.as_ptr(), test_str.len(), test_str.len());

    assert_eq!(str_buf.len, test_str.len());
    assert_eq!(str_buf.capacity, test_str.len());
    assert_eq!(str_buf.as_str(), Some(test_str));
}

#[test]
fn test_global_str_buf_null_pointer() {
    let str_buf = GlobalStrBuf::from_raw(std::ptr::null(), 0, 0);

    assert_eq!(str_buf.as_str(), None);
}

#[test]
fn test_global_str_buf_drop() {
    let test_string = "Test string for drop".to_string();
    let str_buf = GlobalStrBuf::new(test_string);

    // The str_buf should be dropped here and the drop function should be called
    // We can't easily test the drop function directly, but we can verify it doesn't panic
    drop(str_buf);
}

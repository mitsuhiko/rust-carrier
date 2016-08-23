#[macro_use]
extern crate carrier;


#[test]
fn test_result_okay() {
    fn foo() -> Result<i32, ()> {
        Ok(42)
    }
    fn bar() -> Result<String, ()> {
        Ok(try!(foo()).to_string())
    }
    assert_eq!(bar(), Ok("42".to_string()));
}

#[test]
fn test_result_fail() {
    use std::convert::From;

    #[derive(Debug, PartialEq)]
    struct MyError;

    impl From<()> for MyError {
        fn from(_err: ()) -> MyError { MyError }
    }

    fn foo() -> Result<i32, ()> {
        Err(())
    }
    fn bar() -> Result<String, MyError> {
        Ok(try!(foo()).to_string())
    }
    assert_eq!(bar(), Err(MyError));
}

#[test]
fn test_option_some() {
    fn foo() -> Option<i32> {
        Some(21)
    }
    fn bar() -> Option<i32> {
        Some(try!(foo()) * 2)
    }
    assert_eq!(bar(), Some(42));
}

#[test]
fn test_option_none() {
    fn foo() -> Option<i32> {
        None
    }
    fn bar() -> Option<i32> {
        Some(try!(foo()) * 2)
    }
    assert_eq!(bar(), None);
}

#[test]
fn test_option_none_type() {
    fn foo() -> Option<i32> {
        None
    }
    fn bar() -> Option<String> {
        Some(try!(foo()).to_string())
    }
    assert_eq!(bar(), None);
}

#[test]
fn test_result_okay_in_iterator_method() {
    use std::convert::From;

    #[derive(Debug, PartialEq)]
    struct MyError;

    impl From<()> for MyError {
        fn from(_err: ()) -> MyError { MyError }
    }

    fn foo() -> Result<i32, ()> {
        Ok(42)
    }
    fn bar() -> Result<String, MyError> {
        Ok(try!(foo()).to_string())
    }
    fn next() -> Option<Result<String, MyError>> {
        let val = try!(bar());
        Some(Ok(format!("[{}]", val)))
    }
    assert_eq!(next(), Some(Ok("[42]".to_string())));
}

#[test]
fn test_result_fail_in_iterator_method() {
    use std::convert::From;

    #[derive(Debug, PartialEq)]
    struct MyError;

    impl From<()> for MyError {
        fn from(_err: ()) -> MyError { MyError }
    }

    fn foo() -> Result<i32, ()> {
        Err(())
    }
    fn bar() -> Result<String, MyError> {
        Ok(try!(foo()).to_string())
    }
    fn next() -> Option<Result<String, MyError>> {
        try!(bar());
        None
    }
    assert_eq!(next(), Some(Err(MyError)));
}

#[test]
fn test_result_raw_ptr() {
    fn foo() -> *const i64 {
        const NUM : i64 = 42;
        &NUM
    }
    fn bar() -> Option<i64> {
        unsafe {
            let x = try!(foo());
            Some(*x)
        }
    }
    assert_eq!(bar(), Some(42));
}

#[test]
fn test_result_raw_ptr_mut() {
    fn foo() -> *mut i64 {
        0 as *mut i64
    }
    fn bar() -> Option<i64> {
        unsafe {
            let x = try!(foo());
            Some(*x)
        }
    }
    assert_eq!(bar(), None);
}

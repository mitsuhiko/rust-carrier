#[macro_use]
extern crate carrier;

use carrier::{IntoCompletion, Completion};


#[test]
fn test_http_example() {
    struct Response {
        status: i32,
    }

    #[derive(PartialEq, Debug)]
    struct Error(i32);

    impl Response {
        fn is_successful(&self) -> bool { self.status == 200 }
        fn status(&self) -> i32 { self.status }
    }

    impl<E, T> IntoCompletion<Result<T, E>> for Response
        where E: From<Error>
    {
        type Value = Response;

        fn into_completion(self) -> Completion<Response, Result<T, E>> {
            if self.is_successful() {
                Completion::Value(self)
            } else {
                Completion::Abrupt(Err(Error(self.status()).into()))
            }
        }
    }

    fn http_test(status: i32) -> Result<(), Error> {
        let resp = try!(Response { status: status });
        Ok(())
    }

    assert_eq!(http_test(404), Err(Error(404)));
    assert_eq!(http_test(200), Ok(()));
}

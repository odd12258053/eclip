use std::env::Args;

pub enum ArgValue {
    Option(String),
    Argument(String)
}

pub trait Validator {
    fn validate(pre: Self, arg: ArgValue, args: &mut Args) -> Self;
}

impl Validator for String {
    fn validate(_pre: Self, arg: ArgValue, args: &mut Args) -> Self {
        match arg {
            ArgValue::Option(_) => args.next().unwrap(),
            ArgValue::Argument(arg) => arg
        }
    }
}

impl Validator for bool {
    fn validate(_pre: Self, arg: ArgValue, _args: &mut Args) -> Self {
        match arg {
            ArgValue::Option(_) => true,
            ArgValue::Argument(arg) => arg.parse().unwrap()
        }
    }
}

macro_rules! validator_for_numeric {
    ( $( $i:ident ),* ) => {
        $(
            impl Validator for $i {
                fn validate(_pre: Self, arg: ArgValue, args: &mut Args) -> Self {
                    match arg {
                        ArgValue::Option(_) => args.next().unwrap(),
                        ArgValue::Argument(arg) => arg
                    }.parse().unwrap()
                }
            }
        )*
    };
}

validator_for_numeric!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64
);


impl<T: Validator + Default> Validator for Option<T> {
    fn validate(pre: Self, arg: ArgValue, args: &mut Args) -> Self {
        match pre {
            Some(pre) => Some(Validator::validate(pre, arg, args)),
            None => Some(Validator::validate(Default::default(), arg, args)),
        }
    }
}

impl<T: Validator + Default> Validator for Vec<T> {
    fn validate(mut pre: Self, arg: ArgValue, args: &mut Args) -> Self {
        pre.push(Validator::validate(Default::default(), arg, args));
        pre
    }
}

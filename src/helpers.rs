pub fn Int32x32To64(a: i32, b: i32) -> i64 { a as i64 * b as i64 }

#[macro_export]
macro_rules! IsTagEnabled {
    ($e: expr) => {
        false
    }
}

#[macro_export]
macro_rules! TraceTag {
    ($e: expr) => {
    }
}

#[macro_export]
macro_rules! IFC {
    ($e: expr) => {
        assert_eq!($e, S_OK);
    }
}

#[macro_export]
macro_rules! __analysis_assume {
    ($e: expr) => {
    }
}


#[macro_export]
macro_rules! IFCOOM {
    ($e: expr) => {
        assert_ne!($e, NULL());
    }
}

#[macro_export]
macro_rules! RRETURN1 {
    ($e: expr, $s1: expr) => {
        if $e == $s1 {
        } else {
            assert_eq!($e, S_OK);
        }
        return $e;
    }
}

#[macro_export]
macro_rules! RRETURN {
    ($e: expr) => {
        assert_eq!($e, S_OK);
        return $e;
    }
}


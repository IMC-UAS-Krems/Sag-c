macro_rules! rc {
    ( $( $x:expr ),* ) => {{
        std::rc::Rc::new([$($x),*])
    }};
}

macro_rules! arc {
    ( $( $x:expr ),* ) => {{
        std::sync::arc::Arc::new([$($x),*])
    }};
}

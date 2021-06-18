#![allow(missing_docs)] // TODO
#![allow(dead_code)] // TODO
#![deny(rust_2018_idioms)]
#![deny(clippy::too_many_arguments)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![forbid(unsafe_code)]
#![warn(clippy::style)]
#![warn(clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_same_arms)]

#[derive(PartialEq, Eq, Clone)]
enum Shape {
    Bottom,
    Any,
    Bool,
    Num,
    Rec,
    Coll,
    Str,
    Null,
    NullBool,
    NullNum,
    NullRec,
    NullColl,
    NullStr,
}

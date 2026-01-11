#![feature(bool_to_result)]
#![allow(dead_code)]
#![feature(if_let_guard)]
#![feature(trait_alias)]
#![allow(unused_variables)]

pub mod backend;
pub mod common;
pub mod scene;

#[derive(Debug)]
pub enum Message {
    Backend(Result<backend::Output, backend::Error>),
    Scene(scene::Message),
    Tick,
}

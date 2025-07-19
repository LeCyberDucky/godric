#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(if_let_guard)]
pub mod backend;
pub mod common;
pub mod scene;

#[derive(Debug)]
pub enum Message {
    Backend(Result<backend::Output, backend::Error>),
    Scene(scene::Message),
}

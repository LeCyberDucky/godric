#![allow(unused_variables)]
#![allow(dead_code)]
pub mod backend;
pub mod common;
pub mod scene;

#[derive(Debug)]
pub enum Message {
    Backend(Result<backend::Output, backend::Error>),
    Scene(scene::Message),
}

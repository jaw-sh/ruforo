#[macro_use]
extern crate lazy_static;

pub mod chat;
pub mod create_user;
mod ffmpeg;
pub mod filesystem;
pub mod forum;
pub mod frontend;
pub mod hub;
pub mod index;
pub mod init;
pub mod login;
pub mod logout;
pub mod member;
pub mod middleware;
mod orm;
pub mod post;
mod s3;
pub mod session;
mod template;
pub mod thread;
mod ugc;
mod user;
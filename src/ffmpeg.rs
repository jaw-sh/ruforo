use ffmpeg_next::codec::id::Id::{AV1, OPUS, VORBIS, VP8, VP9};
use ffmpeg_next::format::context::Input;
use ffmpeg_next::media::Type;
use std::path::Path;

pub fn get_dimensions_from_input(ctx: &Input) -> Option<(u32, u32)> {
    for stream in ctx.streams() {
        let codec = stream.codec();
        if codec.medium() == Type::Video {
            if let Ok(video) = codec.decoder().video() {
                return Some((video.width(), video.height()));
            }
        }
    }
    None
}

pub fn get_extension_from_input(ctx: &Input) -> Option<String> {
    let format = ctx.format();
    log::error!("Name: {:#?}", format.name());
    log::error!("Description: {:#?}", format.description());
    log::error!("Extensions: {:#?}", format.extensions());
    log::error!("MIME Types: {:#?}", format.mime_types());

    for (k, v) in ctx.metadata().iter() {
        log::error!("{}: {}", k, v);
    }
    for stream in ctx.streams() {
        let codec = stream.codec();
        log::error!("\tmedium: {:?}", codec.medium());
        log::error!("\tid: {:?}", codec.id());
    }

    match format.name() {
        "matroska,webm" => {
            log::info!("found mkv/webm");
            let stream_count = ctx.streams().count();
            if stream_count != 2 {
                log::info!("stream_count: {:?}", stream_count);
                return Some("mkv".to_owned());
            }
            let mut streams = ctx.streams();
            let video_id = streams.next().unwrap().codec().id(); // we already counted the streams, unwrapping should be fine
            match video_id {
                VP8 | VP9 | AV1 => {
                    let audio_id = streams.next().unwrap().codec().id();
                    match audio_id {
                        OPUS | VORBIS => {
                            log::info!("validated webm");
                            Some("webm".to_owned())
                        }
                        _ => {
                            log::info!("audio_id: {:?}", audio_id);
                            Some("mkv".to_owned())
                        }
                    }
                }
                _ => {
                    log::info!("video_id: {:?}", video_id);
                    Some("mkv".to_owned())
                }
            }
        }
        _ => {
            log::error!("unhandled format");
            None
        }
    }
}

pub fn open_with_ffmpeg<P: AsRef<Path>>(path: &P) -> Option<Input> {
    match ffmpeg_next::format::input(path) {
        Ok(ctx) => Some(ctx),
        Err(e) => match e {
            ffmpeg_next::Error::InvalidData => {
                log::error!("invalid data {}", e);
                None
            }
            _ => {
                log::error!("unhandled input error: {}", e);
                None
            }
        },
    }
}

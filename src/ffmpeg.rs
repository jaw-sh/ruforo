use ffmpeg_next::codec::id::Id::{AV1, OPUS, VORBIS, VP8, VP9};
use std::path::Path;

pub async fn get_extension_ffmpeg<P: AsRef<Path>>(path: &P) -> Option<String> {
    match ffmpeg_next::format::input(path) {
        Ok(ctx) => {
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
                    log::info!("get_extension_ffmpeg: found mkv/webm");
                    let stream_count = ctx.streams().count();
                    if stream_count != 2 {
                        log::info!("get_extension_ffmpeg: stream_count: {:?}", stream_count);
                        return Some("mkv".to_owned());
                    }
                    let mut streams = ctx.streams();
                    let video_id = streams.next().unwrap().codec().id(); // we already counted the streams, unwrapping should be fine
                    match video_id {
                        VP8 | VP9 | AV1 => {
                            let audio_id = streams.next().unwrap().codec().id();
                            match audio_id {
                                OPUS | VORBIS => {
                                    log::info!("get_extension_ffmpeg: validated webm");
                                    Some("webm".to_owned())
                                }
                                _ => {
                                    log::info!("get_extension_ffmpeg: audio_id: {:?}", audio_id);
                                    Some("mkv".to_owned())
                                }
                            }
                        }
                        _ => {
                            log::info!("get_extension_ffmpeg: video_id: {:?}", video_id);
                            Some("mkv".to_owned())
                        }
                    }
                }
                _ => {
                    log::error!("get_extension_ffmpeg: unhandled format");
                    None
                }
            }
        }
        Err(e) => match e {
            ffmpeg_next::Error::InvalidData => {
                log::error!("get_extension_ffmpeg: invalid data {}", e);
                None
            }
            _ => {
                log::error!("get_extension_ffmpeg: unhandled input error: {}", e);
                None
            }
        },
    }
}

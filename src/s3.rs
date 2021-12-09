use mime::Mime;
use rusoto_core::Region;
use rusoto_core::RusotoError;
use rusoto_s3::{
    ListObjectsV2Error, ListObjectsV2Output, ListObjectsV2Request, PutObjectError, PutObjectOutput,
    PutObjectRequest, S3Client, S3,
};
use std::collections::HashMap;
use std::path::Path;

pub struct S3Bucket {
    s3: S3Client,
    bucket_name: String,
}

/// this is my fancy intelligent extension extractor
pub fn get_extension_guess(filename: &str) -> Option<String> {
    lazy_static! {
        static ref EXT_LOOKUP: HashMap<&'static str, &'static str> = HashMap::from([
            ("aac", "aac"),
            ("apng", "apng"),
            ("avi", "avi"),
            ("avif", "avif"),
            ("bmp", "bmp"),
            ("djvu", "djvu"),
            ("flac", "flac"),
            ("gif", "gif"),
            ("htm", "html"),
            ("html", "html"),
            ("ico", "ico"),
            ("jpeg", "jpeg"),
            ("jpg", "jpeg"),
            ("json", "json"),
            ("ktx", "ktx"),
            ("m4a", "m4a"),
            ("mka", "mka"),
            ("mkv", "mkv"),
            ("mov", "mov"),
            ("mp3", "mp3"),
            ("mp4", "mp4"),
            ("ogg", "ogg"),
            ("ogv", "ogv"),
            ("pdf", "pdf"),
            ("png", "png"),
            ("rm", "rm"),
            ("sh", "sh"),
            ("svg", "svg"),
            ("txt", "txt"),
            ("weba", "weba"),
            ("webm", "webm"),
            ("webp", "webp"),
            ("xml", "xml"),
            ("zip", "zip"),
        ]);
    };
    fn get_extension_guess_return(filename: &str, idx: usize) -> Option<String> {
        Some(filename[idx + 1..].to_ascii_lowercase())
    }
    const MAX_EXT_LEN: usize = 9; // longest extensions I can think of: sha256sum/gitignore

    // get and specially check the top-level extension, we intentionally skip some rules
    let mut begin_idx = match filename.rfind('.') {
        Some(idx) => {
            if idx == 0
                || idx == filename.len()
                || filename.len() - idx > MAX_EXT_LEN + 1 // +1 because we count the '.' here
                || filename[idx + 1..].chars().all(|x| x.is_ascii_alphanumeric()) == false
            {
                return None;
            }

            // we have a list of extensions that we're okay with just accepting
            match EXT_LOOKUP.get(&filename[idx + 1..]) {
                Some(ext) => {
                    log::error!("EXT_LOOKUP: {}", ext);
                    return Some(ext.to_string());
                }
                None => idx,
            }
        }
        None => return None,
    };

    loop {
        let sub_str = &filename[..begin_idx];
        log::error!("sub_str: {}", sub_str);

        // find beginning of next possible extension
        let new_idx = match sub_str.rfind('.') {
            Some(idx) => idx,
            None => return get_extension_guess_return(&filename, begin_idx),
        };

        // check if double period
        if new_idx == begin_idx - 1 {
            log::info!("get_extension_greedy: found double");
            return get_extension_guess_return(&filename, begin_idx);
        }

        // new sub-extension
        let sub_ext = &sub_str[new_idx + 1..];

        // check if too long
        if sub_ext.len() > MAX_EXT_LEN {
            log::info!("get_extension_greedy: too long");
            return get_extension_guess_return(&filename, begin_idx);
        }

        // check if all numbers

        if sub_ext.parse::<u32>().is_ok() {
            log::info!("get_extension_greedy: all numbers");
            return get_extension_guess_return(&filename, begin_idx);
        }

        // check if isn't ASCII
        if sub_ext.chars().all(|x| x.is_ascii_alphanumeric()) == false {
            log::info!("get_extension_greedy: not ASCII");
            return get_extension_guess_return(&filename, begin_idx);
        }

        begin_idx = new_idx;
    }
}

pub async fn get_extension_ffmpeg<P: AsRef<Path>>(path: &P) -> Option<String> {
    match ffmpeg::format::input(path) {
        Ok(ctx) => {
            let format = ctx.format();
            log::error!("Name: {:#?}", format.name());
            log::error!("Description: {:#?}", format.description());
            log::error!("Extensions: {:#?}", format.extensions());
            log::error!("MIME Types: {:#?}", format.mime_types());
            let acodec = ctx.audio_codec();
            if let Some(codec) = acodec {
                log::error!("AudioCodec: {} - {}", codec.name(), codec.description());
            }
            let vcodec = ctx.video_codec();
            if let Some(codec) = vcodec {
                log::error!("VideoCodec: {} - {}", codec.name(), codec.description());
            }
            if let Some(codec) = ctx.data_codec() {
                log::error!("DataCodec: {} - {}", codec.name(), codec.description());
            }
            // for v in ctx.video_codec().iter() {
            //     log::error!("VideoCodec: {} - {}", v.name(), v.description());
            // }
            for (k, v) in ctx.metadata().iter() {
                log::error!("{}: {}", k, v);
            }
            for stream in ctx.streams() {
                let codec = stream.codec();
                log::error!("\tmedium: {:?}", codec.medium());
                log::error!("\tid: {:?}", codec.id());
            }
            Some("Yes".to_owned())
        }
        Err(e) => match e {
            ffmpeg::Error::InvalidData => {
                log::error!("ffmpeg: invalid data {}", e);
                None
            }
            _ => {
                log::error!("ffmpeg: unhandled input error: {}", e);
                None
            }
        },
    }
}

pub fn get_extension(filename: &str, mime: &Mime) -> Option<String> {
    // We check the MIME manually because the mime and mime_guess crates are both inadequate. We
    // are only looking for formats where we can assume it is the only relevant extension.
    // For example we'd never want to add a format like .gz to the hashmaps, we'd rely on _guess for that.
    lazy_static! {
        static ref MIME_LOOKUP: HashMap<&'static str, &'static str> = HashMap::from([
            ("application/json", "json"),
            ("application/pdf", "pdf"),
            ("application/vnd.rn-realmedia", "rm"),
            ("application/x-sh", "sh"),
            ("application/zip", "zip"),
            ("audio/aac", "aac"),
            ("audio/flac", "flac"),
            ("audio/m4a", "m4a"),
            ("audio/mp4", "mp4"),
            ("audio/mpeg", "mp3"),
            ("audio/ogg", "ogg"),
            ("audio/webm", "weba"),
            ("audio/x-matroska", "mka"),
            ("image/apng", "apng"),
            ("image/avif", "avif"),
            ("image/bmp", "bmp"),
            ("image/gif", "gif"),
            ("image/jpeg", "jpeg"),
            ("image/ktx", "ktx"),
            ("image/png", "png"),
            ("image/svg+xml", "svg"),
            ("image/vnd.djvu", "djvu"),
            ("image/webp", "webp"),
            ("image/x-icon", "ico"),
            ("text/html", "html"),
            ("text/plain", "txt"),
            ("text/xml", "xml"),
            ("video/mp4", "mp4"),
            ("video/ogg", "ogv"),
            ("video/quicktime", "mov"),
            ("video/webm", "webm"),
            ("video/x-matroska", "mkv"),
            ("video/x-msvideo", "avi"),
        ]);
    }
    let result = MIME_LOOKUP.get(mime.as_ref().to_ascii_lowercase().as_str());
    match result {
        Some(v) => {
            log::info!("MIME_LOOKUP: Found {}", v);
            Some(v.to_string())
        }
        None => get_extension_guess(filename),
    }

    // Old Method, static hashmap is probably faster than a jump table
    //
    // match mime.type_() {
    //     mime::IMAGE => match mime.subtype().as_str() {
    //         "apng" => Some("apng".to_owned()),
    //         "avif" => Some("avif".to_owned()),
    //         "bmp" => Some("bmp".to_owned()),
    //         "gif" => Some("gif".to_owned()),
    //         "jpeg" => Some("jpeg".to_owned()),
    //         "png" => Some("png".to_owned()),
    //         "svg+xml" => Some("svg".to_owned()),
    //         "webp" => Some("webp".to_owned()),
    //         _ => get_extension_guess(filename),
    //     },
    //     mime::VIDEO => match mime.subtype().as_str() {
    //         "x-msvideo" => Some("avi".to_owned()),
    //         "ogg" => Some("ogv".to_owned()),
    //         "webm" => Some("webm".to_owned()),
    //         "x-matroska" => Some("mkv".to_owned()),
    //         _ => get_extension_guess(filename),
    //     },
    //     mime::AUDIO => match mime.subtype().as_str() {
    //         "m4a" => Some("m4a".to_owned()),
    //         "ogg" => Some("ogg".to_owned()),
    //         "webm" => Some("webm".to_owned()),
    //         "x-matroska" => Some("mka".to_owned()),
    //         _ => get_extension_guess(filename),
    //     },
    //     _ => get_extension_guess(filename),
    // }
}

impl S3Bucket {
    pub fn new(region: Region, bucket_name: &str) -> S3Bucket {
        log::info!("New S3Bucket");

        S3Bucket {
            s3: S3Client::new(region),
            bucket_name: bucket_name.to_owned(),
        }
    }

    pub async fn list_objects_v2(
        &self,
        filename: &str,
    ) -> Result<ListObjectsV2Output, RusotoError<ListObjectsV2Error>> {
        log::info!("S3Bucket: list_objects_v2: {}", filename);

        // dude claims list_objects_v2 is faster than head_object
        // https://www.peterbe.com/plog/fastest-way-to-find-out-if-a-file-exists-in-s3
        let list_request = ListObjectsV2Request {
            bucket: self.bucket_name.to_owned(),
            prefix: Some(filename.to_owned()),
            ..Default::default()
        };

        self.s3.list_objects_v2(list_request).await
    }

    pub async fn put_object(
        &self,
        data: Vec<u8>,
        filename: &str,
    ) -> Result<PutObjectOutput, RusotoError<PutObjectError>> {
        log::info!("S3Bucket: put_object: {}", filename);

        let put_request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: filename.to_owned(),
            body: Some(data.into()),
            ..Default::default()
        };

        self.s3.put_object(put_request).await
    }
}

/// my test bucket, TODO support multiple buckets with configuration stored in the DB
pub fn s3_test_client() -> S3Bucket {
    let my_region = Region::Custom {
        name: "localhost".to_owned(),
        endpoint: "http://localhost:9000".to_owned(),
    };
    S3Bucket::new(my_region, "test0")
}

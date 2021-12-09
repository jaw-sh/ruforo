use rusoto_core::Region;
use rusoto_core::RusotoError;
use rusoto_s3::{
    ListObjectsV2Error, ListObjectsV2Output, ListObjectsV2Request, PutObjectError, PutObjectOutput,
    PutObjectRequest, S3Client, S3,
};

pub struct S3Bucket {
    s3: S3Client,
    bucket_name: String,
}

/// this is my fancy intelligent extension extractor
pub fn get_extension_greedy(filename: &str) -> Option<&str> {
    let mut begin_idx = match filename.rfind('.') {
        Some(idx) => {
            if idx == 0 || idx == filename.len() {
                return None;
            }
            idx
        }
        None => return None,
    };

    loop {
        let sub_str = &filename[..begin_idx];
        log::error!("sub_str: {}", sub_str);

        // find beginning of next possible extension
        let new_idx = match sub_str.rfind('.') {
            Some(idx) => idx,
            None => return Some(&filename[begin_idx + 1..]),
        };

        // check if double period
        if new_idx == begin_idx - 1 {
            log::info!("get_extension_greedy: found double");
            return Some(&filename[begin_idx + 1..]);
        }

        // check if the extension chunk is all numbers
        let sub_ext = &sub_str[new_idx + 1..];
        log::error!("Thing: {}", sub_ext);
        if sub_ext.parse::<u32>().is_ok() {
            log::info!("get_extension_greedy: all numbers");
            return Some(&filename[begin_idx + 1..]);
        }
        begin_idx = new_idx;
    }
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

use rusoto_core::Region;
use rusoto_core::RusotoError;
use rusoto_s3::{
    ListObjectsV2Error, ListObjectsV2Output, ListObjectsV2Request, PutObjectError, PutObjectOutput,
    PutObjectRequest, S3Client, S3,
};

pub struct S3Bucket {
    s3: S3Client,
    bucket_name: String,
    pub pub_url: String,
}

impl S3Bucket {
    pub fn new(region: Region, bucket_name: String, pub_url: String) -> S3Bucket {
        log::info!("Initializing new S3 Bucket.");

        S3Bucket {
            s3: S3Client::new(region),
            bucket_name,
            pub_url,
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

        // we could ensure minimum filename length to be safe, but we'll just panic instead
        let prefix1 = &filename[0..2];
        let prefix2 = &filename[2..4];
        let key = format!("{}/{}/{}", prefix1, prefix2, filename);
        let put_request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key,
            body: Some(data.into()),
            ..Default::default()
        };

        self.s3.put_object(put_request).await
    }
}

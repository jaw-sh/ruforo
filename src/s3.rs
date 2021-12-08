use rusoto_core::Region;
use rusoto_core::RusotoError;
use rusoto_s3::{PutObjectError, PutObjectOutput, PutObjectRequest, S3Client, S3};

pub struct S3Bucket {
    s3: S3Client,
    bucket_name: String,
}

impl S3Bucket {
    pub fn new(region: Region, bucket_name: &str) -> S3Bucket {
        log::info!("New S3Bucket");

        S3Bucket {
            s3: S3Client::new(region),
            bucket_name: bucket_name.to_owned(),
        }
    }

    pub async fn put_object(
        &self,
        data: Vec<u8>,
        filename: &str,
    ) -> Result<PutObjectOutput, RusotoError<PutObjectError>> {
        log::info!("S3Bucket: put_object: {}", filename);

        // let check_for_file // TODO

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

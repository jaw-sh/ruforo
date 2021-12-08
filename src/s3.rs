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
        filename: &str,
    ) -> Result<PutObjectOutput, RusotoError<PutObjectError>> {
        let mut contents: Vec<u8> = Vec::new();
        contents.push(b'a');

        let put_request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: filename.to_owned(),
            body: Some(contents.into()),
            ..Default::default()
        };

        self.s3.put_object(put_request).await
    }
}

pub async fn upload() {
    // Hash an input all at once.
    let hash1 = blake3::hash(b"foobarbaz");

    // Hash an input incrementally.
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"foo");
    hasher.update(b"bar");
    hasher.update(b"baz");
    let hash2 = hasher.finalize();
    assert_eq!(hash1, hash2);

    // Extended output. OutputReader also implements Read and Seek.
    let mut output = [0; 1000];
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut output);
    assert_eq!(&output[..32], hash1.as_bytes());

    // Print a hash as hex.
    println!("{}", hash1);
}

pub async fn s3_test() {
    let my_region = Region::Custom {
        name: "localhost".to_owned(),
        endpoint: "http://localhost:9000".to_owned(),
    };
    let s3 = S3Bucket::new(my_region, "test0");
    s3.put_object("README0.md").await.unwrap();
}

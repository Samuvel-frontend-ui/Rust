use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use std::{fs, io::Write, env};
use uuid::Uuid;

pub async fn save_profile_pic(mut payload: Multipart) -> Option<String> {
    dotenvy::dotenv().ok();
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    fs::create_dir_all(&upload_dir).ok()?;

    let mut file_path = None;

    while let Some(item) = payload.next().await {
        let mut field = item.ok()?;
        let content_disposition = field.content_disposition()?;
        let field_name = content_disposition.get_name().unwrap_or("");

        if field_name == "profile_pic" {
            let filename = format!("{}_{}", Uuid::new_v4(), "profile.jpg");
            let filepath = format!("{}/{}", upload_dir, filename);

            let mut f = fs::File::create(&filepath).ok()?;
            while let Some(chunk) = field.next().await {
                let data = chunk.ok()?;
                f.write_all(&data).ok()?;
            }

            file_path = Some(filepath);
        }
    }

    file_path
}

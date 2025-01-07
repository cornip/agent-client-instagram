use image::GenericImageView;
use agent_client_instagram::AgentClientInstagram;
use agent_client_instagram::types::*;
use std::fs;
use std::env;
use image;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let username = env::var("INSTAGRAM_USERNAME").unwrap();
    let password = env::var("INSTAGRAM_PASSWORD").unwrap();
    let two_factor_key = Some(env::var("INSTAGRAM_TWO_FACTOR_KEY").unwrap());
    let mut client = AgentClientInstagram::new(username, password, two_factor_key);
    client.login().await?;
    let image_data = fs::read("./path/to/your/image.jpg")?;
    let img = image::load_from_memory(&image_data)?;
    let dimensions = img.dimensions();
    let format = image::guess_format(&image_data)?;
    
    let mime_type = match format {
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Jpeg => "image/jpeg",
        _ => return Err("Unsupported image format".into()),
    };

    let image = ImageUpload {
        data: image_data,
        image_type: mime_type.to_string(),
        width: dimensions.0,
        height: dimensions.1,
    };
    let caption = Some("Hello World!".to_string());
    let res = client.create_post(image, caption).await?;
    println!("Post created: {}", res);
    Ok(())
}

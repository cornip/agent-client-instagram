use agent_client_instagram::AgentClientInstagram;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let auth_token = env::var("INSTAGRAM_AUTH_TOKEN").unwrap();
    let client = AgentClientInstagram::new_with_token(auth_token);
    let timeline = client.get_timeline().await?;
    println!("Timeline: {:?}", timeline);
    Ok(())
}

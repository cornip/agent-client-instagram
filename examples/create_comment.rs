use agent_client_instagram::AgentClientInstagram;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let username = env::var("INSTAGRAM_USERNAME").unwrap();
    let password = env::var("INSTAGRAM_PASSWORD").unwrap();
    let two_factor_key = Some(env::var("INSTAGRAM_TWO_FACTOR_KEY").unwrap());
    let mut client = AgentClientInstagram::new(username, password, two_factor_key);
    client.login().await?;
    let res = client.comment_post("3540477562118254725_71436944435", "comment3", None).await?;
    println!("Comment created: {:?}", res);
    Ok(())
}

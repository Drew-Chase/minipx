#[actix_web::main]
async fn main()->anyhow::Result<()>{
	minipx_web_lib::run().await
}

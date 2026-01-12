use crate::db;
use crate::auth;
use tracing;

pub async fn init_db(db_url: &str) -> Result<crate::db::DbPool, Box<dyn std::error::Error>> {
    // 2. DB 초기화
    let pool = db::init_db(db_url).await?;

    // 초기 관리자 계정 생성 (없으면)
    let admin_exists = db::get_user(&pool, "admin").await?.is_some();
    if !admin_exists {
        let hash = auth::hash_password("changeme")?;
        db::create_user(&pool, "admin", &hash).await?;
        tracing::info!("👤 Created default admin user: admin / changeme");
    }

    Ok(pool)
}

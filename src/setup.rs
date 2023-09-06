use sea_orm::*;

// Replace with your database URL and database name
const DATABASE_URL: &str = "mysql://root:ad879037-c7a4-4063-9236-6bfc35d54b7d@139.159.180.129:3306/rustdb";
const DB_NAME: &str = "rustdb";

pub(super) async fn set_up_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

    // let db = match db.get_database_backend() {
    //     DbBackend::MySql => {
    //         db.execute(Statement::from_string(
    //             db.get_database_backend(),
    //             format!("CREATE DATABASE IF NOT EXISTS `{}`;", DB_NAME),
    //         ))
    //             .await?;
    //
    //         let url = format!("{}/{}", DATABASE_URL, DB_NAME);
    //         Database::connect(&url).await?
    //     }
    //     DbBackend::Postgres => {
    //         db.execute(Statement::from_string(
    //             db.get_database_backend(),
    //             format!("DROP DATABASE IF EXISTS \"{}\";", DB_NAME),
    //         ))
    //             .await?;
    //         db.execute(Statement::from_string(
    //             db.get_database_backend(),
    //             format!("CREATE DATABASE \"{}\";", DB_NAME),
    //         ))
    //             .await?;
    //
    //         let url = format!("{}/{}", DATABASE_URL, DB_NAME);
    //         Database::connect(&url).await?
    //     }
    //     DbBackend::Sqlite => db,
    // };

    Ok(db)
}

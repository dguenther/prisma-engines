use migration_core::migration_connector::{ConnectorError, ConnectorResult};
use url::Url;

pub(crate) async fn mongo_setup(schema: &str, url: &str) -> ConnectorResult<()> {
    let url = Url::parse(url).map_err(ConnectorError::url_parse_error).unwrap();
    let db_name = url.path().trim_start_matches('/').to_string();
    let client_options = mongodb::options::ClientOptions::parse(url).await.unwrap();
    let client = mongodb::Client::with_options(client_options).unwrap();

    client
        .database(&db_name)
        .drop(Some(
            mongodb::options::DropDatabaseOptions::builder()
                .write_concern(mongodb::options::WriteConcern::builder().journal(true).build())
                .build(),
        ))
        .await
        .unwrap();

    let ast = datamodel::parse_schema_ast(schema).unwrap();
    let parsed_schema = datamodel::parse_schema_parserdb(schema, &ast).unwrap();

    for model in parsed_schema.db.walk_models() {
        client
            .database(&db_name)
            .create_collection(model.database_name(), None)
            .await
            .unwrap();
    }

    Ok(())
}

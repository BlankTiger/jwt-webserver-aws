use color_eyre::Result;
use data::app::App;
use data::db_actions::DbMockData;
use data::setup::setup;

#[tokio::main]
async fn main() -> Result<()> {
    setup().await?;
    let db_mock_data = DbMockData::new();
    db_mock_data.clear().await?;
    db_mock_data.fill().await?;
    App::new().start_app().await?;
    Ok(())
}

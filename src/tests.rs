use crate::config::Config;
use rocket::fairing::AdHoc;
use rocket::http::uri::fmt::{Query, UriDisplay};
use rocket::http::ContentType;
use rocket::local::blocking::Client;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, UriDisplayQuery)]
#[serde(crate = "rocket::serde")]
struct LoginInfo {
    email: String,
    password: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, UriDisplayQuery)]
#[serde(crate = "rocket::serde")]
struct UpdateMarketNameInfo {
    market_name: String,
}

fn rocket_build(config: Config) -> Rocket<Build> {
    let figment = rocket::Config::figment().merge((
        "databases.squeakroad",
        rocket_db_pools::Config {
            url: config.db_url,
            min_connections: None,
            max_connections: 1024,
            connect_timeout: 3,
            idle_timeout: None,
        },
    ));

    rocket::custom(figment)
}

fn test_admin_settings(_base: &str, stage: AdHoc, config: Config) {
    // NOTE: If we had more than one test running concurently that dispatches
    // DB-accessing requests, we'd need transactions or to serialize all tests.
    let client = Client::tracked(rocket_build(config.clone()).attach(stage)).unwrap();

    // Log in as admin user.
    let admin_login_info = LoginInfo {
        email: config.admin_username,
        password: config.admin_password,
    };
    let login_response = client
        .post("/login")
        .header(ContentType::Form)
        .body((&admin_login_info as &dyn UriDisplay<Query>).to_string())
        .dispatch();
    println!("login_response: {:?}", login_response);

    // Update the market name.
    let update_market_name_info = UpdateMarketNameInfo {
        market_name: "test-market-name".to_string(),
    };
    let update_market_name_response = client
        .post("/update_market_name/change")
        .header(ContentType::Form)
        .body((&update_market_name_info as &dyn UriDisplay<Query>).to_string())
        .dispatch();
    println!(
        "update_market_name_response: {:?}",
        update_market_name_response
    );

    // Get the index page and check the market name.
    let index_page_response = client.get("/").dispatch();
    let index_page_string = index_page_response.into_string().unwrap();
    println!("{}", index_page_string);
    assert!(index_page_string.contains("test-market-name"));
}

#[test]
fn test_routes() {
    let config_figment = Config::get_config().merge(("db_url", "sqlite://:memory:".to_string()));
    let config: Config = config_figment.extract().unwrap();

    test_admin_settings("/", crate::routes::stage(config.clone()), config);
}

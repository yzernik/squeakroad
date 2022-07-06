use crate::config::Config;
use rocket::fairing::AdHoc;
use rocket::http::uri::fmt::{Query, UriDisplay};
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rocket::serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Post {
    title: String,
    text: String,
}

fn test_posts(base: &str, stage: AdHoc) {
    // Number of posts we're going to create/read/delete.
    const N: usize = 20;

    // NOTE: If we had more than one test running concurently that dispatches
    // DB-accessing requests, we'd need transactions or to serialize all tests.
    let client = Client::tracked(rocket::build().attach(stage)).unwrap();

    // Clear everything from the database.
    assert_eq!(client.delete(base).dispatch().status(), Status::Ok);
    assert_eq!(
        client.get(base).dispatch().into_json::<Vec<i64>>(),
        Some(vec![])
    );

    // Add some random posts, ensure they're listable and readable.
    for i in 1..=N {
        let title = format!("My Post - {}", i);
        let text = format!("Once upon a time, at {}'o clock...", i);
        let post = Post {
            title: title.clone(),
            text: text.clone(),
        };

        // Create a new post.
        let response = client.post(base).json(&post).dispatch().into_json::<Post>();
        assert_eq!(response.unwrap(), post);

        // Ensure the index shows one more post.
        let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client.get(format!("{}/{}", base, last)).dispatch();
        assert_eq!(response.into_json::<Post>().unwrap(), post);
    }

    // Now delete all of the posts.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
        let id = list.get(0).expect("have post");

        // Delete that post.
        let response = client.delete(format!("{}/{}", base, id)).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    // Ensure they're all gone.
    let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
    assert!(list.is_empty());

    // Trying to delete should now 404.
    let response = client.delete(format!("{}/{}", base, 1)).dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

fn test_admin_settings(_base: &str, stage: AdHoc, config: Config) {
    // NOTE: If we had more than one test running concurently that dispatches
    // DB-accessing requests, we'd need transactions or to serialize all tests.
    let client = Client::tracked(rocket::build().attach(stage)).unwrap();

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
    let config_figment = Config::get_config();
    let config: Config = config_figment.extract().unwrap();
    test_posts("/posts", crate::routes::stage(config.clone()));
    test_admin_settings("/", crate::routes::stage(config.clone()), config.clone());
}

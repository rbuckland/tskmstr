use serde::{Deserialize, Serialize};

use graph_rs_sdk::oauth::{AccessToken, OAuth};

#[allow(unused_imports)]
use log::{debug, error, info, warn};

pub async fn run(todolist_config: &O365TodoConfiguration) {
    debug!("starting the server...");
    start_server_main(&todolist_config).await;
}

/// ```
use graph_rs_sdk::*;
use warp::Filter;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccessCode {
    code: String,
}

use std::cell::RefCell;

use super::model::O365TodoConfiguration;
thread_local!(static OAUTH_CLIENT: RefCell<Option<OAuth>> = RefCell::new(None));

fn oauth_client_factory(client_id: String, client_secret: String) -> impl Fn() -> OAuth + Clone {
    debug!("creating oauth client");
    move || {
        let mut oauth = OAuth::new();
        oauth
            .client_id(&client_id)
            .client_secret(&client_secret)
            .add_scope("files.read")
            .add_scope("files.readwrite")
            .add_scope("files.read.all")
            .add_scope("files.readwrite.all")
            .add_scope("offline_access")
            .redirect_uri("http://localhost:8998/redirect")
            .authorize_url("https://login.microsoftonline.com/common/oauth2/v2.0/authorize")
            .access_token_url("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .refresh_token_url("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .response_type("code");
        oauth
    }
}

pub async fn set_and_req_access_code(access_code: AccessCode) -> GraphResult<()> {
    debug!("requesting access: {:?}", access_code);
    let mut oauth_client = OAUTH_CLIENT.with(|oauth_cell| {
        oauth_cell
            .borrow_mut()
            .as_ref()
            .unwrap_or_else(|| panic!("oauth has not been initialized yet"))
            .clone()
    });

    // The response type is automatically set to token, and the grant type is automatically
    // set to authorization_code if either of these were not previously set.
    // This is done here as an example.
    oauth_client.access_code(access_code.code.as_str());
    let mut request = oauth_client.build_async().authorization_code_grant();

    // Returns reqwest::Response
    let response = request.access_token().send().await;
    println!("{response:#?}");

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let mut access_token: AccessToken = response.json().await?;
                let jwt = access_token.jwt();
                println!("{jwt:#?}");
                oauth_client.access_token(access_token);
                println!("{:#?}", &oauth_client);
            } else {
                let result: reqwest::Result<serde_json::Value> = response.json().await;
                match result {
                    Ok(body) => println!("{body:#?}"),
                    Err(err) => println!("Error on deserialization:\n{err:#?}"),
                }
            }
        }
        Err(err) => println!("Error sending request: {:#?}", err),
    };

    Ok(())
}

async fn handle_redirect(
    code_option: Option<AccessCode>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    debug!("redirect called");

    match code_option {
        Some(access_code) => {
            // Print out the code for debugging purposes.
            println!("{access_code:#?}");

            // Set the access code and request an access token.
            // Callers should handle the Result from requesting an access token
            // in case of an error here.
            set_and_req_access_code(access_code).await;

            debug!("auth succeeded");

            // Generic login page response.
            Ok(Box::new(
                "Successfully Logged In! You can close your browser.",
            ))
        }
        None => Err(warp::reject()),
    }
}

pub async fn start_server_main(todolist_config: &O365TodoConfiguration) {

    debug!("start_Server_main is runnig: {:?}", todolist_config);


    let query = warp::query::<AccessCode>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<AccessCode>,), std::convert::Infallible>((None,)) });

    let routes = warp::get()
        .and(warp::path("redirect"))
        .and(query)
        .and_then(handle_redirect);

    let mut oauth_x = oauth_client_factory(
        todolist_config.application_client_id.clone(),
        todolist_config.client_secret.clone(),
    )();

    let mut request = oauth_x.build_async().authorization_code_grant();

    OAUTH_CLIENT.with(|oauth_cell| {
        let mut oauth = oauth_cell.borrow_mut();
        oauth.insert(oauth_x);
    });
    request.browser_authorization().open().unwrap();

    warp::serve(routes).run(([127, 0, 0, 1], 8998)).await;
}

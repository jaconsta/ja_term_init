use bpaf::Bpaf;
use get_input::FromTerminal;
use json_print::pretty_print_json;
use page_query::fetch_json_api;
use url_query::HttpClient;
use user_inputs_options::{get_option_input, OptionInputs};
use weather_query::get_the_weather;

mod app_traits {
    use async_trait::async_trait;

    pub trait GetInput {
        fn query_input(&self, question: &str) -> Option<String>;
        fn query_secret(&self, _question: &str) -> Option<String> {
            None
        }
    }

    #[async_trait]
    pub trait QueryClient {
        async fn fetch_text(&self, url: &str, token: &str) -> Option<String>;
    }
}

static FALLBACK_WEATHER: &'static str = "Berlin";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Arguments {
    #[bpaf(long("city"), short('c'), fallback(String::from(FALLBACK_WEATHER)))]
    /// Default city to query the weather
    pub weather_city: String,

    #[bpaf(
        long("bearer"),
        short('b'),
        env("J_BEARER"),
        fallback(String::from(""))
    )]
    /// Token to use on requests
    pub query_token: String,
}

mod get_input {
    use std::io;

    use crate::app_traits::GetInput;

    #[derive(Default)]
    pub struct FromTerminal;
    impl GetInput for FromTerminal {
        fn query_input(&self, question: &str) -> Option<String> {
            println!("{}", question);
            let mut option_input = String::new();

            let did_read = io::stdin().read_line(&mut option_input);
            if let Err(_) = did_read {
                return None;
            }
            Some(option_input)
        }
        fn query_secret(&self, question: &str) -> Option<String> {
            rpassword::prompt_password(question).ok()
        }
    }
}

mod user_inputs_options {
    use colored::*;
    use std::io;

    pub enum OptionInputs {
        GetTemperature,
        PrettyPrintJson,
        GetJsonAPI,
    }

    impl OptionInputs {
        fn named_modules() {
            vec!["Get temperature", "Pretty print json", "GET to an API"]
                .into_iter()
                .enumerate()
                .for_each(|(idx, text)| {
                    println!("{} : {}", format!("{}", idx + 1).cyan(), text);
                });
        }

        fn from_u64(index: u64) -> Option<OptionInputs> {
            match index {
                1 => Some(OptionInputs::GetTemperature),
                2 => Some(OptionInputs::PrettyPrintJson),
                3 => Some(OptionInputs::GetJsonAPI),
                x => {
                    println!("`{}` Is not an options", x);
                    None
                }
            }
        }
    }

    pub fn get_option_input() -> Option<OptionInputs> {
        println!("Select an option to print out messages.");
        OptionInputs::named_modules();

        let mut option_input = String::new();

        let did_read = io::stdin().read_line(&mut option_input);
        if let Err(_) = did_read {
            return None;
        }

        println!(
            // Just an empty space. Left as reference multi-line
            r#"

        "#
        );
        match option_input.trim().parse::<u64>() {
            Ok(x) => OptionInputs::from_u64(x),
            Err(_) => {
                println!("No valid option provided");
                None
            }
        }
    }
}

pub mod json_print {
    use core::panic;

    use crate::app_traits::GetInput;

    pub fn pretty_print_json(user_query: &impl GetInput) {
        let ugly_input = match user_query.query_input("Show me that ugly json.") {
            Some(x) => String::from(x.trim()),
            None => panic!("Error"),
        };
        pretty_json(ugly_input);
    }

    pub fn pretty_json(stringified: String) {
        let ugly_input = serde_json::from_str::<serde_json::Value>(&stringified)
            .expect("I only work with true JSONs.");
        println!("\n{}", serde_json::to_string_pretty(&ugly_input).unwrap());
    }
}

mod url_query {
    use async_trait::async_trait;
    use reqwest::{Client, Url};

    use crate::app_traits::QueryClient;

    pub struct HttpClient {
        client: Client,
    }

    impl HttpClient {
        pub fn new() -> HttpClient {
            HttpClient {
                client: reqwest::Client::new(),
            }
        }
    }
    #[async_trait]
    impl QueryClient for HttpClient {
        async fn fetch_text(&self, url: &str, token: &str) -> Option<String> {
            let url = if let Ok(parsed_url) = Url::parse(&url) {
                parsed_url
            } else {
                panic!("Not a valid url");
            };

            let mut req = self.client.get(url);

            if token != "" {
                println!("token {}.", token);
                req = req.bearer_auth(token);
            }

            let body: String = req
                .send()
                .await
                .ok()
                .expect("failed Loading page")
                .text()
                .await
                .ok()
                .expect("Failed to parse response");

            Some(body)
        }
    }
}

mod weather_query {
    use core::panic;

    use crate::app_traits::QueryClient;

    pub async fn get_the_weather(http_query: &impl QueryClient, city: &str) {
        let (width, _height) = termion::terminal_size().unwrap();
        let url = format!("https://wttr.in/{}", city);
        let body = match http_query.fetch_text(&url, "").await {
            Some(body) => body,
            None => panic!("Could not load weather."),
        };
        let content = html2text::from_read_rich(body.as_bytes(), width as usize);
        for line in content {
            println!("{}", line.into_string()); // Need to handle the removed tags and colors
        }
    }
}

mod page_query {
    use crate::{
        app_traits::{GetInput, QueryClient},
        json_print::pretty_json,
    };

    pub async fn fetch_json_api(
        user_query: &impl GetInput,
        http_query: &impl QueryClient,
        pre_token: &str,
    ) {
        let url = match user_query.query_input("Url: ") {
            Some(u) => u,
            None => panic!("No url"),
        };
        let mut token = String::from(pre_token);
        if pre_token == "" {
            token = match user_query.query_secret("Token (If empty ignored): ") {
                Some(x) => String::from(x.trim()),
                None => "".into(),
            };
        } else {
            println!("Using token from env.")
        }
        let body = match http_query.fetch_text(&url, &token).await {
            Some(body) => body,
            None => panic!("Could not get answer"),
        };
        pretty_json(body);
    }
}

#[tokio::main]
async fn main() {
    let argument_options = arguments().run();
    let opt = match get_option_input() {
        Some(opt) => opt,
        None => std::process::exit(1),
    };

    let terminal_input = FromTerminal {};
    let http_client = HttpClient::new();
    match opt {
        OptionInputs::GetTemperature => {
            get_the_weather(&http_client, &argument_options.weather_city).await
        }
        OptionInputs::PrettyPrintJson => pretty_print_json(&terminal_input),
        OptionInputs::GetJsonAPI => {
            fetch_json_api(&terminal_input, &http_client, &argument_options.query_token).await
        }
    }
}

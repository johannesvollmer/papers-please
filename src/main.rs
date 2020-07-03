use reqwest::{Client};
use std::iter::Iterator;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let document_handles = vec![
        ("@kouhia".to_string(), "10.1016/j.algal.2015.04.001".to_string()),
        ("@zellers".to_string(), "10.18653/v1/P19-1472".to_string())
    ].into_iter();

    let documents = get_documents(document_handles).await;

    for (name, doi, result) in &documents {
        match result {
            Err(error) => eprintln!("could not resolve reference {} ({}): {}", name, doi, error),
            Ok(value) => {
                let values = nom_bibtex::Bibtex::parse(value).expect("invalid bibtex file");
                let values = values.bibliographies();
                println!("found reference {}: \n{:?}\n", name, values);
            }
        }
    }

    Ok(())
}


async fn get_documents(handles: impl Iterator<Item = (String, String)>)
    -> Vec<(String, String, Result<String, reqwest::Error>)>
{
    let client = &Client::new();

    let requests = handles.map(|(name, doi)| {
        async move {
            let url = "http://dx.doi.org/".to_string() + &doi; // TODO escape string
            match client.get(url.as_str()).header("accept", "application/x-bibtex").send().await {
                Ok(result) => (name, doi, result.text().await),
                Err(err) => (name, doi, Err(err))
            }
        }
    });

    futures::future::join_all(requests).await
}

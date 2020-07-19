use reqwest::{Client, Url};
use std::iter::Iterator;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::time::Instant;
use futures::join;
use std::string::ParseError;
use std::num::ParseIntError;

mod parse;



struct Paper {
    definitions: HashMap<String, Definition>,
    contents: Vec<Hierarchy>
}

struct Hierarchy {
    name: String,
    content: String,
    parts: Vec<Hierarchy>
}

enum Definition {
    Publication(Publication),
    Figure(Figure),
    Section(SectionId)
}

struct Figure {
    path: PathBuf,
    description: String,
}

#[derive(Default, Debug)]
struct Publication {
    doi: Option<String>,
    isbn: Option<String>,

    title: Option<String>,
    authors: Option<Vec<Author>>,
    publisher: Option<String>,
    address: Option<String>,
    volume: Option<String>,
    book_title: Option<String>,
    journal: Option<String>,
    editor: Option<String>,
    edition: Option<String>,
    page: Option<String>,
    url: Option<String>,
    year: Option<String>,
}

#[derive(Default, Debug)]
struct SectionId {
    chapter_name: String,
    sub_chapter: Option<Box<SectionId>>,
}

type Author = Vec<String>;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut base_values = vec![
        ("@algae".to_string(), Publication {
            doi: Some("10.1016/j.algal.2015.04.001".to_string()),
            .. Publication::default()
        }),
        ("@hellaswag".to_string(), Publication {
            doi: Some("10.18653/v1/P19-1472".to_string()),
            .. Publication::default()
        }),
        ("@3d".to_string(), Publication {
            isbn: Some("9780201758672".to_string()),
            .. Publication::default()
        }),
        ("@pete".to_string(), Publication {
            .. Publication::default()
        }),
    ];

    dbg!(&base_values);
    let client = Client::new();

    futures::future::join_all(
        base_values.iter_mut()
            .map(|(id, publication)| extend(&client, publication))
    ).await;

    dbg!(&base_values);

    let problems = futures::future::join_all(
        base_values.iter()
            .map(|(id, publication)| find_problems(&client, id, publication))
    ).await;

    dbg!(problems);

    Ok(())
}




async fn extend(client: &Client, publication: &mut Publication) {
    let mut requests = Vec::new();

    if let Some(doi) = publication.doi.as_ref() {
        let url = "http://dx.doi.org/".to_string() + &doi; // TODO escape string
        requests.push(client.get(url.as_str()).header("accept", "application/x-bibtex").send());
    }

    if let Some(isbn) = publication.isbn.as_ref() {
        let url = format!("http://www.ottobib.com/isbn/{}/bibtex", isbn); // TODO escape string
        requests.push(client.get(url.as_str()).send());
    }

    for request in futures::future::join_all(requests).await {
        let response = request.and_then(|r| r.error_for_status());

        if let Ok(response) = response {
            let response = response.text().await;

            if let Ok(response) = response {
                let values = nom_bibtex::Bibtex::parse(response.as_str()).expect("invalid bibtex file");

                for bibliographies in values.bibliographies() {
                    let kind = bibliographies.entry_type();
                    for (name, value) in bibliographies.tags() {
                        match name.to_lowercase().as_str() {
                            "title" => { publication.title.get_or_insert(value.clone()); }, // TODO apply { } transformations
                            "publisher" => { publication.publisher.get_or_insert(value.clone()); },
                            "url" => { publication.url.get_or_insert(value.clone()); },
                            "address" => { publication.address.get_or_insert(value.clone()); },
                            "year" => { publication.year.get_or_insert(value.clone()); },
                            "volume" => { publication.volume.get_or_insert(value.clone()); },
                            "pages" => { publication.page.get_or_insert(value.clone()); },
                            "booktitle" => { publication.book_title.get_or_insert(value.clone()); },
                            "journal" => { publication.journal.get_or_insert(value.clone()); },
                            "author" => { publication.authors.get_or_insert(
                                value.split("and")
                                    .map(|author| author.split_whitespace().map(|name| name.to_string()).collect())
                                    .collect()
                            ); },

                            "doi" | "isbn" | "month" => {}
                            key => { println!("unknown bibtex key: {}", key); }
                        }
                    }
                }

            }
            else { println!("invalid result"); }
        }
        else {
            println!("cannot fetch");
        };
    }
}

async fn find_problems(client: &Client, identifier: &str, publication: &Publication) -> Vec<String> {
    let mut problems = Vec::new();

    if let Some(year) = &publication.year {
        if year.parse::<u64>().is_err() { problems.push(format!("´year {}´ is not a positive whole number ({})", year, identifier)); };
    };

    if let Some(url) = &publication.url {
        if client.get(url.as_str()).send().await.and_then(|r| r.error_for_status()).is_err() {
            problems.push(format!("cannot get contents of ´url {}´ ({})", url, identifier));
        };
    };

    if publication.title.is_none() {
        problems.push(format!("publication {} has no title", identifier));
    }

    if publication.authors.is_none() {
        problems.push(format!("publication {} has no author", identifier));
    }

    if publication.year.is_none() {
        problems.push(format!("publication {} has no year", identifier));
    }

    problems
}
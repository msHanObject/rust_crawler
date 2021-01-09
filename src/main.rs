use std::io::Read;
use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::fs;
use reqwest::Url;
use std::path::Path;
use std::time::Instant;

fn has_extension(url: &&str) -> bool {
    Path::new(url).extension().is_none()
}

fn get_links_from_html(html: &str) -> HashSet<String> {
    Document::from(html)
//        .find(Name("a").or(Name("link")))
        .find(Name("a"))
        .filter_map(|a| a.attr("href"))
        .filter(has_extension)
        .filter_map(normalize_url)
        .collect::<HashSet<String>>()
}

fn normalize_url(url: &str) -> Option<String> {
    let new_url = Url::parse(url);
    match new_url {
        Ok(new_url) => {
            if new_url.has_host() && new_url.host_str().unwrap() == "ghost.rolisz.ro" {
                Some(url.to_string())
            } else {
                None
            }
        },
        Err(_e) => {
            // Relative urls are not parsed by Reqwest
            if url.starts_with('/') {
                Some(format!("https://rolisz.ro{}", url))
            } else {
                None
            }
        }
    }
}

fn fetch_url(client: &reqwest::blocking::Client, url: &str) -> String {
    let mut res = client.get(url).send().unwrap();
    println!("Status for {}: {}", url, res.status());

    let mut body  = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

fn write_file(path: &str, content: &str) {
    fs::create_dir_all(format!("static{}", path)).unwrap();
    fs::write(format!("static{}/index.html", path), content);
}

fn main() {
    let now = Instant::now();

    let client = reqwest::blocking::Client::new();
    let origin_url = "https://rolisz.ro/";

    let body = fetch_url(&client, origin_url);

    write_file("", &body);
    let mut visited = HashSet::new();
    visited.insert(origin_url.to_string());
    let found_urls = get_links_from_html(&body);
    let mut new_urls = found_urls
        .difference(&visited)
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

//    while !new_urls.is_empty() {
    while new_urls.len() > 0 {
        let found_urls: HashSet<String> = new_urls
            .iter()
            .map(|url| {
            let body = fetch_url(&client, url);
            write_file(&url[origin_url.len() - 1..], &body);
            let links = get_links_from_html(&body);
            println!("Visited: {} found {} links", url, links.len());
            links
        })
        .fold(HashSet::new(), |mut acc, x| {
            acc.extend(x);
            acc
        });
        visited.extend(new_urls);
        new_urls = found_urls
            .difference(&visited)
            .map(|x| x.to_string())
            .collect::<HashSet<String>>();
        println!("New urls: {}", new_urls.len())
    }
    println!("URLs found: {:#?}", found_urls);
    println!("{}", now.elapsed().as_secs());
}

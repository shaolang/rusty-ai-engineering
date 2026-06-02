pub async fn read_webpage(url: impl AsRef<str>) -> String {
    let client = reqwest::ClientBuilder::new()
        .user_agent(reqwest::header::USER_AGENT)
        .build()
        .expect("http client created");
    let resp = client.get(url.as_ref()).send().await.expect("page read");
    let doc = scraper::Html::parse_document(resp.text().await.unwrap_or("".to_string()).as_str());
    doc.root_element().text().collect::<Vec<&str>>().join(" ")
}

pub async fn search_web(query: impl AsRef<str>) -> Vec<String> {
    use duckduckgo::{browser::Browser, user_agents::get};

    let browser = Browser::new();
    let user_agent = get("firefox").unwrap();
    let results = browser
        .lite_search(query.as_ref(), "wt-wt", Some(5), user_agent)
        .await
        .expect("search succeeded");

    results.iter().map(|r| r.url.clone()).collect()
}

pub fn create_audio(script: impl AsRef<str>) -> String {
    use any_tts::{ModelType, SynthesisRequest, TtsConfig, load_model};

    let model = load_model(TtsConfig::new(ModelType::Kokoro)).expect("TTS model loaded");
    let audio = model
        .synthesize(
            &SynthesisRequest::new(script.as_ref())
                .with_language("en-us")
                .with_voice("af_heart"),
        )
        .expect("audio created");

    let fname = "podcast.wav";
    audio.save_wav(fname).expect("podcast written to disk");

    fname.to_string()
}

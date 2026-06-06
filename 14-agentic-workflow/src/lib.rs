use helpers::{Args, extract_texts};
use openai_oxide::{OpenAI, types::responses::ResponseCreateRequest};
use podcast_agent::{create_audio, read_webpage, search_web};

pub async fn initiate_podcast(podcast_description: String, client: &OpenAI, args: &Args) -> String {
    let mut workflow = PodcastWorkFlow::new(client.clone(), args.clone(), podcast_description);
    let mut is_sufficient = false;

    while !is_sufficient {
        let web_query = workflow.generate_web_query().await;
        println!("generated web query: {web_query}");
        let urls = search_web(web_query).await;
        println!("urls:\n- {}", urls.join("\n- "));
        workflow.extract_text_from_webpage(urls).await;
        is_sufficient = workflow.has_sufficient_research().await;
    }

    println!("writing podcast script");
    let podcast_script = workflow.write_podcast_script().await;
    println!("creating podcast audio");
    create_audio(podcast_script).await
}

struct PodcastWorkFlow {
    client: OpenAI,
    args: Args,
    podcast_description: String,
    previous_web_queries: Vec<String>,
    research: Vec<String>,
}

impl PodcastWorkFlow {
    pub fn new(client: OpenAI, args: Args, podcast_description: String) -> Self {
        let previous_web_queries = Vec::new();
        let research = Vec::new();
        Self {
            client,
            args,
            podcast_description,
            previous_web_queries,
            research,
        }
    }

    async fn generate_web_query(&mut self) -> String {
        let input = format!(
            "You are doing research for a podcast, whose details are:
             <details>{}</details>. Generate a web search query to help do research
             for this podcast.

             <instructions>
             Ensure that your web search query is different than any of the past queries made:
             <old-query-list>{:#?}</old-query-list>. The web search query should be specific and
             succinct, no more than 6 words. For example, if the podcast is about the latest news
             in Iceland, your query would be simply: Iceland news
             </instructions>
             Generate the web query now.",
            self.podcast_description, self.previous_web_queries,
        );
        let req = ResponseCreateRequest::new(&self.args.model)
            .temperature(self.args.temperature)
            .input(input);
        let response = self
            .client
            .responses()
            .create(req)
            .await
            .expect("responses request completed");
        let query = response.output_text();
        self.previous_web_queries.push(query.clone());

        query
    }

    async fn extract_text_from_webpage(&mut self, urls: Vec<String>) {
        for url in urls {
            let webpage_text = read_webpage(&url).await;
            let input =
                format!(
                "You are doing research for a podcast, whose details are: <details>{}</details>.

                 Extract whatever relevant information you can from this info you've found on
                 the web: <webpage>{}</webpage>. Just include the extracted text and nothing else in
                 your response. Extract the text now.", self.podcast_description, webpage_text);
            let req = ResponseCreateRequest::new(&self.args.model)
                .temperature(self.args.temperature)
                .input(input);
            let response = self
                .client
                .responses()
                .create(req)
                .await
                .expect("response request completed");
            let text = extract_texts(&response.output, false);
            self.research.push(text);
        }
    }

    async fn has_sufficient_research(&self) -> bool {
        let input = format!(
            "You are doing research for a podcast, whose details are: <details>{}</details>. Here is the
             research you have from the web so fat: <research>{:#?}</research>. Do you feel that you
             feel that you have enough info to create a fact-based podcast based on this research with
             the information you have so far? Keep in mind the desired length of the podcast. Respond
             with either: True/False", self.podcast_description, self.research);
        let req = ResponseCreateRequest::new(&self.args.model)
            .temperature(self.args.temperature)
            .input(input);
        let response = self
            .client
            .responses()
            .create(req)
            .await
            .expect("responses request completed");

        response.output_text().trim().to_lowercase() == "true"
    }

    async fn write_podcast_script(&self) -> String {
        let input = format!(
            "You are a podcast scriptwriter, creating scripts for news-based and explainer podcasts.
             The podcast should be based on real facts and web research. As such, do not create any
             fictional information for the podcast. Only use what you find based on your web research.

             Here are the details of what the podcast should be: <details<{}</details>. Here is the
             research you should use to produce the script: <research>{:#?}</research>.

             The script is read by a single host in a news-like style. Do not create music or the like.
             Only create the words to be spoken by the host. Create the script now.",
             self.podcast_description, self.research);
        let req = ResponseCreateRequest::new(&self.args.model)
            .temperature(self.args.temperature)
            .input(input);
        let response = self
            .client
            .responses()
            .create(req)
            .await
            .expect("resonse request completed");

        response.output_text()
    }
}

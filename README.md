# Rusty Common Sense Guide to AI Engineering

Rust implementations of the book [A Common Sense Guide to AI Engineering][book] published by
Pragmatic Programmers.

Most ports expect the environment variable `OPENAI_API_KEY` set and support the follow
CLI arguments:
- `--model`: use the specified the model for the demo
- `--base-url`: connect to model at the specified url; defaults to `https://api.openai.com/v1`
- `--temperature`: set the model's temperature; defaults to 0.0
- `--help`:  show the arguments available

Ports that differ from the above are:
- [token-count](./06-token-count/src/main.rs) from chapter 6

As this repo uses Cargo workspaces to organize the demonstrations, each demo's source directory
has the book's chapter as its prefix, e.g., `01-hello-world` is the demo from chapter 1. However,
when running the demo, drop the chapter prefix from the package `-p` argument (as shown below):

```shell
OPENAI_API_KEY=<api-key> cargo run -p hello-world -- ...
```

## Ports by Chapter

- Chapter 1: HeLLMo, World!
  - [Creating Our First App](./01-hello-world/src/main.rs)
- Chapter 5: Building a Chatbot
  - Augmenting the Prompt
    - [Pirate tone](./05-augment-prompt-pirate-tone/src/main.rs)
    - [Spanish translation](./05-augment-prompt-spanish-translation/src/main.rs)
    - [Grammar checker](./05-augment-prompt-grammar-checker/src/main.rs)
  - [Adding Multi-Turn Dialogue](./05-multi-turn-dialog/src/main.rs)
  - [Managing State With Memory Systems](./05-memory-systems/src/main.rs)
  - [Adding a System Prompt](./05-system-prompt/src/main.rs)
  - [Building the Messages Array](./05-message-array/src/main.rs)
    - This demo uses the [helpers::History](./helpers/src/openai.rs) struct to simplify the
      message collection; as of openai-oxide version 0.14, when
      [openai_oxide::resources::responses::create][resp-create] receives the
      `ResponseCreateRequest` with input type `Vec<ResponseInputItem>`,
      [openai_oxide::resources::responses::Response::output_text][resp-output-text]
      method will always return an empty string. The helper function
      [helper::extract_texts][extract-texts] extracts all texts and return a string.
- Chapter 6: Augmenting a Prompt with Knowledge
  - [token-count](./06-token-count/src/main.rs): does not require an OpenAI client connection
  - [prepare-data](./06-prepare-data/src/main.rs): does not require an OpenAI client connection
  - [knowledge-chatbot](./06-knowledge-chatbot/src/main.rs)
- Chapter 7: Efficient Adding Knowledge with RAG
  - [rag-chatbot](./07-rag-chatbot/src/main.rs)
- Chapter 8: Measuring Quality with Evals
  - eval-chatbot: [main.rs](./08-eval-chatbot/src/main.rs) houses the interaction loop
    and [lib.rs](./08-eval-chatbot/src/lib.rs) contains the functions extracted for reuse
    in trace-generator and rewrite-history demos
  - [trace-generator](./08-trace-generator/src/main.rs)
- Chapter 9: Prompt Engineering
  - [rewrite-history](./09-rewrite-history/src/main.rs)

### Notable differences
Other than the port in chapter 1, all other ports use commonly used functionalities
in [helpers crate](./helpers/src/lib.rs). Chapter 1's port is deliberately left as-is
to show that the full implementation in Rust isn't that complicated, as compared to Python's.
Using `helpers` crate in the rest of the ports minimize distractions from implementation details.

Instead of Pinecone, this repo uses [sqlite-vec][sqlite-vec] and [fastembed-rs][fastembed-rs]
as its vector database. This repo originally used [embedded LanceDB][lancedb] as its vector database
but replaces it with sqlite-vec because the former bloats the binaries rather significantly and
requires protoc toolchain for compilation; sqlite-vec is lighter comparatively and meets the need
of this repo.

### Resources
For convenience, all the files in `resources` directory are copied from the
[book's source code][book]; the copyright of those files belong to the book's author and
the publisher.

[book]: https://pragprog.com/titles/jwpaieng/a-common-sense-guide-to-ai-engineering/
[extract-texts]: ./helpers/src/openai.rs
[fastembed-rs]: https://github.com/Anush008/fastembed-rs
[lancedb]: https://docs.lancedb.com/#1-lancedb-oss
[resp-create]: https://docs.rs/openai-oxide/0.14.0/openai_oxide/resources/responses/struct.Responses.html#method.create
[resp-output-text]: https://docs.rs/openai-oxide/0.14.0/openai_oxide/types/responses/struct.Response.html#method.output_text
[sqlite-vec]: https://github.com/asg017/sqlite-vec

# Rusty Common Sense Guide to AI Engineering

Rust implementations of the book [A Common Sense Guide to AI Engineering][book] published by
Pragmatic Programmers.

All examples expect the `OPENAI_API_KEY` to be exported as an environment variable, e.g.,

```bash
$ OPENAI_API_KEY=<api-key> cargo run -p hello-world -- ...
```

As this repo uses Cargo workspace to organize the demonstrates, each demo's source directory
name has the book's chapter prefix, e.g., `01-hello-world` is the demo from chapter 1.
However, when running the demo, drop the chapter prefix when the package `-p` (as shown above).

## Ports by Chapter

* Chapter 1: HeLLMo, World!
  * [Creating Our First App](./01-hello-world/src/main.rs)
* Chapter 5: Building a Chatbot
  * Augmenting the Prompt
    * [Pirate tone](./05-augment-prompt-pirates/src/main.rs)
    * [Spanish translation](./05-augment-prompt-spanish-translation/src/main.rs)
    * [Grammar checker](./05-augment-prompt-grammar-checker/src/main.rs)
  * [Multi-turn dialogue](./05-multi-turn-dialogue/src/main.rs)
  * [Managing state with memory systems](./05-state-with-memory-systems/src/main.rs)
  * [Treating the prompt as an array](./05-prompt-as-array/src/main.rs)
* Chapter 6: Augmenting a Prompt with Knowledge
  * [Preparing the data](./06-prep-data/src/main.rs)
  * [Implementing the knowledge chatbot](./06-knowledge-chatbot/src/main.rs)
* Chapter 7: Efficiently Adding Knowledge with RAG
  * [Implementing a RAG chatbot](./07-rag-chatbot/src/main.rs)
* Chapter 8: Measuring Quality with Evals
  * [Setting Up Our App][08-eval-chatbot]
  * [Generating Traces](./08-generating-traces/src/main.rs): reuses implementation from
    [Setting Up Our App][eval-chatbot] for simplification
* Chapter 10: Reducing Hallucinations
  * [Reducing Hallucinations][10-reduce_hallucinations]: reuses implementation from
    [Setting Up Our App][08-eval-chatbot] for simplification
* Chapter 11: Evaluating and Optimizing RAG
  * [Expanding the Query](./11-expand-query/src/main.rs): reuses implementation from
    [Reducing Hallucinations][10-reduce_hallucinations] for simplication


[08-eval-chatbot]: ./08-eval-chatbot/src/main.rs
[10-reduce-hallucinations]: ./10-reduce-hallucinations/src/lib.rs

### Notable Differences
Other than the language and the libraries used, this port:
- Uses embedded LanceDB instead of Pinecone for simplicity, i.e., no need to sign up Pinecone;
  also uses [fastembed][fastembed] for text embedding.
- Supports interfacing with local LLM servers, such as [LM Studio][lm-studio] and [Ollama][ollama].

### Wheel-equivalent (almost) Crates
The following table shows the crates used in place of the wheels:

| Wheel   | Crate                           | Remarks
|---------|---------------------------------|---------------------------------------------------------------------
| Docling | [Transmutation][transmutation]  | Augmented with [reqwest][reqwest] to retrieve resources from the web
| openai  | [async-openai][async-openai]
| -       | [lancedb][lancedb]              | Replaced Pinecone as vector database
| -       | [fastembed][fastembed]          | Generate text embeddings when populating vector database

As LanceDB uses ProtoBuf, you may need to install protobuf using Homebrew:

```bash
brew install protobuf
```

Or using asdf-vm:

```bash
asdf plugin add protoc https://github.com/paxosglobal/asdf-protoc.git
```

### Resources
For convenience, all the files in `resources` directory are copied from the
[book's source code][book]; the copyright of those files belong to the book's author and
the publisher.

[async-openai]: https://github.com/64bit/async-openai
[book]: https://pragprog.com/titles/jwpaieng/a-common-sense-guide-to-ai-engineering/
[fastembed]: https://github.com/Anush008/fastembed-rs
[lancedb]: https://github.com/lancedb/lancedb
[lm-studio]: https://lmstudio.ai/
[ollama]: https://ollama.com/
[reqwest]: https://github.com/seanmonstar/reqwest
[transmutation]: https://github.com/hivellm/transmutation

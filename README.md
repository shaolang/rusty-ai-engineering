# Rusty Common Sense Guide to AI Engineering

Rust implementations of the book [A Common Sense Guide to AI Engineering][book] published by
Pragmatic Programmers.

[book]: https://pragprog.com/titles/jwpaieng/a-common-sense-guide-to-ai-engineering/

All examples expect the `OPENAI_API_KEY` to be exported as an environment variable, e.g.,

```bash
$ OPENAI_API_KEY=<api-key> cargo run -p hello-world -- ...
```

As this repo uses Cargo workspace to organize the demonstrates, each demo's source directory
name has the book's chapter prefix, e.g., `01-hello-world` is the demo from chapter 1.
However, when running the demo, drop the chapter prefix when the package `-p` (as shown above).


## Ports by Chapter

* Chapter 1: HeLLMo, World!
  * [Creating Our First App](./01-hello-world)
* Chapter 5: Building a Chatbot
  * Augmenting the Prompt
    * [Pirate tone](./05-augment-prompt-pirates)
    * [Spanish translation](./05-augment-prompt-spanish-translation)

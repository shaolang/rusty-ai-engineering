# Rusty Common Sense Guide to AI Engineering

Rust implementations of the book [A Common Sense Guide to AI Engineering][book] published by
Pragmatic Programmers.

All ports expect the environment variable `OPENAI_API_KEY` set. The ports support the follow
CLI arguments:
- `--model`: use the specified the model for the demo
- `--base-url`: connect to model at the specified url; defaults to `https://api.openai.com/v1`
- `--temperature`: set the model's temperature; defaults to 0.0
- `--help`:  show the arguments available

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

### Notable differences
Other than the port in chapter 1, all other ports use commonly used functionalities
in [helpers crate](./helpers/src/lib.rs). Chapter 1's port is deliberately left as-is
to show that the full implementation in Rust isn't that complicated, as compared to Python's.
Using `helpers` crate in the rest of the ports minimize distractions from implementation details.

### Resources
For convenience, all the files in `resources` directory are copied from the
[book's source code][book]; the copyright of those files belong to the book's author and
the publisher.

[book]: https://pragprog.com/titles/jwpaieng/a-common-sense-guide-to-ai-engineering/

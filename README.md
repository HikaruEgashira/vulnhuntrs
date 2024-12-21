<div align="center">

  <img width="250" src="./logo.png" alt="Vulnhuntrs Logo">

A tool to identify remotely exploitable vulnerabilities using LLMs and static code analysis.

**Autonomous AI-discovered 0day vulnerabilities**

</div>

Vulnhuntrs is a security analysis tool designed to detect vulnerabilities in applications. It provides static analysis capabilities to identify potential security issues in your codebase.

## Features

- Static code analysis for security vulnerabilities
- Support for multiple programming languages
- Detailed vulnerability reports
- Example vulnerable applications for testing

![analyze-python](./analyze-python.png)

## Installation

```bash
# Clone the repository
git clone https://github.com/HikaruEgashira/vulnhuntrs.git

# Build the project
cargo build --release
```

## Usage

```bash
vulnhuntrs -r <path-to-project>
```

## Example Applications

The repository includes example vulnerable applications to demonstrate the tool's capabilities:

- Python app(`example/python-vulnerable-app`)+ Claude 3.5 Sonnet 20241022
```bash
export ANTHROPIC_API_KEY=your-api-key
cargo run -- -r example/python-vulnerable-app
```

- Rust app(`example/rust-vulnerable-app`) + GPT-4o-latest
```bash
export OPENAI_API_KEY=your-api-key
cargo run -- -r example/rust-vulnerable-app -m gpt-4o
```

These examples are for educational purposes only. Do not use them in production environments.

## Documentation

- See [DEVELOPMENT.md](DEVELOPMENT.md) for development guidelines and contributing information
- Check the `example/` directory for sample vulnerable applications

## Security

This tool is intended for security research and educational purposes only. Do not use the example vulnerable applications in production environments.

## License

AGPL 3.0

## Acknowledgements

This project was inspired by the [protectai/vulnhunter](https://github.com/protectai/vulnhuntr).

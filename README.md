# Monteur

A lightweight containerized utility for building Java projects from source URLs.

## Overview

Monteur is a Rust-based and containerized tool designed to automate the process of downloading, extracting, and building Java projects from source archives. It supports both Maven and Gradle build systems and handles the extraction of project artifacts. It is used at [ziffer.dev](https://ziffer.dev) to build users' projects.

> Inspiration for this project was taken from [nixpacks](https://nixpacks.com), but because we needed just the artifacts instead of an OCI image, we decided to build our own tool.

## Usage

```bash
monteur <DOWNLOAD_URL>
```

Where `<DOWNLOAD_URL>` is the URL to a tar.gz archive containing the Java project source code.

## Requirements

- Rust (for building from source)
- Maven (depending on the target project)
- Java Development Kit (that's used in the project)

## Docker Usage

Monteur is available as a Docker image with Maven and JDK 21 pre-installed:

```bash
docker run --rm -v /host/path/to/output:/output ghcr.io/zifferdev/monteur:latest <DOWNLOAD_URL>
```

## How It Works

1. Downloads the source code archive from the specified URL (must be .tar.gz)
2. Extracts the archive to a temporary directory
3. Detects the build system (Maven or Gradle)
4. Builds the project with appropriate commands:
   - Maven: `mvn clean package -Dmaven.test.skip=true`
   - Gradle: `./gradlew clean build -x check -x test`
5. Identifies the target JAR file using smart selection rules
6. Copies the JAR file to the output directory

## Building from Source

```bash
git clone https://github.com/zifferdev/monteur.git
cd monteur
cargo build --release
```

## License

MIT License - see the included LICENSE file for more details.

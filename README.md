# GPU Stock Tracker

A simple Rust-based application that periodically checks GPU stock availability on specified URLs. If an item is in stock, it sends a notification via **Discord** or **SMS (Twilio)**.

This tracker:
- Reads a `config.yaml` file to get GPU URLs, how to detect in-stock status, and notification settings.  
- Runs continuously, polling at a set interval.  
- Sends notifications through your chosen channel (Discord or Twilio SMS).

## Table of Contents

1. [Prerequisites](#prerequisites)  
2. [Configuration](#configuration)  
3. [Running Locally (Cargo)](#running-locally-cargo)  
4. [Docker Usage](#docker-usage)  
    - [Using Prebuilt Docker Images](#using-prebuilt-docker-images)  
    - [Local Docker Build (Single Architecture)](#local-docker-build-single-architecture)  
    - [GitHub Actions (Multi-Arch Build)](#github-actions-multi-arch-build)  
    - [Running the Docker Image](#running-the-docker-image)  
5. [Setting Up a Discord Webhook](#setting-up-a-discord-webhook)

---

## Prerequisites

- **Rust** (if you want to run locally, without Docker)  
  - Rust 1.60+ recommended (the example uses 2021 edition).  
- **Docker** if you plan to run in a container.  
  - For multi-architecture builds, ensure you have Docker 19.03+ with [Buildx](https://docs.docker.com/build/buildx/).  

---

## Configuration

Create or edit a file called `config.yaml` (in the project root). Example:

```yaml
notification:
  method: "discord"                    # or "sms"
  discord_webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
  twilio_account_sid: "ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  twilio_auth_token: "your_auth_token"
  twilio_from_number: "+1234567890"
  twilio_to_number: "+1987654321"

monitor_interval_sec: 60               # how often to poll for stock (in seconds)

gpus:
  - name: "NVIDIA GeForce RTX 4080"
    url: "https://www.example.com/product1"
    in_stock_selector: ".nav-col.has-qty-box"

  - name: "AMD Radeon RX 7900"
    url: "https://www.example.com/product2"
    in_stock_selector: ".stock-indicator[data-stock='true']"
```

## Running Locally (Cargo)

1. Install Rust (if you haven’t).
2. Clone or download this repository.
3. Update the config.yaml to reflect your URLs and notification settings.
4. Run:

```bash
cargo run
```

The tracker will:
1. Continuously poll each GPU URL.
2. Print [IN STOCK] or [OUT OF STOCK] to the console.
3. If in stock, it attempts a Discord or Twilio notification.

## Docker Usage

### Using Prebuilt Docker Images

Prebuilt images are available on Docker Hub under the username `deepvolition`. To use a prebuilt image, simply pull and run it:

```bash
docker pull deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture
```

Then run the container:

```bash
docker run --rm -it \
  -v "$PWD/config.yaml:/config.yaml" \
  deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture
```

This will ensure you are always using the latest built image without needing to compile it manually.

### Local Docker Build (Single Architecture)

You can build a Docker image locally (just for your machine’s architecture):

```bash
# Example for x86_64 / amd64
docker build -t gpu-stock-tracker:latest .
```
This uses the Dockerfile in the root directory, which does a multi-stage build:

1. Builder stage: compiles the Rust application in release mode.
2. Final stage: copies the compiled binary into a minimal base image.

#### Running the container (assuming you have config.yaml in your current directory):

```bash
docker run --rm -it \
  -v "$PWD/config.yaml:/config.yaml" \
  gpu-stock-tracker:latest
```

The container will load `config.yaml` at runtime and start monitoring.

### GitHub Actions (Multi-Arch Build)

If you want to build images for both amd64 and arm64 and push them to Docker Hub (or another registry), you can use a GitHub Actions workflow like this (`.github/workflows/build-and-push.yml`):

```yaml
name: Build and Push Docker Images

on:
  push:
    branches: [ "main" ]

jobs:
  build-and-push:
    runs-on: ubuntu-latest

    steps:
      - name: Check out code
        uses: actions/checkout@v3

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v2
        with:
          platforms: all

      - name: Set up Buildx
        uses: docker/setup-buildx-action@v2

      - name: Log in to Docker registry
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}

      - name: Build & push multi-arch
        uses: docker/build-push-action@v3
        with:
          context: .
          file: ./Dockerfile
          platforms: linux/amd64,linux/arm64
          push: true
          tags: deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture
```

This workflow:

1. Runs on GitHub’s ubuntu-latest runner (x86_64).
2. Sets up QEMU to emulate arm64.
3. Builds a multi-architecture Docker image supporting both amd64 and arm64.
4. Pushes the final manifest to `deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture`.

(If you prefer separate tags for each arch, just split it into two build steps with platforms: linux/amd64 and platforms: linux/arm64, respectively.)

## Running the Docker Image

If you pulled the image from Docker Hub under `deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture`, you can run:

```bash
docker pull deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture

docker run --rm -it \
  -v "$PWD/config.yaml:/config.yaml" \
  deepvolition/gpu-stock-tracker:{arch}  # Replace {arch} with amd64 or arm64 depending on your system architecture
```

1. `-v "$PWD/config.yaml:/config.yaml"` mounts your local `config.yaml` into the container.
2. If the container is run on an ARM machine (like a Raspberry Pi 4), Docker will automatically pull and run the arm64 image if your image is multi-arch. On x86_64, it uses the amd64 image.

## Setting Up a Discord Webhook

To receive notifications via Discord, you need to set up a webhook:

1. Open Discord and navigate to your server settings.
2. Click on Integrations and then Webhooks.
3. Click New Webhook and give it a name.
4. Select the channel where you want notifications to appear.
5. Copy the generated webhook URL.
6. Paste this URL into the config.yaml file under discord_webhook_url.

Once configured, the GPU Stock Tracker will send messages to the designated Discord channel when stock status changes.
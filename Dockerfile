FROM node:20-slim AS base

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

EXPOSE 8787
CMD npx wrangler d1 migrations apply DB --env dev --local && \
    npx wrangler dev --env dev --port 8787 --ip 0.0.0.0
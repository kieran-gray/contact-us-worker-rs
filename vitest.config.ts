import { defineWorkersConfig, readD1Migrations } from "@cloudflare/vitest-pool-workers/config";

export default defineWorkersConfig(async () => {
  const migrations = await readD1Migrations("./migrations");

  return {
    test: {
      poolOptions: {
        workers: {
          wrangler: { configPath: "./wrangler.toml" },
          miniflare: {
            bindings: {
              ENVIRONMENT: "test",
              TURNSTILE_SITEVERIFY_URL: "https://test.com/turnstile/v0/siteverify",
              TURNSTILE_SECRET_KEY: "test-secret-key",
              ALLOWED_ORIGINS: "http://localhost:5173",
              TEST_MIGRATIONS: migrations,
            },
            d1Databases: {
              DB: "test-db"
            },
          },
        },
      },
    },
  };
});
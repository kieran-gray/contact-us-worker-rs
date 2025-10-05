import { SELF, fetchMock, env, applyD1Migrations } from "cloudflare:test";
import { describe, it, expect, beforeAll, afterEach } from "vitest";

describe("Contact Us Worker", () => {
  beforeAll(async () => {
    fetchMock.activate();
    fetchMock.disableNetConnect();

    await applyD1Migrations(env.DB, env.TEST_MIGRATIONS);
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  it("responds with 404 for unknown routes", async () => {
    const response = await SELF.fetch("http://example.com/404");
    expect(response.status).toBe(404);
    expect(await response.text()).toBe("Not Found");
  });

  it("handles OPTIONS request for CORS preflight", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/contact-us/", {
      method: "OPTIONS",
      headers: {
        Origin: "http://localhost:5173",
      },
    });

    expect(response.status).toBe(200);
    expect(response.headers.get("Access-Control-Allow-Origin")).toBe("http://localhost:5173");
    expect(response.headers.get("Access-Control-Allow-Methods")).toBe("POST, OPTIONS");
    expect(response.headers.get("Access-Control-Allow-Headers")).toBe("Content-Type");
  });

  it("rejects POST request without required fields", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/contact-us/", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: "http://localhost:5173",
      },
      body: JSON.stringify({}),
    });

    expect(response.status).toBe(400);
  });

  it("accepts valid POST request with all required fields", async () => {
    fetchMock
      .get("https://test.com")
      .intercept({ method: "POST", path: "/turnstile/v0/siteverify" })
      .reply(200, JSON.stringify({ success: true }));

    const response = await SELF.fetch("http://example.com/api/v1/contact-us/", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: "http://localhost:5173",
      },
      body: JSON.stringify({
        category: "IDEA",
        email: "test@example.com",
        name: "Test User",
        message: "This is a test message",
        token: "test-token",
      }),
    });

    const data = await response.json();
    console.log("Response status:", response.status);
    console.log("Response data:", data);
    expect(response.status).toBe(200);
    expect(data).toHaveProperty("message");
  });

  it("blocks requests from disallowed origins", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/contact-us/", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: "http://evil.com",
      },
      body: JSON.stringify({
        category: "IDEA",
        email: "test@example.com",
        name: "Test User",
        message: "This is a test message",
        token: "test-token",
      }),
    });

    expect(response.status).toBe(403);
    expect(response.headers.get("Access-Control-Allow-Origin")).toBeNull();
  });

  it("rejects request when Turnstile validation fails", async () => {
    fetchMock
      .get("https://test.com")
      .intercept({ method: "POST", path: "/turnstile/v0/siteverify" })
      .reply(200, JSON.stringify({
        success: false,
        "error-codes": ["invalid-input-response"]
      }));

    const response = await SELF.fetch("http://example.com/api/v1/contact-us/", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: "http://localhost:5173",
      },
      body: JSON.stringify({
        category: "IDEA",
        email: "test@example.com",
        name: "Test User",
        message: "This is a test message",
        token: "invalid-token",
      }),
    });

    expect(response.status).toBe(401);
  });
});
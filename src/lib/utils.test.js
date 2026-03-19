import { describe, it, expect } from "vitest";
import {
  formatSize,
  isValidDbName,
  projectDomain,
  statusCategory,
  sanitizeDbName,
} from "./utils.js";

describe("formatSize", () => {
  it("formats zero bytes", () => {
    expect(formatSize(0)).toBe("0 B");
  });

  it("formats bytes", () => {
    expect(formatSize(512)).toBe("512 B");
    expect(formatSize(1)).toBe("1 B");
  });

  it("formats kilobytes", () => {
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(2048)).toBe("2.0 KB");
    expect(formatSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(formatSize(1048576)).toBe("1.0 MB");
    expect(formatSize(10485760)).toBe("10.0 MB");
  });

  it("formats gigabytes", () => {
    expect(formatSize(1073741824)).toBe("1.0 GB");
    expect(formatSize(2147483648)).toBe("2.0 GB");
  });
});

describe("isValidDbName", () => {
  it("accepts valid names", () => {
    expect(isValidDbName("my_database")).toBe(true);
    expect(isValidDbName("test123")).toBe(true);
    expect(isValidDbName("DB_NAME")).toBe(true);
  });

  it("rejects invalid names", () => {
    expect(isValidDbName("my-database")).toBe(false);
    expect(isValidDbName("drop; --")).toBe(false);
    expect(isValidDbName("")).toBe(false);
    expect(isValidDbName("my database")).toBe(false);
  });
});

describe("projectDomain", () => {
  it("generates .test domain", () => {
    expect(projectDomain("myapp")).toBe("myapp.test");
    expect(projectDomain("blog")).toBe("blog.test");
  });
});

describe("statusCategory", () => {
  it("classifies running", () => {
    expect(statusCategory("Running")).toBe("running");
  });

  it("classifies stopped", () => {
    expect(statusCategory("Stopped")).toBe("stopped");
  });

  it("classifies installed", () => {
    expect(statusCategory("Installed")).toBe("installed");
  });

  it("classifies errors", () => {
    expect(statusCategory("Error: timeout")).toBe("error");
  });

  it("classifies unknown", () => {
    expect(statusCategory(null)).toBe("unknown");
    expect(statusCategory("")).toBe("unknown");
  });
});

describe("sanitizeDbName", () => {
  it("passes through valid names", () => {
    expect(sanitizeDbName("myapp")).toBe("myapp");
    expect(sanitizeDbName("test_123")).toBe("test_123");
  });

  it("replaces invalid characters with underscore", () => {
    expect(sanitizeDbName("my-app")).toBe("my_app");
    expect(sanitizeDbName("my app")).toBe("my_app");
    expect(sanitizeDbName("my.app")).toBe("my_app");
  });
});

import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import ts from "typescript";

const source = await readFile(
  new URL("../src/lib/text-search.ts", import.meta.url),
  "utf8",
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2020,
  },
}).outputText;
const tempDir = await mkdtemp(
  join(dirname(fileURLToPath(import.meta.url)), ".text-search-"),
);
const modulePath = join(tempDir, "text-search.mjs");
await writeFile(modulePath, compiled);
const { matchesTextQuery } = await import(pathToFileURL(modulePath).href);
await rm(tempDir, { recursive: true, force: true });

test("空查询匹配全部，文本查询忽略大小写和两端空格", () => {
  assert.equal(matchesTextQuery("", "QuickPane"), true);
  assert.equal(
    matchesTextQuery("  DOCS ", "开发文档", "https://docs.example"),
    true,
  );
  assert.equal(
    matchesTextQuery("missing", "开发文档", "https://docs.example"),
    false,
  );
});

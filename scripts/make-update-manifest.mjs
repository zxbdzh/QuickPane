#!/usr/bin/env node
import { writeFile } from "node:fs/promises";

const required = ["UPDATE_VERSION", "UPDATE_URL", "UPDATE_SIGNATURE"];
for (const name of required) {
  if (!process.env[name]) throw new Error(`${name} is required`);
}

const manifest = {
  version: process.env.UPDATE_VERSION,
  notes: process.env.UPDATE_NOTES || "",
  pub_date: process.env.UPDATE_PUB_DATE || new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: process.env.UPDATE_SIGNATURE,
      url: process.env.UPDATE_URL,
    },
  },
};

const output = process.env.UPDATE_MANIFEST || "latest.json";
await writeFile(output, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
process.stdout.write(`Wrote ${output}\n`);

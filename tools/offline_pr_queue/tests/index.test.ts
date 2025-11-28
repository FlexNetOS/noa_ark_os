import { readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

const testsDirUrl = new URL(".", import.meta.url);
const testsDirPath = fileURLToPath(testsDirUrl);

for (const entry of readdirSync(testsDirPath, { withFileTypes: true })) {
  if (!entry.isFile()) continue;
  if (!entry.name.endsWith(".test.ts")) continue;
  if (entry.name === "index.test.ts") continue;
  const fileUrl = new URL(entry.name, testsDirUrl);
  await import(fileUrl.href);
}

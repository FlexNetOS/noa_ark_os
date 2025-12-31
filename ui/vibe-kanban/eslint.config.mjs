import { FlatCompat } from "@eslint/eslintrc";
import js from "@eslint/js";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
	baseDirectory: __dirname,
	recommendedConfig: js.configs.recommended,
	allConfig: js.configs.all,
});
const require = createRequire(import.meta.url);
const legacyConfig = require("./.eslintrc.cjs");

export default [
	...compat.config(legacyConfig),
	{
		ignores: ["eslint.config.mjs", ".eslintrc.cjs", "scripts/**"],
	},
];

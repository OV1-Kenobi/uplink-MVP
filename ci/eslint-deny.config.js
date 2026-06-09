/**
 * ESLint boundary enforcement for Uplink web/
 *
 * BOUNDARY RULE: TypeScript source files outside web/src/wasm/ are FORBIDDEN
 * from calling fetch(), new WebSocket(), new EventSource(), or XMLHttpRequest.
 *
 * All network operations must go through web/src/wasm/uplink-client.ts,
 * which is the only file that may call into the wasm-bindgen bundle.
 *
 * Run: eslint web/src --config ci/eslint-deny.config.js --max-warnings 0
 */

import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

// Parser + plugin are declared in web/package.json. Resolve them from
// web/node_modules relative to this config's own location (not cwd), so the
// documented `cd web && eslint --config ../ci/eslint-deny.config.js` command works.
const require = createRequire(
  resolve(dirname(fileURLToPath(import.meta.url)), "../web/package.json"),
);
const tsParser = require("@typescript-eslint/parser");
const tsPlugin = require("@typescript-eslint/eslint-plugin");

export default [
  {
    // Match both documented invocations: `cd web && eslint src` (cwd=web/) and
    // `eslint web/src` from the repo root — globs are cwd-relative in flat config.
    files: ["src/**/*.{ts,tsx}", "web/src/**/*.{ts,tsx}"],
    ignores: [
      // The wasm boundary directory is allowed to interface with wasm
      "src/wasm/**",
      "web/src/wasm/**",
    ],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
      },
    },
    plugins: {
      "@typescript-eslint": tsPlugin,
    },
    rules: {
      // Ban raw fetch() calls outside the wasm wrapper
      "no-restricted-globals": [
        "error",
        {
          name: "fetch",
          message:
            "Do not call fetch() directly. Use uplink-client.ts instead (BOUNDARY rule).",
        },
        {
          name: "XMLHttpRequest",
          message: "Do not use XMLHttpRequest. Use uplink-client.ts instead.",
        },
      ],

      // Ban new WebSocket() and new EventSource() via no-restricted-syntax
      "no-restricted-syntax": [
        "error",
        {
          selector: "NewExpression[callee.name='WebSocket']",
          message:
            "Do not open raw WebSocket connections. All relay/LSP connections must go through uplink-core wasm (BOUNDARY rule).",
        },
        {
          selector: "NewExpression[callee.name='EventSource']",
          message:
            "Do not use EventSource directly. Use uplink-client.ts instead (BOUNDARY rule).",
        },
      ],
    },
  },
];

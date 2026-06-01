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

import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";

export default [
  {
    files: ["web/src/**/*.{ts,tsx}"],
    ignores: [
      // The boundary file itself is allowed to interface with wasm
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

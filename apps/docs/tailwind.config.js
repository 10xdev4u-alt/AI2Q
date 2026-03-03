const { createPreset } = require("fumadocs-ui/tailwind-plugin");

/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "class",
  content: [
    "./components/**/*.{ts,tsx}",
    "./app/**/*.{ts,tsx}",
    "./content/**/*.{md,mdx}",
    "./node_modules/fumadocs-ui/dist/**/*.js",
  ],
  presets: [
    createPreset({
      preset: "ocean",
    }),
  ],
  theme: {
    extend: {
      colors: {
        accent: {
          DEFAULT: "#6c5ce7",
          foreground: "#ffffff",
        },
      },
    },
  },
};

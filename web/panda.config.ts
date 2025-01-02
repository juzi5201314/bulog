import { defineConfig } from "@pandacss/dev";
import { createPreset } from '@park-ui/panda-preset'
import accent from '@park-ui/panda-preset/colors/neutral'
import gray from '@park-ui/panda-preset/colors/mauve'

export default defineConfig({
  // Whether to use css reset
  preflight: true,

  // Where to look for your css declarations
  include: ["./src/**/*.{js,jsx,ts,tsx}", "./pages/**/*.{js,jsx,ts,tsx}"],

  // Files to exclude
  exclude: [],

  // Useful for theme customization
  theme: {
    extend: {},
  },

  presets: [createPreset({ accentColor: accent, grayColor: gray, radius: 'sm' })],

  jsxFramework: "react",

  // The output directory for your css system
  outdir: "styled-system",
});

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./assets/**/*.html"],
  theme: {
    extend: {
      colors: {
        gruvbox: {
          black: '#282828', // Background color
          red: '#cc241d',
          green: '#98971a',
          yellow: '#d79921',
          blue: '#458588',
          purple: '#b16286',
          aqua: '#689d6a',
          gray: '#a89984', // Default text color
          orange: '#d65d0e',
        },
      },
    },
  },
  plugins: [],
}


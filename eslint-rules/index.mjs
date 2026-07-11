import noRawUiElements from "./no-raw-ui-elements.mjs";

/** @type {import('eslint').ESLint.Plugin} */
const plugin = {
  rules: {
    "no-raw-ui-elements": noRawUiElements,
  },
};

export default plugin;

import noRawUiElements from "./no-raw-ui-elements.mjs";
import noDerivedClosure from "./no-derived-closure.mjs";

/** @type {import('eslint').ESLint.Plugin} */
const plugin = {
  rules: {
    "no-raw-ui-elements": noRawUiElements,
    "no-derived-closure": noDerivedClosure,
  },
};

export default plugin;

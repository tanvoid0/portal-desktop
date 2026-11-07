import noRawUiElements from './no-raw-ui-elements.mjs';
import noDerivedClosure from './no-derived-closure.mjs';
import noLegacySyntax from './no-legacy-syntax.mjs';

/** @type {import('eslint').ESLint.Plugin} */
const plugin = {
  rules: {
    'no-raw-ui-elements': noRawUiElements,
    'no-derived-closure': noDerivedClosure,
    'no-legacy-syntax': noLegacySyntax,
  },
};

export default plugin;

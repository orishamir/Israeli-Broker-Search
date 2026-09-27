/** @type {import('stylelint').Config} */
export default {
  extends: ['stylelint-config-recommended'],
  plugins: ['stylelint-plugin-use-baseline'],
  // The <style> blocks of Svelte components.
  overrides: [{ files: ['**/*.svelte'], customSyntax: 'postcss-html' }],
  ignoreFiles: ['dist/**', 'test-results/**', 'src/lib/core/**'],
  rules: {
    // Flags CSS that browsers of the last 2.5 years don't all support
    // (Baseline "widely available"), the same target Vite builds for.
    // Allowed: newer, but older browsers just go without them. The
    // -webkit- prefixes Safari needs are added by Vite's build.
    'plugin/use-baseline': [
      true,
      {
        available: 'widely',
        ignoreProperties: {
          // A blur behind popovers and the dialog.
          'backdrop-filter': [],
          // Colored checkboxes and sliders.
          'accent-color': [],
          // The full tariff's opening animation.
          'interpolate-size': ['allow-keywords'],
          'user-select': ['none'],
        },
        // Fade-ins as things appear.
        ignoreAtRules: ['starting-style'],
        // The full tariff's opening animation.
        ignoreSelectors: ['details-content'],
      },
    ],
    // Svelte's way of styling outside the component.
    'selector-pseudo-class-no-unknown': [true, { ignorePseudoClasses: ['global'] }],
  },
}

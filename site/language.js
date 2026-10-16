// Picks the language before the page is drawn: the visitor's choice, otherwise the browser's if the site has it,
// otherwise English, already written in the page. Another language hides the page until site.js has put its texts in,
// rather than showing the English ones for an instant.
const LANGUAGE_KEY = 'squatsouls.siteLanguage';
const browserLanguage = navigator.language.slice(0, 2);
// The lang attribute only changes once the texts are in, so it stays right if they cannot be loaded.
document.documentElement.dataset.language = localStorage.getItem(LANGUAGE_KEY)
  ?? (['en', 'fr'].includes(browserLanguage) ? browserLanguage : 'en');
if (document.documentElement.dataset.language !== 'en') document.documentElement.classList.add('translating');

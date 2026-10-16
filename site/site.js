import { pickSlogan } from './slogans.js';

const LANGUAGE_KEY = 'squatsouls.siteLanguage';
const LANGUAGES = ['en', 'fr'];

// System of the visitor, matching a data-os card, or null on a phone, a tablet or an unknown system.
function detectOs() {
  const platform = (navigator.userAgentData?.platform || navigator.platform || '').toLowerCase();
  if (/android|iphone|ipad/i.test(navigator.userAgent)) return null;
  if (platform.startsWith('win')) return 'windows';
  // An iPad also says it runs on a Mac, but has a touch screen.
  if (platform.startsWith('mac')) return navigator.maxTouchPoints > 1 ? null : 'macos';
  if (platform.includes('linux')) return 'linux';
  return null;
}

const os = detectOs();
const card = os && document.querySelector(`[data-os="${os}"]`);

function setLanguage(language, texts) {
  document.documentElement.lang = language;
  document.querySelectorAll('[data-t]').forEach((el) => {
    el.textContent = texts[el.dataset.t];
  });
  document.querySelectorAll('[data-alt]').forEach((el) => {
    el.alt = texts[el.dataset.alt];
  });
  document.getElementById('cta').textContent = card
    ? texts.cta.replace('{os}', card.querySelector('h3').textContent)
    : texts.ctaAny;
  document.querySelectorAll('.lang button').forEach((b) => b.classList.toggle('on', b.dataset.lang === language));
}

async function loadLanguage(language) {
  const response = await fetch(`locales/${language}.json`);
  setLanguage(language, await response.json());
}

document.getElementById('slogan').textContent = pickSlogan();

// The main button downloads directly when the system has a single package, otherwise leads to its card.
if (card) {
  card.classList.add('mine');
  const buttons = card.querySelectorAll('.btn');
  if (buttons.length === 1) document.getElementById('cta').href = buttons[0].href;
} else {
  document.getElementById('other').hidden = true;
}

document.querySelectorAll('.lang button').forEach((b) => b.addEventListener('click', () => {
  localStorage.setItem(LANGUAGE_KEY, b.dataset.lang);
  loadLanguage(b.dataset.lang);
}));

const browserLanguage = navigator.language.slice(0, 2);
loadLanguage(localStorage.getItem(LANGUAGE_KEY) ?? (LANGUAGES.includes(browserLanguage) ? browserLanguage : 'en'));

import { pickSlogan } from './slogans.js';
/* global LANGUAGE_KEY -- declared by language.js, loaded before */

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
let shownTexts = {};

function showScreenshots(language) {
  document.querySelectorAll('[data-shot]').forEach((el) => {
    el.src = `screenshots/${language}/${el.dataset.shot}`;
  });
}

function setLanguage(language, texts) {
  document.documentElement.lang = language;
  shownTexts = texts;
  document.querySelectorAll('[data-t]').forEach((el) => {
    el.textContent = texts[el.dataset.t];
  });
  document.querySelectorAll('[data-alt]').forEach((el) => {
    el.alt = texts[el.dataset.alt];
  });
  showScreenshots(language);
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

// A screenshot opens full size, the arrow keys go through the others in page order, a click anywhere or Escape closes it.
const lightbox = document.getElementById('lightbox');
const lightboxImage = lightbox.querySelector('img');
const lightboxTitle = lightbox.querySelector('h3');
const lightboxText = lightbox.querySelector('p');
const shots = [...document.querySelectorAll('img.shot')];
let shown = 0;

// The caption repeats the title and text of the screenshot's card; the hero one has no card and names its texts itself.
function showShot(index) {
  shown = (index + shots.length) % shots.length;
  const shot = shots[shown];
  const shotCard = shot.closest('.card');
  lightboxImage.src = shot.src;
  lightboxImage.alt = shot.alt;
  lightboxTitle.textContent = shotCard ? shotCard.querySelector('h3').textContent : shownTexts[shot.dataset.title];
  lightboxText.textContent = shotCard ? shotCard.querySelector('p').textContent : shownTexts[shot.dataset.text];
}

shots.forEach((shot, index) => shot.addEventListener('click', () => {
  showShot(index);
  lightbox.showModal();
}));
lightbox.addEventListener('click', () => lightbox.close());
lightbox.addEventListener('keydown', (event) => {
  if (event.key === 'ArrowRight') showShot(shown + 1);
  if (event.key === 'ArrowLeft') showShot(shown - 1);
});

// LANGUAGE_KEY and the language come from language.js. The screenshots start loading without waiting for the texts;
// if these cannot be loaded, the page shows up again in English, as written.
const language = document.documentElement.dataset.language;
showScreenshots(language);
loadLanguage(language)
  .catch(() => showScreenshots('en'))
  .finally(() => document.documentElement.classList.remove('translating'));

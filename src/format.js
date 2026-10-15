import { language, t } from './i18n.js';

export function formatDay(iso) {
  const [y, m, d] = iso.split('-').map(Number);
  const year = y === new Date().getFullYear() ? undefined : 'numeric';
  return new Date(y, m - 1, d).toLocaleDateString(language.value, { weekday: 'short', day: 'numeric', month: 'short', year });
}

export function formatTime(seconds) {
  return new Date(seconds * 1000).toLocaleTimeString(language.value, { hour: '2-digit', minute: '2-digit' });
}

// Countdown in minutes and seconds: 1:05.
export function formatCountdown(seconds) {
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
}

// Duration rounded to the minute, in hours past one hour.
export function formatDuration(seconds) {
  const minutes = Math.round(seconds / 60);
  if (minutes < 1) return t('duration.underMinute');
  if (minutes < 60) return t('duration.minutes', { minutes });
  return t('duration.hours', { hours: Math.floor(minutes / 60), minutes: String(minutes % 60).padStart(2, '0') });
}

// Session summary to paste in a messaging app: game and duration, events triggered, totals per
// exercise, then jokers if they were recorded.
export function sessionSummary({ game, duration, events, totals, jokers }) {
  return [
    `${game} · ${formatDuration(duration)}`,
    events.map(({ event, count }) => t('history.eventCount', { event, count })).join(' · '),
    formatTotals(totals),
    jokers && t('history.jokersLine', { earned: t('history.earned', jokers.earned), spent: t('history.spent', jokers.spent) }),
  ].filter(Boolean).join('\n');
}

function formatTotals(totals) {
  return totals.map((total) => `${total.exercise} ${total.amount} ${total.unit}`).join(' · ');
}

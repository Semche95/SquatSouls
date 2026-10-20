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
// exercise, then jokers if they were recorded. The start time leads when given.
export function sessionSummary({ game, duration, events, totals, jokers }, start) {
  return [
    [start !== undefined && formatTime(start), game, formatDuration(duration)].filter(Boolean).join(' · '),
    events.map(({ event, count }) => t('history.eventCount', { event, count })).join(' · '),
    formatTotals(totals),
    jokers && t('history.jokersLine', { earned: t('history.earned', jokers.earned), spent: t('history.spent', jokers.spent) }),
  ].filter(Boolean).join('\n');
}

// Day summary: the day, each session with its start time, then the totals per exercise of the day.
export function daySummary({ date, sessions }) {
  const totals = [];
  for (const total of sessions.flatMap((se) => se.totals)) {
    const same = totals.find((x) => x.exercise === total.exercise && x.unit === total.unit);
    if (same) same.amount += total.amount;
    else totals.push({ ...total });
  }
  const body = sessions.map((se) => sessionSummary(se, se.start)).join('\n\n');
  return `${formatDay(date)}\n${body}\n\n${t('history.dayTotal', { totals: formatTotals(totals) })}`;
}

function formatTotals(totals) {
  return totals.map((total) => `${total.exercise} ${total.amount} ${total.unit}`).join(' · ');
}

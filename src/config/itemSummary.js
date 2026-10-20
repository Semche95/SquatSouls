import { t } from '../i18n/i18n.js';

// Texts summing up an event or an exercise field by field, always in the same order; `plain` marks
// the default value of an optional field, left out unless it changes.

// Timer of an exercise, '' when it is not timed.
export const timerSummary = (ex) => (ex.timed ? t(`config.exercises.${ex.eachSide ? 'timedSideSummary' : 'timedSummary'}.${ex.timed}`) : '');

const field = (text, plainText) => (text ? { text } : { text: plainText, plain: true });

export const eventSummary = (ev) => [
  field(t('config.event.perSet', ev.draws)),
  field(ev.maxPerSession && t('config.event.maxPerSession', ev.maxPerSession), t('config.event.noLimitSummary')),
  field(ev.jokers && t('common.reward', ev.jokers), t('config.event.noJokers')),
];

export const exerciseSummary = (ex) => [
  field(t('config.exercises.rangeUnit', { min: ex.min, max: ex.max, unit: ex.unit })),
  field(timerSummary(ex), t('config.exercises.notTimed')),
  field(ex.recovery !== undefined && t('config.exercises.recoverySummary', { n: ex.recovery }), t('config.exercises.generalRecovery')),
];

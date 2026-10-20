<script setup>
import { computed } from 'vue';
import { t } from '../i18n/i18n.js';

// Changes an import brings (`importChanges.js`), grouped by status with a switch each and one for the whole
// group; `chosen` is the Set of the checked keys. `summary` sums up an item field by field (`itemSummary.js`).
// The rows are computed once per list, a click only replaces the Set: the list holds up to a few hundred rows.
const props = defineProps({ changes: { type: Array, required: true }, summary: { type: Function, required: true } });
const chosen = defineModel('chosen', { type: Set, required: true });

const STATUSES = ['new', 'changed', 'absent'];

const shown = (item) => props.summary(item).filter((f) => !f.plain).map((f) => f.text);
function detail({ status, before, after }) {
  if (status === 'new') return shown(after);
  if (status === 'absent') return shown(before);
  const [from, to] = [before, after].map(props.summary);
  return to.flatMap((f, i) => (f.text === from[i].text ? [] : [t('share.change', { before: from[i].text, after: f.text })]));
}

const groups = computed(() =>
  STATUSES.map((status) => ({ status, rows: props.changes.filter((c) => c.status === status).map((c) => ({ key: c.key, name: c.name, detail: detail(c) })) })).filter(
    (g) => g.rows.length,
  ),
);

const checkedIn = (group) => group.rows.filter((row) => chosen.value.has(row.key)).length;

function toggle(keys, on) {
  const next = new Set(chosen.value);
  for (const key of keys) on ? next.add(key) : next.delete(key);
  chosen.value = next;
}
</script>

<template>
  <div class="import-preview">
    <p v-if="!groups.length">{{ t('share.nothing') }}</p>
    <div v-for="group in groups" :key="group.status" class="import-group" role="group" :aria-labelledby="`import-${group.status}`">
      <div class="import-group-head">
        <h4 :id="`import-${group.status}`">{{ t(`share.${group.status}`) }} <span class="import-count">{{ group.rows.length }}</span></h4>
        <label v-if="group.rows.length > 1" class="check import-all">
          {{ t('share.all') }}
          <input type="checkbox" :checked="checkedIn(group) === group.rows.length" @change="toggle(group.rows.map((row) => row.key), $event.target.checked)" />
        </label>
      </div>
      <p v-if="group.status === 'absent'" class="import-hint">{{ t('share.absentHint') }}</p>
      <ul>
        <li v-for="row in group.rows" :key="row.key">
          <label class="check">
            <span class="import-text">
              <span class="import-name">{{ row.name }}</span>
              <span v-if="row.detail.length" class="import-detail">
                <span v-for="(part, i) in row.detail" :key="i">{{ part }}</span>
              </span>
            </span>
            <input type="checkbox" :checked="chosen.has(row.key)" @change="toggle([row.key], $event.target.checked)" />
          </label>
        </li>
      </ul>
    </div>
  </div>
</template>

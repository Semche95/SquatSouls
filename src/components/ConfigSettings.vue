<script setup>
import { computed } from 'vue';
import { BOUNDS, byName } from '../config/configDraft.js';
import { languageSetting, setLanguage, t } from '../i18n/i18n.js';
import { vReveal } from '../ui/reveal.js';
import { useApi } from '../composables/useApi.js';
import { useConfig } from '../composables/useConfig.js';
import { useSteam } from '../composables/useSteam.js';
import CountStepper from './CountStepper.vue';

const { call } = useApi();
const { config, draft } = useConfig();
const { detected, storeBlocked, unignore } = useSteam();

const LANGUAGES = [
  { value: 'system', label: () => t('config.settings.systemLanguage') },
  { value: 'en', label: () => 'English' },
  { value: 'fr', label: () => 'Français' },
];

// Saved and applied right away, outside the draft. Refused, the saved setting stays chosen.
async function changeLanguage(value) {
  const { setting, language } = await call('set_language', { setting: value }).catch(() => ({}));
  if (!setting) return;
  languageSetting.value = setting;
  await setLanguage(language);
}

// Status of the saved detection, to see it works even without a banner in Play:
// text and color of its badge.
const status = computed(() => {
  const d = detected.value;
  if (!d) return { text: t('config.settings.noGame'), kind: 'idle' };
  if (d.name) return { text: t(d.ignored ? 'config.settings.foundIgnored' : 'config.settings.found', { name: d.name }), kind: 'found' };
  return storeBlocked.value ? { text: t('config.settings.blocked'), kind: 'blocked' } : { text: t('config.settings.unnamed'), kind: 'idle' };
});
</script>

<template>
  <div class="cfg-settings">
    <section class="setting">
      <h2 class="setting-title">{{ t('config.settings.general') }}</h2>
      <div class="setting-card">
        <div class="setting-row check">
          <span id="language">{{ t('config.settings.language') }}</span>
          <div class="segmented" role="radiogroup" aria-labelledby="language">
            <button
              v-for="l in LANGUAGES"
              :key="l.value"
              role="radio"
              :aria-checked="languageSetting === l.value"
              :class="{ active: languageSetting === l.value }"
              @click="changeLanguage(l.value)"
            >
              {{ l.label() }}
            </button>
          </div>
        </div>
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.sound') }}
            <input v-model="draft.sound" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.soundHint') }}</p>
        </div>
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.rememberIntensity') }}
            <input v-model="draft.rememberIntensity" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.rememberIntensityHint') }}</p>
        </div>
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.rememberLimit') }}
            <input v-model="draft.rememberLimit" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.rememberLimitHint') }}</p>
        </div>
      </div>
    </section>
    <section class="setting">
      <h2 class="setting-title">{{ t('config.settings.timer') }}</h2>
      <div class="setting-card">
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.timerAutoDone') }}
            <input v-model="draft.timerAutoDone" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.timerHint') }}</p>
        </div>
        <div class="setting-row check">
          <label for="timer-prep">{{ t('fields.timerPrep') }}</label>
          <CountStepper id="timer-prep" v-model="draft.timerPrep" :bounds="BOUNDS.seconds" :unit="t('fields.secondsUnit')" />
        </div>
      </div>
    </section>
    <section v-reveal="draft.steamDetection" class="setting">
      <h2 class="setting-title">{{ t('config.settings.steam') }}</h2>
      <div class="setting-card">
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.detect') }}
            <input v-model="draft.steamDetection" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.detectHint') }}</p>
        </div>
        <label v-if="draft.steamDetection" class="setting-row nested check">
          {{ t('config.settings.notify') }}
          <input v-model="draft.steamNotify" type="checkbox" />
        </label>
        <details v-if="config.ignoredGames.length" class="ignored-games">
          <summary class="setting-row setting-sub">{{ t('config.settings.ignored', { n: config.ignoredGames.length }) }}</summary>
          <ul class="ignored">
            <li v-for="g in byName(config.ignoredGames)" :key="g.appId" class="setting-row nested">
              <span class="ignored-name">{{ g.name }}</span>
              <button class="ghost" @click="unignore(g.appId)">{{ t('config.settings.unignore') }}</button>
            </li>
          </ul>
        </details>
        <p v-if="config.steamDetection" class="status" :class="status.kind">{{ status.text }}</p>
      </div>
    </section>
    <section v-reveal="draft.tray" class="setting">
      <h2 class="setting-title">{{ t('config.settings.tray') }}</h2>
      <div class="setting-card">
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.showIcon') }}
            <input v-model="draft.tray" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.trayHint') }}</p>
        </div>
        <template v-if="draft.tray">
          <label class="setting-row nested check">
            {{ t('config.settings.closeToTray') }}
            <input v-model="draft.closeToTray" type="checkbox" />
          </label>
          <label class="setting-row nested check">
            {{ t('config.settings.startHidden') }}
            <input v-model="draft.startHidden" type="checkbox" />
          </label>
        </template>
      </div>
    </section>
    <section v-reveal="draft.updateCheck" class="setting">
      <h2 class="setting-title">{{ t('config.settings.updates') }}</h2>
      <div class="setting-card">
        <div class="setting-row">
          <label class="check">
            {{ t('config.settings.updateCheck') }}
            <input v-model="draft.updateCheck" type="checkbox" />
          </label>
          <p class="hint">{{ t('config.settings.updateCheckHint') }}</p>
        </div>
        <label v-if="draft.updateCheck" class="setting-row nested check">
          {{ t('config.settings.updateAuto') }}
          <input v-model="draft.updateAuto" type="checkbox" />
        </label>
      </div>
    </section>
  </div>
</template>

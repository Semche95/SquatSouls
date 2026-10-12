<script setup>
import { computed } from 'vue';
import { byName } from '../configDraft.js';
import { languageSetting, setLanguage, t } from '../i18n.js';
import { useApi } from '../composables/useApi.js';
import { useConfig } from '../composables/useConfig.js';
import { useSteam } from '../composables/useSteam.js';

const { call } = useApi();
const { config, draft } = useConfig();
const { detected, storeBlocked, addGame, unignore } = useSteam();

// Saved and applied right away, outside the draft. Refused, the list goes back to the saved setting.
async function changeLanguage({ target }) {
  try {
    const { setting, language } = await call('set_language', { setting: target.value });
    languageSetting.value = setting;
    await setLanguage(language);
  } catch {
    target.value = languageSetting.value;
  }
}

// Status of the saved detection, to see it works even without a banner in Jouer:
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
    <section class="group setting">
      <label class="check">
        {{ t('config.settings.language') }}
        <select :value="languageSetting" @change="changeLanguage">
          <option value="system">{{ t('config.settings.systemLanguage') }}</option>
          <option value="en">English</option>
          <option value="fr">Français</option>
        </select>
      </label>
    </section>
    <section class="group setting">
      <h2 class="group-title">{{ t('config.settings.tray') }}</h2>
      <div>
        <label class="check">
          {{ t('config.settings.showIcon') }}
          <input v-model="draft.tray" type="checkbox" />
        </label>
        <p class="hint">{{ t('config.settings.trayHint') }}</p>
      </div>
      <div v-if="draft.tray" class="nested">
        <label class="check">
          {{ t('config.settings.closeToTray') }}
          <input v-model="draft.closeToTray" type="checkbox" />
        </label>
        <label class="check">
          {{ t('config.settings.startHidden') }}
          <input v-model="draft.startHidden" type="checkbox" />
        </label>
      </div>
    </section>
    <section class="group setting">
      <h2 class="group-title">{{ t('config.settings.steam') }}</h2>
      <div>
        <label class="check">
          {{ t('config.settings.detect') }}
          <input v-model="draft.steamDetection" type="checkbox" />
        </label>
        <p class="hint">{{ t('config.settings.detectHint') }}</p>
      </div>
      <div v-if="draft.steamDetection" class="nested">
        <label class="check">
          {{ t('config.settings.notify') }}
          <input v-model="draft.steamNotify" type="checkbox" />
        </label>
      </div>
      <p v-if="config.steamDetection" class="status" :class="status.kind">{{ status.text }}</p>
      <template v-if="config.ignoredGames.length">
        <h3 class="group-title sub">{{ t('config.settings.ignored') }}</h3>
        <ul class="ignored">
          <li v-for="g in byName(config.ignoredGames)" :key="g.appId">
            {{ g.name }}
            <span class="row">
              <button class="ghost" @click="addGame(g)">{{ t('common.add') }}</button>
              <button class="ghost" @click="unignore(g.appId)">{{ t('config.settings.unignore') }}</button>
            </span>
          </li>
        </ul>
      </template>
    </section>
  </div>
</template>
